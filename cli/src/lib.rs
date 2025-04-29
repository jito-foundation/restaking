use ::log::info;
use anyhow::anyhow;
use base64::{engine::general_purpose, Engine};
use borsh::BorshDeserialize;
use cli_config::CliConfig;
use cli_signer::CliSigner;
use jito_restaking_client_common::log::PrettyDisplay;
use log::print_base58_tx;
use serde::Serialize;
use solana_account_decoder::{UiAccountEncoding, UiDataSliceConfig};
use solana_rpc_client::nonblocking::rpc_client::RpcClient;
use solana_rpc_client_api::{
    config::{RpcAccountInfoConfig, RpcProgramAccountsConfig},
    filter::{Memcmp, MemcmpEncodedBytes, RpcFilterType},
};
use solana_sdk::{
    instruction::Instruction, pubkey::Pubkey, signers::Signers, transaction::Transaction,
};

pub mod cli_args;
pub mod cli_config;
pub mod cli_signer;
pub mod log;
pub mod restaking;
pub mod restaking_handler;
pub mod vault;
pub mod vault_handler;

pub(crate) trait CliHandler {
    fn cli_config(&self) -> &CliConfig;

    fn print_tx(&self) -> bool;

    fn print_json(&self) -> bool;

    fn print_json_without_reserves(&self) -> bool;

    fn signer(&self) -> anyhow::Result<&CliSigner> {
        self.cli_config()
            .signer
            .as_ref()
            .ok_or_else(|| anyhow!("Signer not provided"))
    }

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
    async fn process_transaction<T>(
        &self,
        ixs: &[Instruction],
        payer: &Pubkey,
        signers: &T,
    ) -> anyhow::Result<()>
    where
        T: Signers + ?Sized,
    {
        let rpc_client = self.get_rpc_client();

        if self.print_tx() {
            print_base58_tx(ixs);
        } else {
            let blockhash = rpc_client.get_latest_blockhash().await?;
            let tx = Transaction::new_signed_with_payer(ixs, Some(payer), signers, blockhash);
            let result = rpc_client.send_and_confirm_transaction(&tx).await?;

            info!("Transaction confirmed: {:?}", result);
        }

        Ok(())
    }

    /// Prints a value either as JSON or using its pretty display format.
    ///
    /// This function provides flexible output formatting for any type that implements both
    /// [`Serialize`] and [`PrettyDisplay`]. It determines the output format based on the
    /// configuration of the containing struct.
    ///
    /// # Format options:
    /// - Default: Uses the [`PrettyDisplay`] trait to format output.
    /// - `--print-json`: Prints the full account information in JSON format.
    /// - `--print-json-without-reserves`: Prints account information in JSON format but automatically
    ///   filters out the `reserved` fields without modifying the original struct.
    fn print_out<T>(
        &self,
        index: Option<usize>,
        address: Option<&Pubkey>,
        value: &T,
    ) -> anyhow::Result<()>
    where
        T: ?Sized + Serialize + PrettyDisplay,
    {
        match (self.print_json(), self.print_json_without_reserves()) {
            (true, true) => {
                return Err(anyhow!("Conflicting flags: both --print-json and --print-json-without-reserves are enabled. Please enable only one of these flags."));
            }
            (true, false) => {
                let json_string = serde_json::to_string_pretty(&value)?;

                println!("{json_string}");
            }
            (false, true) => {
                let mut json_value = serde_json::to_value(value)?;
                self.remove_reserved_fields(&mut json_value);

                let mut account_obj = serde_json::Map::new();
                if let Some(index) = index {
                    account_obj.insert(
                        "index".to_string(),
                        serde_json::Value::String(index.to_string()),
                    );
                }
                if let Some(address) = address {
                    account_obj.insert(
                        "address".to_string(),
                        serde_json::Value::String(address.to_string()),
                    );
                }
                account_obj.insert("data".to_string(), json_value);

                let json_string = serde_json::to_string_pretty(&account_obj)?;

                println!("{json_string}");
            }
            (false, false) => {
                let type_name = std::any::type_name::<T>();
                let msg = address.map_or("".to_string(), |address| {
                    format!("{type_name} at {address}")
                });
                info!("{msg}");
                info!("{}", value.pretty_display());
            }
        }

        Ok(())
    }

    /// Recursively removes all "reserved" fields from a JSON value
    fn remove_reserved_fields(&self, value: &mut serde_json::Value) {
        if let serde_json::Value::Object(map) = value {
            map.remove("reserved");
            map.remove("reserved_space");

            // Recursively process all remaining object values
            for (_, v) in map.iter_mut() {
                self.remove_reserved_fields(v);
            }
        } else if let serde_json::Value::Array(arr) = value {
            // Recursively process array elements
            for item in arr.iter_mut() {
                self.remove_reserved_fields(item);
            }
        }
    }
}
