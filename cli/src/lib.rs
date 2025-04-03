use anyhow::anyhow;
use base64::{engine::general_purpose, Engine};
use solana_account_decoder::{UiAccountEncoding, UiDataSliceConfig};
use solana_rpc_client::nonblocking::rpc_client::RpcClient;
use solana_rpc_client_api::{
    config::{RpcAccountInfoConfig, RpcProgramAccountsConfig},
    filter::{Memcmp, MemcmpEncodedBytes, RpcFilterType},
};
use solana_sdk::{commitment_config::CommitmentConfig, pubkey::Pubkey, signature::Keypair};

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

    fn get_rpc_client(&self) -> RpcClient {
        RpcClient::new_with_commitment(
            self.cli_config().rpc_url.clone(),
            self.cli_config().commitment,
        )
    }

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
}
