use std::str::FromStr;

use anyhow::{anyhow, Result};
use jito_bytemuck::{AccountDeserialize, Discriminator};
use jito_restaking_client::{
    instructions::{
        CooldownNcnVaultTicketBuilder, CooldownOperatorVaultTicketBuilder, InitializeConfigBuilder,
        InitializeNcnBuilder, InitializeNcnOperatorStateBuilder, InitializeNcnVaultTicketBuilder,
        InitializeOperatorBuilder, InitializeOperatorVaultTicketBuilder,
        NcnCooldownOperatorBuilder, NcnDelegateTokenAccountBuilder, NcnWarmupOperatorBuilder,
        OperatorCooldownNcnBuilder, OperatorDelegateTokenAccountBuilder, OperatorSetFeeBuilder,
        OperatorSetSecondaryAdminBuilder, OperatorWarmupNcnBuilder, SetConfigAdminBuilder,
        WarmupNcnVaultTicketBuilder, WarmupOperatorVaultTicketBuilder,
    },
    types::OperatorAdminRole,
};
use jito_restaking_core::{
    config::Config, ncn::Ncn, ncn_operator_state::NcnOperatorState,
    ncn_vault_ticket::NcnVaultTicket, operator::Operator,
    operator_vault_ticket::OperatorVaultTicket,
};
use log::{debug, info};
use solana_account_decoder::UiAccountEncoding;
use solana_program::pubkey::Pubkey;
use solana_rpc_client::{nonblocking::rpc_client::RpcClient, rpc_client::SerializableTransaction};
use solana_rpc_client_api::{
    config::{RpcAccountInfoConfig, RpcProgramAccountsConfig},
    filter::{Memcmp, MemcmpEncodedBytes, RpcFilterType},
};
use solana_sdk::{
    signature::{read_keypair_file, Keypair, Signer},
    transaction::Transaction,
};
use spl_associated_token_account::{
    get_associated_token_address, instruction::create_associated_token_account_idempotent,
};

use crate::{
    restaking::{ConfigActions, NcnActions, OperatorActions, RestakingCommands},
    CliConfig,
};

pub struct RestakingCliHandler {
    cli_config: CliConfig,
    restaking_program_id: Pubkey,
    vault_program_id: Pubkey,
}

impl RestakingCliHandler {
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

    fn get_rpc_client(&self) -> RpcClient {
        RpcClient::new_with_commitment(self.cli_config.rpc_url.clone(), self.cli_config.commitment)
    }

    pub async fn handle(&self, action: RestakingCommands) -> Result<()> {
        match action {
            RestakingCommands::Config {
                action: ConfigActions::Initialize,
            } => self.initialize_config().await,
            RestakingCommands::Config {
                action: ConfigActions::Get,
            } => self.get_config().await,
            RestakingCommands::Config {
                action: ConfigActions::SetAdmin { new_admin },
            } => self.set_config_admin(new_admin).await,
            RestakingCommands::Ncn {
                action:
                    NcnActions::Initialize {
                        path_to_base_keypair,
                    },
            } => self.initialize_ncn(path_to_base_keypair).await,
            RestakingCommands::Ncn {
                action: NcnActions::InitializeNcnOperatorState { ncn, operator },
            } => self.initialize_ncn_operator_state(ncn, operator).await,
            RestakingCommands::Ncn {
                action: NcnActions::NcnWarmupOperator { ncn, operator },
            } => self.ncn_warmup_operator(ncn, operator).await,
            RestakingCommands::Ncn {
                action: NcnActions::NcnCooldownOperator { ncn, operator },
            } => self.ncn_cooldown_operator(ncn, operator).await,
            RestakingCommands::Ncn {
                action: NcnActions::InitializeNcnVaultTicket { ncn, vault },
            } => self.initialize_ncn_vault_ticket(ncn, vault).await,
            RestakingCommands::Ncn {
                action: NcnActions::WarmupNcnVaultTicket { ncn, vault },
            } => self.warmup_ncn_vault_ticket(ncn, vault).await,
            RestakingCommands::Ncn {
                action: NcnActions::CooldownNcnVaultTicket { ncn, vault },
            } => self.cooldown_ncn_vault_ticket(ncn, vault).await,
            RestakingCommands::Ncn {
                action:
                    NcnActions::NcnDelegateTokenAccount {
                        ncn,
                        delegate,
                        token_mint,
                        should_create_token_account,
                    },
            } => {
                self.ncn_delegate_token_account(
                    ncn,
                    delegate,
                    token_mint,
                    should_create_token_account,
                )
                .await
            }
            RestakingCommands::Ncn {
                action: NcnActions::Get { pubkey },
            } => self.get_ncn(pubkey).await,
            RestakingCommands::Ncn {
                action: NcnActions::List,
            } => self.list_ncn().await,
            RestakingCommands::Operator {
                action: OperatorActions::Initialize { operator_fee_bps },
            } => self.initialize_operator(operator_fee_bps).await,
            RestakingCommands::Operator {
                action: OperatorActions::InitializeOperatorVaultTicket { operator, vault },
            } => self.initialize_operator_vault_ticket(operator, vault).await,
            RestakingCommands::Operator {
                action: OperatorActions::WarmupOperatorVaultTicket { operator, vault },
            } => self.warmup_operator_vault_ticket(operator, vault).await,
            RestakingCommands::Operator {
                action: OperatorActions::CooldownOperatorVaultTicket { operator, vault },
            } => self.cooldown_operator_vault_ticket(operator, vault).await,
            RestakingCommands::Operator {
                action: OperatorActions::OperatorWarmupNcn { operator, ncn },
            } => self.operator_warmup_ncn(operator, ncn).await,
            RestakingCommands::Operator {
                action: OperatorActions::OperatorCooldownNcn { operator, ncn },
            } => self.operator_cooldown_ncn(operator, ncn).await,
            RestakingCommands::Operator {
                action:
                    OperatorActions::OperatorSetSecondaryAdmin {
                        operator,
                        new_admin,
                        set_ncn_admin,
                        set_vault_admin,
                        set_voter_admin,
                        set_delegate_admin,
                        set_metadata_admin,
                    },
            } => {
                self.operator_set_secondary_admin(
                    operator,
                    new_admin,
                    set_ncn_admin,
                    set_vault_admin,
                    set_voter_admin,
                    set_delegate_admin,
                    set_metadata_admin,
                )
                .await
            }
            RestakingCommands::Operator {
                action:
                    OperatorActions::OperatorSetFees {
                        operator,
                        operator_fee_bps,
                    },
            } => self.operator_set_fee(operator, operator_fee_bps).await,
            RestakingCommands::Operator {
                action:
                    OperatorActions::OperatorDelegateTokenAccount {
                        operator,
                        delegate,
                        token_mint,
                        should_create_token_account,
                    },
            } => {
                self.operator_delegate_token_account(
                    operator,
                    delegate,
                    token_mint,
                    should_create_token_account,
                )
                .await
            }
            RestakingCommands::Operator {
                action: OperatorActions::Get { pubkey },
            } => self.get_operator(pubkey).await,
            RestakingCommands::Operator {
                action: OperatorActions::List,
            } => self.list_operator().await,
        }
    }

    pub async fn operator_set_fee(&self, operator: String, operator_fee_bps: u16) -> Result<()> {
        let keypair = self
            .cli_config
            .keypair
            .as_ref()
            .ok_or_else(|| anyhow!("No keypair"))?;
        let rpc_client = self.get_rpc_client();

        let (restaking_vault_config, _, _) =
            Config::find_program_address(&self.restaking_program_id);

        let operator = Pubkey::from_str(&operator)?;

        let mut ix_builder = OperatorSetFeeBuilder::new();
        ix_builder
            .operator(operator)
            .new_fee_bps(operator_fee_bps)
            .admin(keypair.pubkey())
            .config(restaking_vault_config)
            .instruction();

        let blockhash = rpc_client.get_latest_blockhash().await?;
        let tx = Transaction::new_signed_with_payer(
            &[ix_builder.instruction()],
            Some(&keypair.pubkey()),
            &[keypair],
            blockhash,
        );
        info!(
            "Setting fees to {:?} to Operator {}",
            operator_fee_bps, operator,
        );
        let result = rpc_client.send_and_confirm_transaction(&tx).await?;
        info!("Transaction confirmed: {:?}", result);

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn operator_set_secondary_admin(
        &self,
        operator: String,
        new_admin: String,
        set_ncn_admin: bool,
        set_vault_admin: bool,
        set_voter_admin: bool,
        set_delegate_admin: bool,
        set_metadata_admin: bool,
    ) -> Result<()> {
        let keypair = self
            .cli_config
            .keypair
            .as_ref()
            .ok_or_else(|| anyhow!("No keypair"))?;
        let rpc_client = self.get_rpc_client();

        let operator = Pubkey::from_str(&operator)?;
        let new_admin = Pubkey::from_str(&new_admin)?;

        let mut roles: Vec<OperatorAdminRole> = vec![];
        if set_ncn_admin {
            roles.push(OperatorAdminRole::NcnAdmin);
        }
        if set_vault_admin {
            roles.push(OperatorAdminRole::VaultAdmin);
        }
        if set_voter_admin {
            roles.push(OperatorAdminRole::VoterAdmin);
        }
        if set_delegate_admin {
            roles.push(OperatorAdminRole::DelegateAdmin);
        }
        if set_metadata_admin {
            roles.push(OperatorAdminRole::MetadataAdmin);
        }

        for role in roles.iter() {
            let mut ix_builder = OperatorSetSecondaryAdminBuilder::new();
            ix_builder
                .new_admin(new_admin)
                .operator(operator)
                .admin(keypair.pubkey())
                .operator_admin_role(*role)
                .instruction();

            let blockhash = rpc_client.get_latest_blockhash().await?;
            let tx = Transaction::new_signed_with_payer(
                &[ix_builder.instruction()],
                Some(&keypair.pubkey()),
                &[keypair],
                blockhash,
            );
            info!(
                "Setting {:?} Admin to {} for Operator {}",
                role, new_admin, operator
            );
            let result = rpc_client.send_and_confirm_transaction(&tx).await?;
            info!("Transaction confirmed: {:?}", result);
        }

        Ok(())
    }

    pub async fn operator_delegate_token_account(
        &self,
        operator: String,
        delegate: String,
        token_mint: String,
        should_create_token_account: bool,
    ) -> Result<()> {
        let keypair = self
            .cli_config
            .keypair
            .as_ref()
            .ok_or_else(|| anyhow!("No keypair"))?;
        let rpc_client = self.get_rpc_client();

        let operator = Pubkey::from_str(&operator)?;
        let delegate = Pubkey::from_str(&delegate)?;
        let token_mint = Pubkey::from_str(&token_mint)?;

        let token_account = get_associated_token_address(&operator, &token_mint);

        let mut ixs = vec![];

        if should_create_token_account {
            let ix = create_associated_token_account_idempotent(
                &keypair.pubkey(),
                &operator,
                &token_mint,
                &spl_token::id(),
            );
            ixs.push(ix);
        }

        let mut ix_builder = OperatorDelegateTokenAccountBuilder::new();
        ix_builder
            .operator(operator)
            .delegate(delegate)
            .delegate_admin(keypair.pubkey())
            .token_account(token_account)
            .token_mint(token_mint);

        ixs.push(ix_builder.instruction());

        let blockhash = rpc_client.get_latest_blockhash().await?;
        let tx = Transaction::new_signed_with_payer(
            &ixs,
            Some(&keypair.pubkey()),
            &[keypair],
            blockhash,
        );
        info!("Setting delegate for mint: {} to {}", token_mint, delegate,);
        let result = rpc_client.send_and_confirm_transaction(&tx).await?;
        info!("Transaction confirmed: {:?}", result);

        Ok(())
    }

    pub async fn ncn_delegate_token_account(
        &self,
        ncn: String,
        delegate: String,
        token_mint: String,
        should_create_token_account: bool,
    ) -> Result<()> {
        let keypair = self
            .cli_config
            .keypair
            .as_ref()
            .ok_or_else(|| anyhow!("No keypair"))?;
        let rpc_client = self.get_rpc_client();

        let ncn = Pubkey::from_str(&ncn)?;
        let delegate = Pubkey::from_str(&delegate)?;
        let token_mint = Pubkey::from_str(&token_mint)?;

        let token_account = get_associated_token_address(&ncn, &token_mint);

        let mut ixs = vec![];

        if should_create_token_account {
            let ix = create_associated_token_account_idempotent(
                &keypair.pubkey(),
                &ncn,
                &token_mint,
                &spl_token::id(),
            );
            ixs.push(ix);
        }

        let mut ix_builder = NcnDelegateTokenAccountBuilder::new();
        ix_builder
            .ncn(ncn)
            .delegate(delegate)
            .delegate_admin(keypair.pubkey())
            .token_account(token_account)
            .token_mint(token_mint);

        ixs.push(ix_builder.instruction());

        let blockhash = rpc_client.get_latest_blockhash().await?;
        let tx = Transaction::new_signed_with_payer(
            &ixs,
            Some(&keypair.pubkey()),
            &[keypair],
            blockhash,
        );
        info!("Setting delegate for mint: {} to {}", token_mint, delegate,);
        let result = rpc_client.send_and_confirm_transaction(&tx).await?;
        info!("Transaction confirmed: {:?}", result);

        Ok(())
    }

    pub async fn initialize_ncn_vault_ticket(&self, ncn: String, vault: String) -> Result<()> {
        let keypair = self
            .cli_config
            .keypair
            .as_ref()
            .ok_or_else(|| anyhow!("No keypair"))?;
        let rpc_client = self.get_rpc_client();

        let ncn = Pubkey::from_str(&ncn)?;
        let vault = Pubkey::from_str(&vault)?;

        let (ncn_vault_ticket, _, _) =
            NcnVaultTicket::find_program_address(&self.restaking_program_id, &ncn, &vault);

        let mut ix_builder = InitializeNcnVaultTicketBuilder::new();
        ix_builder
            .config(Config::find_program_address(&self.restaking_program_id).0)
            .ncn(ncn)
            .vault(vault)
            .ncn_vault_ticket(ncn_vault_ticket)
            .admin(keypair.pubkey())
            .payer(keypair.pubkey())
            .instruction();

        let blockhash = rpc_client.get_latest_blockhash().await?;
        let tx = Transaction::new_signed_with_payer(
            &[ix_builder.instruction()],
            Some(&keypair.pubkey()),
            &[keypair],
            blockhash,
        );
        info!("Initializing NCN Vault Ticket");
        let result = rpc_client.send_and_confirm_transaction(&tx).await?;
        info!("Transaction confirmed: {:?}", result);

        Ok(())
    }

    pub async fn warmup_ncn_vault_ticket(&self, ncn: String, vault: String) -> Result<()> {
        let keypair = self
            .cli_config
            .keypair
            .as_ref()
            .ok_or_else(|| anyhow!("No keypair"))?;
        let rpc_client = self.get_rpc_client();

        let ncn = Pubkey::from_str(&ncn)?;
        let vault = Pubkey::from_str(&vault)?;

        let (ncn_vault_ticket, _, _) =
            NcnVaultTicket::find_program_address(&self.restaking_program_id, &ncn, &vault);

        let mut ix_builder = WarmupNcnVaultTicketBuilder::new();
        ix_builder
            .config(Config::find_program_address(&self.restaking_program_id).0)
            .ncn(ncn)
            .vault(vault)
            .ncn_vault_ticket(ncn_vault_ticket)
            .admin(keypair.pubkey())
            .instruction();

        let blockhash = rpc_client.get_latest_blockhash().await?;
        let tx = Transaction::new_signed_with_payer(
            &[ix_builder.instruction()],
            Some(&keypair.pubkey()),
            &[keypair],
            blockhash,
        );
        info!("Warmup NCN Vault Ticket");
        let result = rpc_client.send_and_confirm_transaction(&tx).await?;
        info!("Transaction confirmed: {:?}", result);

        Ok(())
    }

    pub async fn cooldown_ncn_vault_ticket(&self, ncn: String, vault: String) -> Result<()> {
        let keypair = self
            .cli_config
            .keypair
            .as_ref()
            .ok_or_else(|| anyhow!("No keypair"))?;
        let rpc_client = self.get_rpc_client();

        let ncn = Pubkey::from_str(&ncn)?;
        let vault = Pubkey::from_str(&vault)?;

        let (ncn_vault_ticket, _, _) =
            NcnVaultTicket::find_program_address(&self.restaking_program_id, &ncn, &vault);

        let mut ix_builder = CooldownNcnVaultTicketBuilder::new();
        ix_builder
            .config(Config::find_program_address(&self.restaking_program_id).0)
            .ncn(ncn)
            .vault(vault)
            .ncn_vault_ticket(ncn_vault_ticket)
            .admin(keypair.pubkey())
            .instruction();

        let blockhash = rpc_client.get_latest_blockhash().await?;
        let tx = Transaction::new_signed_with_payer(
            &[ix_builder.instruction()],
            Some(&keypair.pubkey()),
            &[keypair],
            blockhash,
        );
        info!("Cooldown NCN Vault Ticket");
        let result = rpc_client.send_and_confirm_transaction(&tx).await?;
        info!("Transaction confirmed: {:?}", result);

        Ok(())
    }

    pub async fn initialize_ncn_operator_state(&self, ncn: String, operator: String) -> Result<()> {
        let keypair = self
            .cli_config
            .keypair
            .as_ref()
            .ok_or_else(|| anyhow!("No keypair"))?;
        let rpc_client = self.get_rpc_client();

        let ncn = Pubkey::from_str(&ncn)?;
        let operator = Pubkey::from_str(&operator)?;

        let (ncn_operator_state, _, _) =
            NcnOperatorState::find_program_address(&self.restaking_program_id, &ncn, &operator);

        let mut ix_builder = InitializeNcnOperatorStateBuilder::new();
        ix_builder
            .config(Config::find_program_address(&self.restaking_program_id).0)
            .ncn(ncn)
            .operator(operator)
            .ncn_operator_state(ncn_operator_state)
            .admin(keypair.pubkey())
            .payer(keypair.pubkey())
            .instruction();

        let blockhash = rpc_client.get_latest_blockhash().await?;
        let tx = Transaction::new_signed_with_payer(
            &[ix_builder.instruction()],
            Some(&keypair.pubkey()),
            &[keypair],
            blockhash,
        );
        info!("Initializing NCN Operator State");
        let result = rpc_client.send_and_confirm_transaction(&tx).await?;
        info!("Transaction confirmed: {:?}", result);

        Ok(())
    }

    pub async fn ncn_warmup_operator(&self, ncn: String, operator: String) -> Result<()> {
        let keypair = self
            .cli_config
            .keypair
            .as_ref()
            .ok_or_else(|| anyhow!("No keypair"))?;
        let rpc_client = self.get_rpc_client();

        let ncn = Pubkey::from_str(&ncn)?;
        let operator = Pubkey::from_str(&operator)?;

        let (ncn_operator_state, _, _) =
            NcnOperatorState::find_program_address(&self.restaking_program_id, &ncn, &operator);

        let mut ix_builder = NcnWarmupOperatorBuilder::new();
        ix_builder
            .config(Config::find_program_address(&self.restaking_program_id).0)
            .ncn(ncn)
            .operator(operator)
            .ncn_operator_state(ncn_operator_state)
            .admin(keypair.pubkey())
            .instruction();

        let blockhash = rpc_client.get_latest_blockhash().await?;
        let tx = Transaction::new_signed_with_payer(
            &[ix_builder.instruction()],
            Some(&keypair.pubkey()),
            &[keypair],
            blockhash,
        );
        info!("NCN Warmup Operator");
        let result = rpc_client.send_and_confirm_transaction(&tx).await?;
        info!("Transaction confirmed: {:?}", result);

        Ok(())
    }

    pub async fn ncn_cooldown_operator(&self, ncn: String, operator: String) -> Result<()> {
        let keypair = self
            .cli_config
            .keypair
            .as_ref()
            .ok_or_else(|| anyhow!("No keypair"))?;
        let rpc_client = self.get_rpc_client();

        let ncn = Pubkey::from_str(&ncn)?;
        let operator = Pubkey::from_str(&operator)?;

        let (ncn_operator_state, _, _) =
            NcnOperatorState::find_program_address(&self.restaking_program_id, &ncn, &operator);

        let mut ix_builder = NcnCooldownOperatorBuilder::new();
        ix_builder
            .config(Config::find_program_address(&self.restaking_program_id).0)
            .ncn(ncn)
            .operator(operator)
            .ncn_operator_state(ncn_operator_state)
            .admin(keypair.pubkey())
            .instruction();

        let blockhash = rpc_client.get_latest_blockhash().await?;
        let tx = Transaction::new_signed_with_payer(
            &[ix_builder.instruction()],
            Some(&keypair.pubkey()),
            &[keypair],
            blockhash,
        );
        info!("NCN Cooldown Operator");
        let result = rpc_client.send_and_confirm_transaction(&tx).await?;
        info!("Transaction confirmed: {:?}", result);

        Ok(())
    }

    pub async fn operator_warmup_ncn(&self, operator: String, ncn: String) -> Result<()> {
        let keypair = self
            .cli_config
            .keypair
            .as_ref()
            .ok_or_else(|| anyhow!("No keypair"))?;
        let rpc_client = self.get_rpc_client();

        let operator = Pubkey::from_str(&operator)?;
        let ncn = Pubkey::from_str(&ncn)?;

        let (ncn_operator_state, _, _) =
            NcnOperatorState::find_program_address(&self.restaking_program_id, &ncn, &operator);

        let mut ix_builder = OperatorWarmupNcnBuilder::new();
        ix_builder
            .config(Config::find_program_address(&self.restaking_program_id).0)
            .ncn(ncn)
            .operator(operator)
            .ncn_operator_state(ncn_operator_state)
            .admin(keypair.pubkey())
            .instruction();

        let blockhash = rpc_client.get_latest_blockhash().await?;
        let tx = Transaction::new_signed_with_payer(
            &[ix_builder.instruction()],
            Some(&keypair.pubkey()),
            &[keypair],
            blockhash,
        );
        info!("Operator Warmup NCN");
        let result = rpc_client.send_and_confirm_transaction(&tx).await?;
        info!("Transaction confirmed: {:?}", result);

        Ok(())
    }

    pub async fn operator_cooldown_ncn(&self, operator: String, ncn: String) -> Result<()> {
        let keypair = self
            .cli_config
            .keypair
            .as_ref()
            .ok_or_else(|| anyhow!("No keypair"))?;
        let rpc_client = self.get_rpc_client();

        let operator = Pubkey::from_str(&operator)?;
        let ncn = Pubkey::from_str(&ncn)?;

        let (ncn_operator_state, _, _) =
            NcnOperatorState::find_program_address(&self.restaking_program_id, &ncn, &operator);

        let mut ix_builder = OperatorCooldownNcnBuilder::new();
        ix_builder
            .config(Config::find_program_address(&self.restaking_program_id).0)
            .ncn(ncn)
            .operator(operator)
            .ncn_operator_state(ncn_operator_state)
            .admin(keypair.pubkey())
            .instruction();

        let blockhash = rpc_client.get_latest_blockhash().await?;
        let tx = Transaction::new_signed_with_payer(
            &[ix_builder.instruction()],
            Some(&keypair.pubkey()),
            &[keypair],
            blockhash,
        );
        info!("Operator Cooldown NCN");
        let result = rpc_client.send_and_confirm_transaction(&tx).await?;
        info!("Transaction confirmed: {:?}", result);

        Ok(())
    }

    async fn initialize_config(&self) -> Result<()> {
        let keypair = self
            .cli_config
            .keypair
            .as_ref()
            .ok_or_else(|| anyhow!("No keypair"))?;
        let rpc_client = self.get_rpc_client();

        let config_address = Config::find_program_address(&self.restaking_program_id).0;
        let mut ix_builder = InitializeConfigBuilder::new();
        ix_builder
            .config(config_address)
            .admin(keypair.pubkey())
            .vault_program(self.vault_program_id);
        let blockhash = rpc_client.get_latest_blockhash().await?;
        let tx = Transaction::new_signed_with_payer(
            &[ix_builder.instruction()],
            Some(&keypair.pubkey()),
            &[keypair],
            blockhash,
        );
        info!("Initializing restaking config parameters: {:?}", ix_builder);
        info!(
            "Initializing restaking config transaction: {:?}",
            tx.get_signature()
        );
        rpc_client.send_and_confirm_transaction(&tx).await?;
        info!("Transaction confirmed: {:?}", tx.get_signature());
        Ok(())
    }

    pub async fn initialize_ncn(&self, path_to_base_keypair: Option<String>) -> Result<()> {
        let keypair = self
            .cli_config
            .keypair
            .as_ref()
            .ok_or_else(|| anyhow!("No keypair"))?;
        let rpc_client = self.get_rpc_client();

        let base =
            path_to_base_keypair.map_or_else(Keypair::new, |path| read_keypair_file(path).unwrap());
        let ncn = Ncn::find_program_address(&self.restaking_program_id, &base.pubkey()).0;

        let mut ix_builder = InitializeNcnBuilder::new();
        ix_builder
            .config(Config::find_program_address(&self.restaking_program_id).0)
            .ncn(ncn)
            .admin(keypair.pubkey())
            .base(base.pubkey())
            .instruction();

        let blockhash = rpc_client.get_latest_blockhash().await?;
        let tx = Transaction::new_signed_with_payer(
            &[ix_builder.instruction()],
            Some(&keypair.pubkey()),
            &[keypair, &base],
            blockhash,
        );
        info!("Initializing NCN: {:?}", ncn);
        info!("Initializing NCN transaction: {:?}", tx.get_signature());
        let result = rpc_client.send_and_confirm_transaction(&tx).await?;
        info!("Transaction confirmed: {:?}", result);
        let statuses = rpc_client
            .get_signature_statuses(&[*tx.get_signature()])
            .await?;

        let tx_status = statuses
            .value
            .first()
            .unwrap()
            .as_ref()
            .ok_or_else(|| anyhow!("No signature status"))?;
        info!("Transaction status: {:?}", tx_status);
        info!("NCN initialized at address: {:?}", ncn);

        Ok(())
    }

    pub async fn initialize_operator(&self, operator_fee_bps: u16) -> Result<()> {
        let keypair = self
            .cli_config
            .keypair
            .as_ref()
            .ok_or_else(|| anyhow!("No keypair"))?;
        let rpc_client = self.get_rpc_client();

        let base = Keypair::new();
        let operator = Operator::find_program_address(&self.restaking_program_id, &base.pubkey()).0;

        let mut ix_builder = InitializeOperatorBuilder::new();
        ix_builder
            .config(Config::find_program_address(&self.restaking_program_id).0)
            .operator(operator)
            .admin(keypair.pubkey())
            .base(base.pubkey())
            .operator_fee_bps(operator_fee_bps)
            .instruction();

        let blockhash = rpc_client.get_latest_blockhash().await?;
        let tx = Transaction::new_signed_with_payer(
            &[ix_builder.instruction()],
            Some(&keypair.pubkey()),
            &[keypair, &base],
            blockhash,
        );
        info!("Initializing operator: {:?}", operator);
        info!(
            "Initializing operator transaction: {:?}",
            tx.get_signature()
        );
        rpc_client.send_and_confirm_transaction(&tx).await?;
        info!("Transaction confirmed");
        let statuses = rpc_client
            .get_signature_statuses(&[*tx.get_signature()])
            .await?;

        let tx_status = statuses
            .value
            .first()
            .unwrap()
            .as_ref()
            .ok_or_else(|| anyhow!("No signature status"))?;
        info!("Transaction status: {:?}", tx_status);
        info!("Operator initialized at address: {:?}", operator);

        Ok(())
    }

    pub async fn initialize_operator_vault_ticket(
        &self,
        operator: String,
        vault: String,
    ) -> Result<()> {
        let keypair = self
            .cli_config
            .keypair
            .as_ref()
            .ok_or_else(|| anyhow!("Keypair not provided"))?;
        let rpc_client = self.get_rpc_client();

        let operator = Pubkey::from_str(&operator)?;
        let vault = Pubkey::from_str(&vault)?;

        let operator_vault_ticket = OperatorVaultTicket::find_program_address(
            &self.restaking_program_id,
            &operator,
            &vault,
        )
        .0;

        let mut ix_builder = InitializeOperatorVaultTicketBuilder::new();
        ix_builder
            .config(Config::find_program_address(&self.restaking_program_id).0)
            .operator(operator)
            .vault(vault)
            .admin(keypair.pubkey())
            .operator_vault_ticket(operator_vault_ticket)
            .payer(keypair.pubkey());

        let blockhash = rpc_client.get_latest_blockhash().await?;
        let tx = Transaction::new_signed_with_payer(
            &[ix_builder.instruction()],
            Some(&keypair.pubkey()),
            &[keypair],
            blockhash,
        );

        info!(
            "Initializing operator vault ticket transaction: {:?}",
            tx.get_signature()
        );
        let result = rpc_client.send_and_confirm_transaction(&tx).await?;
        info!("Transaction confirmed: {:?}", result);

        info!("\nCreated Operator Vault Ticket");
        info!("Operator address: {}", operator);
        info!("Vault address: {}", vault);
        info!("Operator Vault Ticket address: {}", operator_vault_ticket);

        Ok(())
    }

    pub async fn warmup_operator_vault_ticket(
        &self,
        operator: String,
        vault: String,
    ) -> Result<()> {
        let keypair = self
            .cli_config
            .keypair
            .as_ref()
            .ok_or_else(|| anyhow!("Keypair not provided"))?;
        let rpc_client = self.get_rpc_client();

        let operator = Pubkey::from_str(&operator)?;
        let vault = Pubkey::from_str(&vault)?;

        let operator_vault_ticket = OperatorVaultTicket::find_program_address(
            &self.restaking_program_id,
            &operator,
            &vault,
        )
        .0;

        let mut ix_builder = WarmupOperatorVaultTicketBuilder::new();
        ix_builder
            .config(Config::find_program_address(&self.restaking_program_id).0)
            .operator(operator)
            .vault(vault)
            .operator_vault_ticket(operator_vault_ticket)
            .admin(keypair.pubkey());

        let blockhash = rpc_client.get_latest_blockhash().await?;
        let tx = Transaction::new_signed_with_payer(
            &[ix_builder.instruction()],
            Some(&keypair.pubkey()),
            &[keypair],
            blockhash,
        );

        info!(
            "Warming up operator vault ticket transaction: {:?}",
            tx.get_signature()
        );
        let result = rpc_client.send_and_confirm_transaction(&tx).await?;
        info!("Transaction confirmed: {:?}", result);

        Ok(())
    }

    pub async fn cooldown_operator_vault_ticket(
        &self,
        operator: String,
        vault: String,
    ) -> Result<()> {
        let keypair = self
            .cli_config
            .keypair
            .as_ref()
            .ok_or_else(|| anyhow!("Keypair not provided"))?;
        let rpc_client = self.get_rpc_client();

        let operator = Pubkey::from_str(&operator)?;
        let vault = Pubkey::from_str(&vault)?;

        let operator_vault_ticket = OperatorVaultTicket::find_program_address(
            &self.restaking_program_id,
            &operator,
            &vault,
        )
        .0;

        let mut ix_builder = CooldownOperatorVaultTicketBuilder::new();
        ix_builder
            .config(Config::find_program_address(&self.restaking_program_id).0)
            .operator(operator)
            .vault(vault)
            .operator_vault_ticket(operator_vault_ticket)
            .admin(keypair.pubkey());

        let blockhash = rpc_client.get_latest_blockhash().await?;
        let tx = Transaction::new_signed_with_payer(
            &[ix_builder.instruction()],
            Some(&keypair.pubkey()),
            &[keypair],
            blockhash,
        );

        info!("Cooldown Operator Vault Ticket");
        let result = rpc_client.send_and_confirm_transaction(&tx).await?;
        info!("Transaction confirmed: {:?}", result);

        Ok(())
    }

    pub async fn get_config(&self) -> Result<()> {
        let rpc_client = self.get_rpc_client();

        let config_address = Config::find_program_address(&self.restaking_program_id).0;
        debug!(
            "Reading the restaking configuration account at address: {}",
            config_address
        );

        let account = rpc_client.get_account(&config_address).await?;
        let config = Config::try_from_slice_unchecked(&account.data)?;
        info!(
            "Restaking config at address {}: {:?}",
            config_address, config
        );
        Ok(())
    }

    pub async fn get_ncn(&self, pubkey: String) -> Result<()> {
        let pubkey = Pubkey::from_str(&pubkey)?;
        let account = self.get_rpc_client().get_account(&pubkey).await?;
        let ncn = Ncn::try_from_slice_unchecked(&account.data)?;
        info!("NCN at address {}: {:?}", pubkey, ncn);
        Ok(())
    }

    pub async fn list_ncn(&self) -> Result<()> {
        let rpc_client = self.get_rpc_client();
        let accounts = rpc_client
            .get_program_accounts_with_config(
                &self.restaking_program_id,
                RpcProgramAccountsConfig {
                    filters: Some(vec![RpcFilterType::Memcmp(Memcmp::new(
                        0,
                        MemcmpEncodedBytes::Bytes(vec![Ncn::DISCRIMINATOR]),
                    ))]),
                    account_config: RpcAccountInfoConfig {
                        encoding: Some(UiAccountEncoding::Base64),
                        data_slice: None,
                        commitment: None,
                        min_context_slot: None,
                    },
                    with_context: None,
                    sort_results: None,
                },
            )
            .await?;
        for (ncn_pubkey, ncn) in accounts {
            let ncn = Ncn::try_from_slice_unchecked(&ncn.data)?;
            info!("NCN at address {}: {:?}", ncn_pubkey, ncn);
        }
        Ok(())
    }

    pub async fn get_operator(&self, pubkey: String) -> Result<()> {
        let pubkey = Pubkey::from_str(&pubkey)?;
        let account = self.get_rpc_client().get_account(&pubkey).await?;
        let operator = Operator::try_from_slice_unchecked(&account.data)?;
        info!("Operator at address {}: {:?}", pubkey, operator);

        Ok(())
    }

    pub async fn list_operator(&self) -> Result<()> {
        let rpc_client = self.get_rpc_client();
        let accounts = rpc_client
            .get_program_accounts_with_config(
                &self.restaking_program_id,
                RpcProgramAccountsConfig {
                    filters: Some(vec![RpcFilterType::Memcmp(Memcmp::new(
                        0,
                        MemcmpEncodedBytes::Bytes(vec![Operator::DISCRIMINATOR]),
                    ))]),
                    account_config: RpcAccountInfoConfig {
                        encoding: Some(UiAccountEncoding::Base64),
                        data_slice: None,
                        commitment: None,
                        min_context_slot: None,
                    },
                    with_context: None,
                    sort_results: None,
                },
            )
            .await?;
        for (operator_pubkey, operator) in accounts {
            let operator = Operator::try_from_slice_unchecked(&operator.data)?;
            info!("Operator at address {}: {:?}", operator_pubkey, operator);
        }
        Ok(())
    }

    async fn set_config_admin(&self, new_admin: Pubkey) -> Result<()> {
        let keypair = self
            .cli_config
            .keypair
            .as_ref()
            .ok_or_else(|| anyhow!("No keypair"))?;
        let rpc_client = self.get_rpc_client();

        let config_address = Config::find_program_address(&self.restaking_program_id).0;
        let mut ix_builder = SetConfigAdminBuilder::new();
        ix_builder
            .config(config_address)
            .old_admin(keypair.pubkey())
            .new_admin(new_admin);

        let blockhash = rpc_client.get_latest_blockhash().await?;
        let tx = Transaction::new_signed_with_payer(
            &[ix_builder.instruction()],
            Some(&keypair.pubkey()),
            &[keypair],
            blockhash,
        );
        info!(
            "Setting restaking config admin parameters: {:?}",
            ix_builder
        );
        info!(
            "Setting restaking config admin transaction: {:?}",
            tx.get_signature()
        );
        rpc_client.send_and_confirm_transaction(&tx).await?;
        info!("Transaction confirmed: {:?}", tx.get_signature());
        Ok(())
    }
}
