use std::{sync::Arc, time::Duration};

use base64::{engine::general_purpose, Engine};
use jito_bytemuck::AccountDeserialize;
use jito_jsm_core::get_epoch;
use jito_vault_client::{
    instructions::{
        CloseVaultUpdateStateTrackerBuilder, CrankVaultUpdateStateTrackerBuilder,
        InitializeVaultUpdateStateTrackerBuilder,
    },
    types::WithdrawalAllocationMethod,
};
use jito_vault_core::{
    vault::Vault, vault_operator_delegation::VaultOperatorDelegation,
    vault_update_state_tracker::VaultUpdateStateTracker,
};
use log::error;
use solana_account_decoder::{UiAccountEncoding, UiDataSliceConfig};
use solana_rpc_client::nonblocking::rpc_client::RpcClient;
use solana_rpc_client_api::{
    config::{RpcAccountInfoConfig, RpcProgramAccountsConfig},
    filter::{Memcmp, MemcmpEncodedBytes, RpcFilterType},
};
use solana_sdk::{
    commitment_config::CommitmentConfig, compute_budget::ComputeBudgetInstruction,
    instruction::Instruction, pubkey::Pubkey, signature::Keypair, signer::Signer,
    transaction::Transaction,
};
use tokio::time::sleep;

use crate::{core::get_latest_blockhash_with_retry, error::JitoVaultCrankerError};

pub struct VaultHandler {
    /// `RpcClient`
    rpc_client: Arc<RpcClient>,

    /// Vault Program ID
    vault_program_id: Pubkey,

    /// Config  Address
    config_address: Pubkey,

    /// Priority Fees
    priority_fees: u64,
}

impl VaultHandler {
    pub fn new(
        rpc_client: Arc<RpcClient>,
        vault_program_id: Pubkey,
        config_address: Pubkey,
        priority_fees: u64,
    ) -> Self {
        Self {
            rpc_client,
            vault_program_id,
            config_address,
            priority_fees,
        }
    }

    pub fn get_epoch(&self, slot: u64, epoch_length: u64) -> Result<u64, JitoVaultCrankerError> {
        get_epoch(slot, epoch_length).map_err(|e| JitoVaultCrankerError::MathError(e.to_string()))
    }

    /// Constructs an `RpcProgramAccountsConfig` for querying accounts of a given type `T`.
    ///
    /// # Returns
    /// - `Ok(RpcProgramAccountsConfig)`: A valid configuration for filtering accounts in
    ///   Solana's RPC API.
    /// - `Err(JitoVaultCrankerError)`: If the data size calculation fails (e.g., due to overflow).
    fn get_rpc_program_accounts_config<T: jito_bytemuck::Discriminator>(
        &self,
    ) -> Result<RpcProgramAccountsConfig, JitoVaultCrankerError> {
        let data_size = std::mem::size_of::<T>()
            .checked_add(8)
            .ok_or_else(|| JitoVaultCrankerError::MathOverflow("Failed to add".to_string()))?;
        let encoded_discriminator =
            general_purpose::STANDARD.encode(vec![T::DISCRIMINATOR, 0, 0, 0, 0, 0, 0, 0]);
        let memcmp = RpcFilterType::Memcmp(Memcmp::new(
            0,
            MemcmpEncodedBytes::Base64(encoded_discriminator),
        ));
        let config = RpcProgramAccountsConfig {
            filters: Some(vec![RpcFilterType::DataSize(data_size as u64), memcmp]),
            account_config: RpcAccountInfoConfig {
                encoding: Some(UiAccountEncoding::Base64),
                data_slice: Some(UiDataSliceConfig {
                    offset: 0,
                    length: data_size,
                }),
                commitment: None,
                min_context_slot: None,
            },
            with_context: Some(false),
            sort_results: Some(false),
        };

        Ok(config)
    }

    /// Sends and confirms a transaction with retries, priority fees, and blockhash refresh
    ///
    /// # Arguments
    /// * `instructions` - Vector of instructions to include in the transaction
    ///
    /// # Returns
    /// Returns `Result<(), JitoVaultCrankerError>` indicating success or failure
    async fn send_and_confirm_transaction_with_retry(
        &self,
        payer: &Keypair,
        mut instructions: Vec<Instruction>,
    ) -> Result<(), JitoVaultCrankerError> {
        let rpc_client = self.rpc_client.clone();
        let mut retries = 0;
        const MAX_RETRIES: u8 = 10;

        instructions.insert(
            0,
            ComputeBudgetInstruction::set_compute_unit_price(self.priority_fees),
        );

        while retries < MAX_RETRIES {
            let blockhash = get_latest_blockhash_with_retry(&rpc_client).await?;
            let tx = Transaction::new_signed_with_payer(
                &instructions,
                Some(&payer.pubkey()),
                &[payer],
                blockhash,
            );

            match rpc_client
                .send_and_confirm_transaction_with_spinner_and_commitment(
                    &tx,
                    CommitmentConfig::confirmed(),
                )
                .await
            {
                Ok(_) => return Ok(()),
                Err(err) => {
                    retries += 1;
                    if retries >= MAX_RETRIES {
                        error!(
                            "Transaction failed after {} retries: {:?}",
                            MAX_RETRIES, err
                        );
                        return Err(JitoVaultCrankerError::TransactionRetryExhausted {
                            retries: MAX_RETRIES,
                            last_error: err.to_string(),
                        });
                    }
                    sleep(Duration::from_secs(1)).await;
                }
            };
        }

        Ok(())
    }

    /// Retrieves all existing vaults
    ///
    /// # Returns
    ///
    /// Returns a `Result<Vec<(Pubkey, Vault)>, JitoVaultCrankerError>` containing a vector of `(Pubkey, Vault)` tuples
    /// representing all the vault accounts associated with the program. Each tuple
    /// consists of:
    /// - `Pubkey`: The public key of the vault account.
    /// - `Vault`: The deserialized vault data from the account.
    pub async fn get_vaults(&self) -> Result<Vec<(Pubkey, Vault)>, JitoVaultCrankerError> {
        let rpc_client = self.rpc_client.clone();
        let config = self.get_rpc_program_accounts_config::<Vault>()?;

        let accounts = rpc_client
            .get_program_accounts_with_config(&self.vault_program_id, config)
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
    /// An `Result<Vec<(Pubkey, VaultOperatorDelegation)>, JitoVaultCrankerError>` containing a vector of `(Pubkey, VaultOperatorDelegation)` tuples. Each
    /// tuple represents a vault operator delegation account and includes:
    /// - `Pubkey`: The public key of the vault operator delegation account.
    /// - `VaultOperatorDelegation`: The deserialized vault operator delegation data.
    pub async fn get_vault_operator_delegations(
        &self,
    ) -> Result<Vec<(Pubkey, VaultOperatorDelegation)>, JitoVaultCrankerError> {
        let rpc_client = self.rpc_client.clone();
        let config = self.get_rpc_program_accounts_config::<VaultOperatorDelegation>()?;

        let accounts = rpc_client
            .get_program_accounts_with_config(&self.vault_program_id, config)
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

    /// Retrieves the `VaultUpdateStateTracker` for a specific vault and epoch.
    ///
    /// # Returns
    ///
    /// Returns a `Result<VaultUpdateStateTracker, JitoVaultCrankerError>` containing the deserialized state tracker
    /// for the given vault and epoch. If successful, the state tracker is returned; otherwise,
    /// an error is returned with contextual information.
    pub async fn get_update_state_tracker(
        &self,
        vault: &Pubkey,
        ncn_epoch: u64,
    ) -> Result<VaultUpdateStateTracker, JitoVaultCrankerError> {
        let rpc_client = self.rpc_client.clone();
        let pubkey =
            VaultUpdateStateTracker::find_program_address(&self.vault_program_id, vault, ncn_epoch)
                .0;

        let account = rpc_client.get_account(&pubkey).await?;

        match VaultUpdateStateTracker::try_from_slice_unchecked(&account.data) {
            Ok(tracker) => Ok(*tracker),
            Err(e) => Err(JitoVaultCrankerError::Deserialization {
                account_type: "VaultUpdateStateTracker".to_string(),
                pubkey: pubkey.to_string(),
                src: e.to_string(),
            }),
        }
    }

    /// Performs a complete vault update cycle: initializes tracker, cranks it, and closes it.
    ///
    /// # Returns
    ///
    /// Returns `Result<(), JitoVaultCrankerError>` indicating success or failure of the update operation.
    pub async fn do_vault_update(
        &self,
        payer: &Keypair,
        epoch: u64,
        vault: &Pubkey,
        operators: &[Pubkey],
    ) -> Result<(), JitoVaultCrankerError> {
        let tracker_pubkey =
            VaultUpdateStateTracker::find_program_address(&self.vault_program_id, vault, epoch).0;

        log::info!("Updating vault: {vault}");

        // Initialize
        if let Err(e) = self.get_update_state_tracker(vault, epoch).await {
            log::info!("Get tracker failed, initializing. Expecting AccountNotFound: {e}");
            self.initialize_vault_update_state_tracker(payer, vault, tracker_pubkey)
                .await?;
        }

        log::info!("Initialized tracker for vault: {vault}, tracker: {tracker_pubkey}");

        // Crank
        self.crank(payer, epoch, vault, operators, tracker_pubkey)
            .await?;

        log::info!("Cranked vault: {vault}");

        // Close
        let tracker = self.get_update_state_tracker(vault, epoch).await?;
        if operators.is_empty() || tracker.all_operators_updated(operators.len() as u64)? {
            self.close_vault_update_state_tracker(payer, vault, epoch, tracker_pubkey)
                .await?;
        } else {
            return Err(JitoVaultCrankerError::IncompleteCranking {
                vault: vault.to_string(),
                tracker: tracker_pubkey.to_string(),
            });
        }

        log::info!("Closed tracker for vault: {vault}");
        Ok(())
    }

    /// Initializes a vault update state tracker for a given epoch and vault.
    ///
    /// # Returns
    ///
    /// Returns `Result<(), JitoVaultCrankerError>` indicating success or failure of initialization.
    pub async fn initialize_vault_update_state_tracker(
        &self,
        payer: &Keypair,
        vault: &Pubkey,
        tracker_pubkey: Pubkey,
    ) -> Result<(), JitoVaultCrankerError> {
        let mut init_ix_builder = InitializeVaultUpdateStateTrackerBuilder::new();
        init_ix_builder
            .config(self.config_address)
            .vault(*vault)
            .vault_update_state_tracker(tracker_pubkey)
            .payer(payer.pubkey())
            .withdrawal_allocation_method(WithdrawalAllocationMethod::Greedy);
        let mut init_ix = init_ix_builder.instruction();
        init_ix.program_id = self.vault_program_id;

        self.send_and_confirm_transaction_with_retry(payer, vec![init_ix])
            .await?;
        Ok(())
    }

    /// Cranks the [`VaultUpdateStateTracker`] for a specific epoch and list of operators.
    ///
    /// # Returns
    ///
    /// This method returns a `Result<(), JitoVaultCrankerError>` that indicates whether the crank operation
    /// was successful or not.
    pub async fn crank(
        &self,
        payer: &Keypair,
        epoch: u64,
        vault: &Pubkey,
        operators: &[Pubkey],
        tracker_pubkey: Pubkey,
    ) -> Result<(), JitoVaultCrankerError> {
        let tracker = self.get_update_state_tracker(vault, epoch).await?;

        if operators.is_empty() || tracker.all_operators_updated(operators.len() as u64)? {
            return Ok(());
        }

        let end_index = (epoch as usize).checked_rem(operators.len()).ok_or(
            JitoVaultCrankerError::MathError("Division by zero in epoch calculation".to_string()),
        )?;

        // Skip updated operators if cranking has already started
        let start_index = if tracker.last_updated_index() == u64::MAX {
            end_index
        } else {
            tracker.last_updated_index() as usize
        };

        let operators_iter = if start_index < end_index {
            // Crank from start index to end index
            operators
                .iter()
                .take(end_index)
                .skip(start_index)
                .collect::<Vec<_>>()
                .into_iter()
        } else {
            // Crank through operators from start index to operators.len() and then 0 to end_index
            operators
                .iter()
                .skip(start_index)
                .chain(operators.iter().take(end_index))
                .collect::<Vec<_>>()
                .into_iter()
        };

        // Need to send each transaction in serial since strict sequence is required
        for operator in operators_iter {
            let vault_operator_delegation = VaultOperatorDelegation::find_program_address(
                &self.vault_program_id,
                vault,
                operator,
            )
            .0;

            let mut ix_builder = CrankVaultUpdateStateTrackerBuilder::new();
            ix_builder
                .config(self.config_address)
                .vault(*vault)
                .operator(*operator)
                .vault_operator_delegation(vault_operator_delegation)
                .vault_update_state_tracker(tracker_pubkey);
            let mut ix = ix_builder.instruction();
            ix.program_id = self.vault_program_id;

            self.send_and_confirm_transaction_with_retry(payer, vec![ix])
                .await?;
        }

        Ok(())
    }

    /// Closes a vault update state tracker for a given epoch and vault.
    ///
    /// # Returns
    ///
    /// Returns `Result<(), JitoVaultCrankerError>` indicating success or failure of closing.
    pub async fn close_vault_update_state_tracker(
        &self,
        payer: &Keypair,
        vault: &Pubkey,
        epoch: u64,
        tracker_pubkey: Pubkey,
    ) -> Result<(), JitoVaultCrankerError> {
        let mut close_ix_builder = CloseVaultUpdateStateTrackerBuilder::new();
        close_ix_builder
            .config(self.config_address)
            .vault(*vault)
            .payer(payer.pubkey())
            .vault_update_state_tracker(tracker_pubkey)
            .ncn_epoch(epoch);
        let mut close_ix = close_ix_builder.instruction();
        close_ix.program_id = self.vault_program_id;

        self.send_and_confirm_transaction_with_retry(payer, vec![close_ix])
            .await?;
        Ok(())
    }
}
