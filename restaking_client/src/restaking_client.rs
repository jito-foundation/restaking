use solana_rpc_client::nonblocking::rpc_client::RpcClient;

pub struct RestakingRpcClient {
    client: RpcClient,
}

impl RestakingRpcClient {
    pub fn new(client: RpcClient) -> Self {
        Self { client }
    }
}
