use std::{future::Future, sync::Arc};

use solana_rpc_client::nonblocking::rpc_client::RpcClient;
use solana_rpc_client_api::client_error::Error as ClientError;
use solana_sdk::{
    account::Account, commitment_config::CommitmentConfig, hash::Hash, pubkey::Pubkey,
};
use tokio::task;

pub async fn retry<F, Fut, T, E>(mut f: F, retries: usize) -> Result<T, E>
where
    F: FnMut() -> Fut + Send,
    Fut: Future<Output = Result<T, E>> + Send,
{
    let mut attempts = 0usize;
    loop {
        let future = f();
        match future.await {
            Ok(result) => return Ok(result),
            Err(e) => {
                attempts = match attempts.checked_add(1) {
                    Some(new_attempts) => new_attempts,
                    None => {
                        return Err(e);
                    }
                };
                if attempts > retries {
                    return Err(e);
                }
            }
        }
    }
}

pub async fn get_latest_blockhash_with_retry(client: &RpcClient) -> Result<Hash, ClientError> {
    retry(
        || async {
            client
                .get_latest_blockhash_with_commitment(CommitmentConfig::finalized())
                .await
                .map(|r| r.0)
        },
        5,
    )
    .await
}

pub async fn get_multiple_accounts_with_retry(
    client: &RpcClient,
    pubkeys: &[Pubkey],
) -> Result<Vec<Option<Account>>, ClientError> {
    retry(|| async { client.get_multiple_accounts(pubkeys).await }, 5).await
}

pub async fn get_multiple_accounts_batched(
    accounts: &[Pubkey],
    rpc_client: &Arc<RpcClient>,
) -> anyhow::Result<Vec<Option<Account>>> {
    let tasks = accounts.chunks(100).map(|chunk| {
        let client = Arc::clone(rpc_client);
        let chunk = chunk.to_owned();
        task::spawn(
            async move { get_multiple_accounts_with_retry(&client, chunk.as_slice()).await },
        )
    });

    let mut accounts_result = Vec::new();
    for result in futures::future::join_all(tasks).await.into_iter() {
        match result {
            Ok(Ok(accounts)) => accounts_result.extend(accounts),
            Ok(Err(e)) => {
                return Err(anyhow::anyhow!("RPC client error: {}", e));
            }
            Err(e) => return Err(anyhow::anyhow!("Task join error: {}", e)),
        }
    }
    Ok(accounts_result)
}
