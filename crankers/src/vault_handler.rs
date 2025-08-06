use std::time::Duration;

use anyhow::Context;
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
    config::Config, vault::Vault, vault_operator_delegation::VaultOperatorDelegation,
    vault_update_state_tracker::VaultUpdateStateTracker,
};
use log::{error, info};
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

use crate::core::get_latest_blockhash_with_retry;

const MAX_RETRIES: u8 = 10;

pub struct VaultHandler {
    rpc_url: String,
    vault_program_id: Pubkey,
    config_address: Pubkey,
    priority_fees: u64,
}

impl VaultHandler {
    pub fn new(
        rpc_url: &str,
        vault_program_id: Pubkey,
        config_address: Pubkey,
        priority_fees: u64,
    ) -> Self {
        Self {
            rpc_url: rpc_url.to_string(),
            vault_program_id,
            config_address,
            priority_fees,
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

    /// Constructs an `RpcProgramAccountsConfig` for querying accounts of a given type `T`.
    ///
    /// # Returns
    /// - `Ok(RpcProgramAccountsConfig)`: A valid configuration for filtering accounts in
    ///   Solana's RPC API.
    /// - `Err(anyhow::Error)`: If the data size calculation fails (e.g., due to overflow).
    fn get_rpc_program_accounts_config<T: jito_bytemuck::Discriminator>(
        &self,
    ) -> anyhow::Result<RpcProgramAccountsConfig> {
        let data_size = std::mem::size_of::<T>()
            .checked_add(8)
            .ok_or_else(|| anyhow::anyhow!("Failed to add"))?;
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
    /// * `payer` - Keypair of payer
    /// * `instructions` - Vector of instructions to include in the transaction
    ///
    /// # Returns
    /// Returns `anyhow::Result<()>` indicating success or failure
    async fn send_and_confirm_transaction_with_retry(
        &self,
        payer: &Keypair,
        mut instructions: Vec<Instruction>,
    ) -> anyhow::Result<()> {
        let rpc_client = self.get_rpc_client();
        let mut retries = 0;

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

            let err = match rpc_client
                .send_and_confirm_transaction_with_spinner_and_commitment(
                    &tx,
                    CommitmentConfig::confirmed(),
                )
                .await
            {
                Ok(_) => return Ok(()),
                Err(err) => {
                    retries += 1;
                    if retries < MAX_RETRIES {
                        sleep(Duration::from_secs(1)).await;
                    }
                    err
                }
            };

            if retries >= MAX_RETRIES {
                error!(
                    "Transaction failed after {} retries: {:?}",
                    MAX_RETRIES, err
                );
            }
        }

        Err(anyhow::anyhow!(
            "Transaction failed after {} retries",
            MAX_RETRIES
        ))
    }

    /// Splits a vector of instructions into multiple transactions to stay within Solana's
    /// transaction size limit of 1232 bytes.
    ///
    /// This function dynamically batches instructions by testing the actual transaction size
    /// rather than using fixed batch sizes. Each transaction will include a compute budget
    /// instruction at the beginning.
    async fn split_instructions_by_size(
        &self,
        instructions: &[Instruction],
        payer: &Keypair,
        max_size: usize,
    ) -> anyhow::Result<Vec<Transaction>> {
        let mut transactions = Vec::new();
        let mut current_batch = Vec::new();

        let compute_budget_ix =
            ComputeBudgetInstruction::set_compute_unit_price(self.priority_fees);

        for instruction in instructions {
            // Create a test transaction with current batch + new instruction
            let mut test_batch = vec![compute_budget_ix.clone()];
            test_batch.extend(current_batch.clone());
            test_batch.push(instruction.clone());

            let blockhash = get_latest_blockhash_with_retry(&self.get_rpc_client()).await?;
            let test_tx = Transaction::new_signed_with_payer(
                &test_batch,
                Some(&payer.pubkey()),
                &[payer],
                blockhash,
            );

            let tx_size = test_tx.signatures.len() + test_tx.message_data().len();

            if tx_size > max_size && !current_batch.is_empty() {
                // Finalize current batch
                let mut final_batch = vec![compute_budget_ix.clone()];
                final_batch.extend(current_batch.clone());

                let blockhash = get_latest_blockhash_with_retry(&self.get_rpc_client()).await?;
                let tx = Transaction::new_signed_with_payer(
                    &final_batch,
                    Some(&payer.pubkey()),
                    &[payer],
                    blockhash,
                );
                transactions.push(tx);

                // Start new batch with current instruction
                current_batch = vec![instruction.clone()];
            } else {
                current_batch.push(instruction.clone());
            }
        }

        // Handle remaining instructions
        if !current_batch.is_empty() {
            let mut final_batch = vec![compute_budget_ix];
            final_batch.extend(current_batch);

            let blockhash = get_latest_blockhash_with_retry(&self.get_rpc_client()).await?;
            let tx = Transaction::new_signed_with_payer(
                &final_batch,
                Some(&payer.pubkey()),
                &[payer],
                blockhash,
            );
            transactions.push(tx);
        }

        Ok(transactions)
    }

    /// Retrieves Jito Vault Program account
    pub async fn get_vault_program_account<T: AccountDeserialize>(
        &self,
        pubkey: &Pubkey,
    ) -> anyhow::Result<T> {
        let rpc_client = self.get_rpc_client();

        match rpc_client.get_account(pubkey).await {
            Ok(account) => match T::try_from_slice_unchecked(&account.data) {
                Ok(vault_operator_delegation) => Ok(*vault_operator_delegation),
                Err(e) => {
                    let context = format!("Failed deserializing: {pubkey}");
                    Err(anyhow::Error::new(e).context(context))
                }
            },
            Err(e) => {
                let context = format!("Error: Failed to get account: {pubkey}");
                Err(anyhow::Error::new(e).context(context))
            }
        }
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
    /// An `anyhow::Result` containing a vector of `(Pubkey, VaultOperatorDelegation)` tuples. Each
    /// tuple represents a vault operator delegation account and includes:
    /// - `Pubkey`: The public key of the vault operator delegation account.
    /// - `VaultOperatorDelegation`: The deserialized vault operator delegation data.
    pub async fn get_vault_operator_delegations(
        &self,
    ) -> anyhow::Result<Vec<(Pubkey, VaultOperatorDelegation)>> {
        let rpc_client = self.get_rpc_client();
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
    /// Returns an `anyhow::Result<VaultUpdateStateTracker>` containing the deserialized state tracker
    /// for the given vault and epoch. If successful, the state tracker is returned; otherwise,
    /// an error is returned with contextual information.
    pub async fn get_update_state_tracker(
        &self,
        vault: &Pubkey,
        ncn_epoch: u64,
    ) -> anyhow::Result<VaultUpdateStateTracker> {
        let rpc_client = self.get_rpc_client();

        let pubkey =
            VaultUpdateStateTracker::find_program_address(&self.vault_program_id, vault, ncn_epoch)
                .0;

        match rpc_client.get_account(&pubkey).await {
            Ok(account) => match VaultUpdateStateTracker::try_from_slice_unchecked(&account.data) {
                Ok(tracker) => Ok(*tracker),
                Err(e) => {
                    let context = format!("Failed deserializing VaultUpdateStateTracker: {pubkey}");
                    Err(anyhow::Error::new(e).context(context))
                }
            },
            Err(e) => {
                let context =
                    format!("Error: Failed to get VaultUpdateStateTracker account: {pubkey}");
                Err(anyhow::Error::new(e).context(context))
            }
        }
    }

    /// Performs a complete vault update cycle: initializes tracker, cranks it, and closes it.
    ///
    /// # Returns
    ///
    /// Returns `anyhow::Result<()>` indicating success or failure of the update operation.
    pub async fn do_vault_update(
        &self,
        slot: u64,
        config: &jito_vault_core::config::Config,
        payer: &Keypair,
        vault: &Pubkey,
        operators: &[Pubkey],
    ) -> anyhow::Result<()> {
        let epoch = get_epoch(slot, config.epoch_length())?;
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
        self.crank(slot, config, payer, vault, operators, tracker_pubkey)
            .await?;

        log::info!("Cranked vault: {vault}");

        // Close
        let tracker = self.get_update_state_tracker(vault, epoch).await?;
        if operators.is_empty() || tracker.all_operators_updated(operators.len() as u64)? {
            self.close_vault_update_state_tracker(payer, vault, epoch, tracker_pubkey)
                .await?;
        } else {
            let context = format!(
                "Cranking failed to update all operators for vault: {vault}, tracker: {tracker_pubkey}"
            );
            return Err(anyhow::anyhow!(context));
        }

        log::info!("Closed tracker for vault: {vault}");

        Ok(())
    }

    /// Initializes a vault update state tracker for a given epoch and vault.
    ///
    /// # Returns
    ///
    /// Returns `anyhow::Result<()>` indicating success or failure of initialization.
    pub async fn initialize_vault_update_state_tracker(
        &self,
        payer: &Keypair,
        vault: &Pubkey,
        tracker_pubkey: Pubkey,
    ) -> anyhow::Result<()> {
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

    /// Retrieves operators that need to be updated and builds their crank instructions.
    ///
    /// # Arguments
    /// * `operators_iter` - Iterator of operator public keys to check
    /// * `slot` - Current slot number
    /// * `config` - Configuration containing epoch length
    /// * `vault` - Vault public key
    /// * `tracker_pubkey` - Vault update state tracker public key
    ///
    /// # Returns
    /// * `Vec<Instruction>` - Vector of crank instructions for operators that need updates
    async fn retrieve_non_updated_operators(
        &self,
        operators_iter: &[&Pubkey],
        slot: u64,
        config: &Config,
        vault: &Pubkey,
        tracker_pubkey: Pubkey,
    ) -> anyhow::Result<Vec<Instruction>> {
        let mut instructions = Vec::with_capacity(operators_iter.len());

        for operator in operators_iter {
            let vault_operator_delegation_pubkey = VaultOperatorDelegation::find_program_address(
                &self.vault_program_id,
                vault,
                operator,
            )
            .0;

            let vault_operator_delegation: VaultOperatorDelegation = self
                .get_vault_program_account(&vault_operator_delegation_pubkey)
                .await?;

            // Check if operator is NOT already updated (inverted logic)
            if vault_operator_delegation
                .check_is_already_updated(slot, config.epoch_length())
                .is_ok()
            {
                let mut ix_builder = CrankVaultUpdateStateTrackerBuilder::new();
                ix_builder
                    .config(self.config_address)
                    .vault(*vault)
                    .operator(**operator)
                    .vault_operator_delegation(vault_operator_delegation_pubkey)
                    .vault_update_state_tracker(tracker_pubkey);

                let mut ix = ix_builder.instruction();
                ix.program_id = self.vault_program_id;

                instructions.push(ix);
            }
        }

        Ok(instructions)
    }

    /// Cranks the [`VaultUpdateStateTracker`] for a specific epoch and list of operators.
    ///
    /// - Try to crank maximum 10 times
    /// - Batch multiple operator cranks per one transaction
    /// - Cycle send transaction, check `is_already_updated`, then retry
    ///
    /// # Returns
    ///
    /// This method returns an `anyhow::Result<()>` that indicates whether the crank operation
    /// was successful or not.
    pub async fn crank(
        &self,
        slot: u64,
        config: &Config,
        payer: &Keypair,
        vault: &Pubkey,
        operators: &[Pubkey],
        tracker_pubkey: Pubkey,
    ) -> anyhow::Result<()> {
        let rpc_client = self.get_rpc_client();
        let epoch = get_epoch(slot, config.epoch_length())?;
        let tracker = self.get_update_state_tracker(vault, epoch).await?;

        if operators.is_empty() || tracker.all_operators_updated(operators.len() as u64)? {
            return Ok(());
        }

        let end_index = (epoch as usize)
            .checked_rem(operators.len())
            .context("No operators to crank")?;

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
        } else {
            // Crank through operators from start index to operators.len() and then 0 to end_index
            operators
                .iter()
                .skip(start_index)
                .chain(operators.iter().take(end_index))
                .collect::<Vec<_>>()
        };

        // Need to send each transaction in serial since strict sequence is required
        let instructions = self
            .retrieve_non_updated_operators(&operators_iter, slot, config, vault, tracker_pubkey)
            .await?;

        if instructions.is_empty() {
            return Ok(());
        }

        let txs = self
            .split_instructions_by_size(&instructions, payer, 1232)
            .await?;

        for (i, tx) in txs.iter().enumerate() {
            let mut retries = 0;

            // Retry loop for current transaction
            loop {
                match rpc_client
                    .send_and_confirm_transaction_with_spinner_and_commitment(
                        tx,
                        CommitmentConfig::confirmed(),
                    )
                    .await
                {
                    Ok(_) => {
                        info!(
                            "✅ Transaction {}/{} completed successfully",
                            i + 1,
                            txs.len()
                        );
                        break; // Success - move to next transaction
                    }
                    Err(err) => {
                        retries += 1;

                        if retries <= MAX_RETRIES {
                            info!(
                            "⚠️  Transaction {}/{} failed (attempt {}/{}), retrying in 1s: {:?}",
                            i + 1, txs.len(), retries, MAX_RETRIES, err
                        );
                            sleep(Duration::from_secs(1)).await;
                        } else {
                            error!(
                                "❌ Transaction {}/{} failed permanently after {} retries: {:?}",
                                i + 1,
                                txs.len(),
                                MAX_RETRIES,
                                err
                            );
                            return Err(anyhow::anyhow!(
                                "Transaction {} failed after {} retries: {}",
                                i + 1,
                                MAX_RETRIES,
                                err
                            ));
                        }
                    }
                }
            }
        }

        info!(
            "🎉 All {} transactions completed successfully for vault cranking!",
            txs.len()
        );
        Ok(())
    }

    /// Closes a vault update state tracker for a given epoch and vault.
    ///
    /// # Returns
    ///
    /// Returns `anyhow::Result<()>` indicating success or failure of closing.
    pub async fn close_vault_update_state_tracker(
        &self,
        payer: &Keypair,
        vault: &Pubkey,
        epoch: u64,
        tracker_pubkey: Pubkey,
    ) -> anyhow::Result<()> {
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
