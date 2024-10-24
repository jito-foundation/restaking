use std::str::FromStr;

use anyhow::{anyhow, Result};
use jito_bytemuck::{AccountDeserialize, Discriminator};
use jito_vault_client::instructions::{
    CreateTokenMetadataBuilder, InitializeConfigBuilder, InitializeVaultBuilder,
};
use jito_vault_core::{config::Config, vault::Vault};
use jito_vault_sdk::inline_mpl_token_metadata;
use log::{debug, info};
use solana_account_decoder::UiAccountEncoding;
use solana_program::pubkey::Pubkey;
use solana_rpc_client::{nonblocking::rpc_client::RpcClient, rpc_client::SerializableTransaction};
use solana_rpc_client_api::{
    config::{RpcAccountInfoConfig, RpcProgramAccountsConfig},
    filter::{Memcmp, MemcmpEncodedBytes, RpcFilterType},
};
use solana_sdk::{
    signature::{Keypair, Signer},
    transaction::Transaction,
};

use crate::{
    vault::{ConfigActions, VaultActions, VaultCommands},
    CliConfig,
};

pub struct VaultCliHandler {
    cli_config: CliConfig,
    restaking_program_id: Pubkey,
    vault_program_id: Pubkey,
}

impl VaultCliHandler {
    pub const fn new(
        cli_config: CliConfig,
        restaking_program_id: Pubkey,
        vault_program_id: Pubkey,
    ) -> Self {
        Self {
            cli_config,
            restaking_program_id,
            vault_program_id,
        }
    }

    pub async fn handle(&self, action: VaultCommands) -> Result<()> {
        match action {
            VaultCommands::Config {
                action: ConfigActions::Initialize,
            } => self.initialize_config().await,
            VaultCommands::Config {
                action: ConfigActions::Get,
            } => self.get_config().await,
            VaultCommands::Vault {
                action:
                    VaultActions::Initialize {
                        token_mint,
                        deposit_fee_bps,
                        withdrawal_fee_bps,
                        reward_fee_bps,
                        decimals,
                    },
            } => {
                self.initialize_vault(
                    token_mint,
                    deposit_fee_bps,
                    withdrawal_fee_bps,
                    reward_fee_bps,
                    decimals,
                )
                .await
            }
            VaultCommands::Vault {
                action: VaultActions::Get { pubkey },
            } => self.get_vault(pubkey).await,
            VaultCommands::Vault {
                action: VaultActions::List,
            } => self.list_vaults().await,
            VaultCommands::Vault {
                action:
                    VaultActions::CreateTokenMetadata {
                        vault,
                        name,
                        symbol,
                        uri,
                    },
            } => self.create_token_metadata(vault, name, symbol, uri).await,
        }
    }

    pub async fn initialize_config(&self) -> Result<()> {
        let keypair = self
            .cli_config
            .keypair
            .as_ref()
            .ok_or_else(|| anyhow!("Keypair not provided"))?;
        let rpc_client = self.get_rpc_client();

        let mut ix_builder = InitializeConfigBuilder::new();
        let config_address = Config::find_program_address(&self.vault_program_id).0;
        let ix_builder = ix_builder
            .config(config_address)
            .admin(keypair.pubkey())
            .restaking_program(self.restaking_program_id);

        let blockhash = rpc_client.get_latest_blockhash().await?;
        let tx = Transaction::new_signed_with_payer(
            &[ix_builder.instruction()],
            Some(&keypair.pubkey()),
            &[keypair],
            blockhash,
        );
        info!("Initializing vault config parameters: {:?}", ix_builder);
        info!(
            "Initializing vault config transaction: {:?}",
            tx.get_signature()
        );
        rpc_client.send_and_confirm_transaction(&tx).await?;
        info!("Transaction confirmed: {:?}", tx.get_signature());
        Ok(())
    }

    async fn get_config(&self) -> Result<()> {
        let rpc_client = self.get_rpc_client();

        let config_address = Config::find_program_address(&self.vault_program_id).0;
        debug!(
            "Reading the restaking configuration account at address: {}",
            config_address
        );

        let account = rpc_client.get_account(&config_address).await?;
        let config = Config::try_from_slice_unchecked(&account.data)?;
        info!("Vault config at address {} : {:?}", config_address, config);
        Ok(())
    }

    pub async fn initialize_vault(
        &self,
        token_mint: String,
        deposit_fee_bps: u16,
        withdrawal_fee_bps: u16,
        reward_fee_bps: u16,
        decimals: u8,
    ) -> Result<()> {
        let token_mint = Pubkey::from_str(&token_mint)?;
        let keypair = self
            .cli_config
            .keypair
            .as_ref()
            .ok_or_else(|| anyhow!("Keypair not provided"))?;
        let rpc_client = self.get_rpc_client();

        let base = Keypair::new();
        let vault = Vault::find_program_address(&self.vault_program_id, &base.pubkey()).0;

        let vrt_mint = Keypair::new();

        let mut ix_builder = InitializeVaultBuilder::new();
        ix_builder
            .config(Config::find_program_address(&self.vault_program_id).0)
            .vault(vault)
            .vrt_mint(vrt_mint.pubkey())
            .st_mint(token_mint)
            .admin(keypair.pubkey())
            .base(base.pubkey())
            .deposit_fee_bps(deposit_fee_bps)
            .withdrawal_fee_bps(withdrawal_fee_bps)
            .reward_fee_bps(reward_fee_bps)
            .decimals(decimals);

        let blockhash = rpc_client.get_latest_blockhash().await?;
        let tx = Transaction::new_signed_with_payer(
            &[ix_builder.instruction()],
            Some(&keypair.pubkey()),
            &[keypair, &base, &vrt_mint],
            blockhash,
        );
        info!("Initializing vault transaction: {:?}", tx.get_signature());
        rpc_client.send_and_confirm_transaction(&tx).await?;
        info!("Transaction confirmed: {:?}", tx.get_signature());

        Ok(())
    }

    pub async fn get_vault(&self, pubkey: String) -> Result<()> {
        let pubkey = Pubkey::from_str(&pubkey)?;
        let rpc_client = self.get_rpc_client();
        let account = rpc_client.get_account(&pubkey).await?;
        let vault = Vault::try_from_slice_unchecked(&account.data)?;
        info!("vault at address {}: {:?}", pubkey, vault);
        Ok(())
    }

    pub async fn list_vaults(&self) -> Result<()> {
        let rpc_client = self.get_rpc_client();
        let accounts = rpc_client
            .get_program_accounts_with_config(
                &self.vault_program_id,
                RpcProgramAccountsConfig {
                    filters: Some(vec![RpcFilterType::Memcmp(Memcmp::new(
                        0,
                        MemcmpEncodedBytes::Bytes(vec![Vault::DISCRIMINATOR]),
                    ))]),
                    account_config: RpcAccountInfoConfig {
                        encoding: Some(UiAccountEncoding::Base64),
                        data_slice: None,
                        commitment: None,
                        min_context_slot: None,
                    },
                    with_context: None,
                },
            )
            .await?;
        for (vault_pubkey, vault) in accounts {
            let vault = Vault::try_from_slice_unchecked(&vault.data)?;
            info!("vault at address {}: {:?}", vault_pubkey, vault);
        }
        Ok(())
    }

    fn get_rpc_client(&self) -> RpcClient {
        RpcClient::new_with_commitment(self.cli_config.rpc_url.clone(), self.cli_config.commitment)
    }

    async fn create_token_metadata(
        &self,
        vault: String,
        name: String,
        symbol: String,
        uri: String,
    ) -> Result<()> {
        let keypair = self
            .cli_config
            .keypair
            .as_ref()
            .ok_or_else(|| anyhow!("Keypair not provided"))?;
        let vault_pubkey = Pubkey::from_str(&vault)?;

        let rpc_client = self.get_rpc_client();
        let vault_account = rpc_client.get_account(&vault_pubkey).await?;
        let vault = Vault::try_from_slice_unchecked(&vault_account.data)?;

        let metadata = Pubkey::find_program_address(
            &[
                b"metadata",
                inline_mpl_token_metadata::id().as_ref(),
                vault.vrt_mint.as_ref(),
            ],
            &inline_mpl_token_metadata::id(),
        )
        .0;

        let ix = CreateTokenMetadataBuilder::new()
            .vault(vault_pubkey)
            .admin(keypair.pubkey())
            .vrt_mint(vault.vrt_mint)
            .payer(keypair.pubkey())
            .metadata(metadata)
            .name(name)
            .symbol(symbol)
            .uri(uri)
            .instruction();

        let recent_blockhash = rpc_client.get_latest_blockhash().await?;
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&keypair.pubkey()),
            &[keypair],
            recent_blockhash,
        );

        info!(
            "Creating token metadata transaction: {:?}",
            tx.get_signature()
        );
        rpc_client
            .send_and_confirm_transaction(&tx)
            .await
            .map_err(|e| anyhow!(e.to_string()))?;
        info!("Transaction confirmed: {:?}", tx.get_signature());

        Ok(())
    }
}
