use crate::result::Result;
use solana_rpc_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;

pub struct VaultRpcClient {
    client: RpcClient,
    program_id: Pubkey,
}

impl VaultRpcClient {
    pub fn new(client: RpcClient, program_id: &Pubkey) -> Self {
        Self {
            client,
            program_id: *program_id,
        }
    }

    pub fn initialize_config(&self) -> Result<()> {
        Ok(())
    }
}
