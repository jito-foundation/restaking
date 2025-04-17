use ::log::info;
use anyhow::anyhow;
use base64::{engine::general_purpose, Engine};
use borsh::BorshDeserialize;
use jito_restaking_client_common::log::PrettyDisplay;
use log::print_base58_tx;
use solana_account_decoder::{UiAccountEncoding, UiDataSliceConfig};
use solana_rpc_client::nonblocking::rpc_client::RpcClient;
use solana_rpc_client_api::{
    config::{RpcAccountInfoConfig, RpcProgramAccountsConfig},
    filter::{Memcmp, MemcmpEncodedBytes, RpcFilterType},
};
use solana_sdk::{
    commitment_config::CommitmentConfig, instruction::Instruction, pubkey::Pubkey,
    signature::Keypair, transaction::Transaction,
};

pub mod cli_args;
pub mod log;
pub mod restaking;
pub mod restaking_handler;
pub mod vault;
pub mod vault_handler;

pub struct CliConfig {
    pub rpc_url: String,

    pub commitment: CommitmentConfig,

    pub keypair: Option<Keypair>,
}

pub(crate) trait CliHandler {
    fn cli_config(&self) -> &CliConfig;

    fn print_tx(&self) -> bool;

    /// Creates a new RPC client using the configuration from the CLI handler.
    ///
    /// This method constructs an RPC client with the URL and commitment level specified in the
    /// CLI configuration. The client can be used to communicate with a Solana node for
    /// submitting transactions, querying account data, and other RPC operations.
    fn get_rpc_client(&self) -> RpcClient {
        RpcClient::new_with_commitment(
            self.cli_config().rpc_url.clone(),
            self.cli_config().commitment,
        )
    }

    /// Creates an RPC program accounts configuration for fetching accounts of type `T` with an optional public key filter.
    ///
    /// This method constructs a configuration that can be used with RPC methods to fetch program accounts
    /// that match specific criteria. It automatically adds filters for the account data size and the discriminator
    /// of type `T` to ensure only accounts of the expected type are returned.
    fn get_rpc_program_accounts_config<T: jito_bytemuck::Discriminator>(
        &self,
        filter_pubkey: Option<(&Pubkey, usize)>,
    ) -> anyhow::Result<RpcProgramAccountsConfig> {
        let data_size = std::mem::size_of::<T>()
            .checked_add(8)
            .ok_or_else(|| anyhow!("Failed to add"))?;

        let encoded_discriminator =
            general_purpose::STANDARD.encode(vec![T::DISCRIMINATOR, 0, 0, 0, 0, 0, 0, 0]);
        let discriminator_filter = RpcFilterType::Memcmp(Memcmp::new(
            0,
            MemcmpEncodedBytes::Base64(encoded_discriminator),
        ));

        let mut filters = vec![
            RpcFilterType::DataSize(data_size as u64),
            discriminator_filter,
        ];

        if let Some((pubkey, offset)) = filter_pubkey {
            let pubkey_filter = RpcFilterType::Memcmp(Memcmp::new(
                offset,
                MemcmpEncodedBytes::Base64(general_purpose::STANDARD.encode(pubkey.to_bytes())),
            ));

            filters.push(pubkey_filter);
        }

        let config = RpcProgramAccountsConfig {
            filters: Some(filters),
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
    /// Fetches and deserializes an account
    ///
    /// This method retrieves account data using the configured RPC client,
    /// then deserializes it into the specified account type using Borsh deserialization.
    async fn get_account<T: BorshDeserialize + PrettyDisplay>(
        &self,
        account_pubkey: &Pubkey,
    ) -> anyhow::Result<T> {
        let rpc_client = self.get_rpc_client();

        let account = rpc_client.get_account(account_pubkey).await?;
        let account = T::deserialize(&mut account.data.as_slice())?;

        Ok(account)
    }

    /// Processes a transaction by either printing it as Base58 or sending it.
    ///
    /// This method handles the logic for processing a set of instructions as a transaction.
    /// If `print_tx` is enabled in the CLI handler (helpful for running commands in Squads), it will print the transaction in Base58 format
    /// without sending it. Otherwise, it will submit and confirm the transaction.
    async fn process_transaction(
        &self,
        ixs: &[Instruction],
        payer: &Pubkey,
        keypairs: &[&Keypair],
    ) -> anyhow::Result<()> {
        let rpc_client = self.get_rpc_client();

        if self.print_tx() {
            print_base58_tx(ixs);
        } else {
            let blockhash = rpc_client.get_latest_blockhash().await?;
            let tx = Transaction::new_signed_with_payer(ixs, Some(payer), keypairs, blockhash);
            let result = rpc_client.send_and_confirm_transaction(&tx).await?;

            info!("Transaction confirmed: {:?}", result);
        }

        Ok(())
    }
}
