use std::sync::Arc;

use jito_bytemuck::AccountDeserialize;
use jito_restaking_core::operator::Operator;
use solana_rpc_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{commitment_config::CommitmentConfig, pubkey::Pubkey};

use crate::core::get_multiple_accounts_batched;

pub struct RestakingHandler {
    rpc_url: String,
}

impl RestakingHandler {
    pub fn new(rpc_url: &str) -> Self {
        Self {
            rpc_url: rpc_url.to_string(),
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

    /// Fetches a list of operator public keys from the provided array of public keys.
    ///
    /// This method retrieves multiple accounts from the RPC client based on the
    /// provided `operator_pubkeys`. It deserializes the account data into [`Operator`] structs and
    /// filters out any accounts that are not found or cannot be deserialized. The resulting list of
    /// `(Pubkey, Operator)` pairs is sorted by the `index` field of the `Operator`, and only the
    /// public keys are returned.
    ///
    /// # Returns
    ///
    /// An `anyhow::Result` wrapping a `Vec<Pubkey>` containing the public keys of the operators,
    /// sorted by their corresponding operator index.
    ///
    /// # Errors
    ///
    /// Returns an error if the Solana RPC client fails to retrieve the accounts or if deserialization
    /// of the account data into the `Operator` struct fails.
    pub async fn get_operators(
        &self,
        operator_pubkeys: &[Pubkey],
    ) -> anyhow::Result<Vec<(Pubkey, Operator)>> {
        let rpc_client = self.get_rpc_client();

        let accounts =
            get_multiple_accounts_batched(operator_pubkeys, &Arc::new(rpc_client)).await?;

        let mut operators: Vec<(Pubkey, Operator)> = accounts
            .into_iter()
            .enumerate()
            .filter_map(|(index, acc)| match acc {
                Some(acc) => Operator::try_from_slice_unchecked(&acc.data)
                    .map_or(None, |v| Some((operator_pubkeys[index], *v))),
                None => None,
            })
            .collect();

        operators.sort_by(|a, b| a.1.index().cmp(&b.1.index()));

        Ok(operators)
    }
}
