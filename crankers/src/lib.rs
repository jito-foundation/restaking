use std::{collections::HashMap, path::PathBuf, sync::Arc};

use error::JitoVaultCrankerError;
use jito_bytemuck::AccountDeserialize;
use jito_vault_core::{vault::Vault, vault_operator_delegation::VaultOperatorDelegation};
use solana_rpc_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{pubkey::Pubkey, signature::read_keypair_file};
use vault_handler::VaultHandler;
pub mod core;
pub mod error;
pub mod metrics;
pub mod restaking_handler;
pub mod vault_handler;

pub struct JitoVaultCranker {
    /// RPC client
    rpc_client: Arc<RpcClient>,

    /// Keypair path
    keypair_path: PathBuf,

    /// Vault Program ID
    vault_program_id: Pubkey,

    /// Priority Fees
    priority_fees: u64,
}

impl JitoVaultCranker {
    pub fn new(
        rpc_client: Arc<RpcClient>,
        keypair_path: PathBuf,
        vault_program_id: Pubkey,
        priority_fees: u64,
    ) -> Self {
        Self {
            rpc_client,
            keypair_path,
            vault_program_id,
            priority_fees,
        }
    }

    /// Performs a single vault update cycle, checking and updating all vaults that need updating.
    ///
    /// This method executes the complete vault update workflow:
    /// 1. Determines the current epoch
    /// 2. Identifies vaults that require updates for the current epoch
    /// 3. Groups vault operator delegations by vault
    /// 4. Spawns concurrent tasks to update each vault
    pub async fn update_vaults_once(&self) -> Result<(), JitoVaultCrankerError> {
        let keypair = read_keypair_file(&self.keypair_path)
            .map_err(|e| JitoVaultCrankerError::KeypairRead(e.to_string()))?;
        let payer = Arc::new(keypair);

        let config_address =
            jito_vault_core::config::Config::find_program_address(&self.vault_program_id).0;

        let account = self.rpc_client.get_account(&config_address).await?;
        let config = jito_vault_core::config::Config::try_from_slice_unchecked(&account.data)
            .map_err(|e| JitoVaultCrankerError::Deserialization {
                account_type: "Config".to_string(),
                pubkey: config_address.to_string(),
                src: e.to_string(),
            })?;

        let vault_handler = Arc::new(VaultHandler::new(
            self.rpc_client.clone(),
            self.vault_program_id,
            config_address,
            self.priority_fees,
        ));

        let slot = self.rpc_client.get_slot().await?;
        let epoch = vault_handler.get_epoch(slot, config.epoch_length())?;

        log::info!("Checking for vaults to update. Slot: {slot}, Current Epoch: {epoch}");

        let vaults = vault_handler.get_vaults().await?;
        let delegations = vault_handler.get_vault_operator_delegations().await?;

        let vaults_need_update: Vec<(Pubkey, Vault)> = vaults
            .into_iter()
            .filter(|(_pubkey, vault)| {
                vault
                    .is_update_needed(slot, config.epoch_length())
                    .expect("Config epoch length is 0")
            })
            .collect();

        // All delegations are passed along. Delegation filtering logic is handled in `VaultHandler::crank`
        let mut grouped_delegations: HashMap<Pubkey, Vec<(Pubkey, VaultOperatorDelegation)>> =
            HashMap::from_iter(vaults_need_update.iter().map(|(vault, _)| (*vault, vec![])));
        for (pubkey, delegation) in delegations {
            if vaults_need_update
                .iter()
                .any(|(vault_pubkey, _)| *vault_pubkey == delegation.vault)
            {
                grouped_delegations
                    .entry(delegation.vault)
                    .or_default()
                    .push((pubkey, delegation));
            }
        }

        log::info!("Updating {} vaults", vaults_need_update.len());

        let tasks: Vec<_> = grouped_delegations
            .into_iter()
            .map(|(vault, mut delegations)| {
                // Sort by VaultOperatorDelegation index for correct cranking order
                delegations.sort_by_key(|(_pubkey, delegation)| delegation.index());
                let operators: Vec<Pubkey> = delegations
                    .iter()
                    .map(|(_pubkey, delegation)| delegation.operator)
                    .collect();

                // Spawn each vault update as a separate task
                tokio::spawn({
                    let vault_handler = vault_handler.clone();
                    let payer = payer.clone();

                    async move {
                        match vault_handler
                            .do_vault_update(&payer, epoch, &vault, &operators)
                            .await
                        {
                            Ok(_) => {
                                log::info!("Successfully updated vault: {vault}");
                            }
                            Err(e) => {
                                log::error!("Failed to update vault: {vault}, error: {e}");
                            }
                        }
                    }
                })
            })
            .collect();

        for task in tasks {
            if let Err(e) = task.await {
                log::error!("Task failed to complete: {}", e);
            }
        }

        Ok(())
    }
}
