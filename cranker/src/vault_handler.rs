use jito_bytemuck::{AccountDeserialize, Discriminator};
use jito_vault_client::instructions::CrankVaultUpdateStateTrackerBuilder;
use jito_vault_core::{
    vault::Vault, vault_operator_delegation::VaultOperatorDelegation,
    vault_update_state_tracker::VaultUpdateStateTracker,
};
use solana_account_decoder::UiAccountEncoding;
use solana_rpc_client::nonblocking::rpc_client::RpcClient;
use solana_rpc_client_api::{
    config::{RpcAccountInfoConfig, RpcProgramAccountsConfig},
    filter::{Memcmp, MemcmpEncodedBytes, RpcFilterType},
};
use solana_sdk::{
    commitment_config::CommitmentConfig, pubkey::Pubkey, signature::Keypair, signer::Signer,
    transaction::Transaction,
};

pub struct VaultHandler<'a> {
    rpc_url: String,
    payer: &'a Keypair,
    vault_program_id: Pubkey,
    config_address: Pubkey,
}

impl<'a> VaultHandler<'a> {
    pub fn new(
        rpc_url: &str,
        payer: &'a Keypair,
        vault_program_id: Pubkey,
        config_address: Pubkey,
    ) -> Self {
        Self {
            rpc_url: rpc_url.to_string(),
            payer,
            vault_program_id,
            config_address,
        }
    }

    /// Creates a new `RpcClient` instance with the specified commitment level.
    ///
    /// # Returns
    ///
    /// An `RpcClient` instance configured to use the stored `rpc_url` and the
    /// `confirmed` commitment level for interactions with the Solana blockchain.
    fn get_rpc_client(&self) -> RpcClient {
        RpcClient::new_with_commitment(self.rpc_url.clone(), CommitmentConfig::confirmed())
    }

    /// Retrieves all existing vaults
    ///
    /// # Returns
    ///
    /// Returns an `anyhow::Result` containing a vector of `(Pubkey, Vault)` tuples
    /// representing all the vault accounts associated with the program. Each tuple
    /// consists of:
    /// - `Pubkey`: The public key of the vault account.
    /// - `Vault`: The deserialized vault data from the account.
    pub async fn get_vaults(&self) -> anyhow::Result<Vec<(Pubkey, Vault)>> {
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
                        ..RpcAccountInfoConfig::default()
                    },
                    ..RpcProgramAccountsConfig::default()
                },
            )
            .await?;

        let vaults: Vec<(Pubkey, Vault)> = accounts
            .into_iter()
            .filter_map(|(pubkey, acc)| {
                Vault::try_from_slice_unchecked(&acc.data).map_or(None, |v| Some((pubkey, *v)))
            })
            .collect();

        Ok(vaults)
    }

    /// Retrieves all existing `VaultOperatorDelegation` accounts associated with the program.
    ///
    /// # Returns
    ///
    /// An `anyhow::Result` containing a vector of `(Pubkey, VaultOperatorDelegation)` tuples. Each
    /// tuple represents a vault operator delegation account and includes:
    /// - `Pubkey`: The public key of the vault operator delegation account.
    /// - `VaultOperatorDelegation`: The deserialized vault operator delegation data.
    pub async fn get_vault_operator_delegation(
        &self,
    ) -> anyhow::Result<Vec<(Pubkey, VaultOperatorDelegation)>> {
        let rpc_client = self.get_rpc_client();
        let accounts = rpc_client
            .get_program_accounts_with_config(
                &self.vault_program_id,
                RpcProgramAccountsConfig {
                    filters: Some(vec![RpcFilterType::Memcmp(Memcmp::new(
                        0,
                        MemcmpEncodedBytes::Bytes(vec![VaultOperatorDelegation::DISCRIMINATOR]),
                    ))]),
                    account_config: RpcAccountInfoConfig {
                        encoding: Some(UiAccountEncoding::Base64),
                        ..RpcAccountInfoConfig::default()
                    },
                    ..RpcProgramAccountsConfig::default()
                },
            )
            .await?;

        let delegations: Vec<(Pubkey, VaultOperatorDelegation)> = accounts
            .into_iter()
            .filter_map(|(pubkey, acc)| {
                VaultOperatorDelegation::try_from_slice_unchecked(&acc.data)
                    .map_or(None, |v| Some((pubkey, *v)))
            })
            .collect();

        Ok(delegations)
    }

    /// Cranks the [`VaultUpdateStateTracker`] for a specific epoch and list of operators.
    ///
    /// # Returns
    ///
    /// This method returns an `anyhow::Result<()>` that indicates whether the crank operation
    /// was successful or not.
    pub async fn crank(
        &self,
        epoch: u64,
        vault: &Pubkey,
        operators: &[Pubkey],
    ) -> anyhow::Result<()> {
        let rpc_client = self.get_rpc_client();

        for operator in operators {
            let vault_operator_delegation = VaultOperatorDelegation::find_program_address(
                &self.vault_program_id,
                vault,
                operator,
            )
            .0;

            let tracker_pubkey =
                VaultUpdateStateTracker::find_program_address(&self.vault_program_id, vault, epoch)
                    .0;

            log::info!(
                "Crank Vault Operator Delegation: {}, Vault Update State Tracker: {}",
                vault_operator_delegation,
                tracker_pubkey
            );

            let mut ix_builder = CrankVaultUpdateStateTrackerBuilder::new();
            ix_builder
                .config(self.config_address)
                .vault(*vault)
                .operator(*operator)
                .vault_operator_delegation(vault_operator_delegation)
                .vault_update_state_tracker(tracker_pubkey);
            let mut ix = ix_builder.instruction();
            ix.program_id = self.vault_program_id;

            let blockhash = match rpc_client.get_latest_blockhash().await {
                Ok(bh) => bh,
                Err(e) => {
                    let context = format!("Failed to get latest blockhash: {e}");
                    return Err(anyhow::Error::new(e).context(context));
                }
            };
            let tx = Transaction::new_signed_with_payer(
                &[ix],
                Some(&self.payer.pubkey()),
                &[&self.payer],
                blockhash,
            );

            match rpc_client.send_and_confirm_transaction(&tx).await {
                Ok(sig) => {
                    log::info!("Transaction confirmed: {:?}", sig);
                }
                Err(e) => {
                    let context = format!("Failed to send transaction: {:?}", e);
                    return Err(anyhow::Error::new(e).context(context));
                }
            }
        }

        Ok(())
    }
}
