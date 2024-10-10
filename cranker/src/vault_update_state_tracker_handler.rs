use jito_bytemuck::AccountDeserialize;
use jito_vault_core::vault_update_state_tracker::VaultUpdateStateTracker;
use solana_rpc_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{commitment_config::CommitmentConfig, pubkey::Pubkey};

pub struct VaultUpdateStateTrackerHandler {
    rpc_url: String,
    vault_program_id: Pubkey,
}

impl VaultUpdateStateTrackerHandler {
    pub fn new(rpc_url: &str, vault_program_id: Pubkey) -> Self {
        Self {
            rpc_url: rpc_url.to_string(),
            vault_program_id,
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
}
