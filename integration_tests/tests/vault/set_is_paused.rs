#[cfg(test)]
mod tests {
    use jito_vault_sdk::error::VaultError;
    use solana_program::pubkey::Pubkey;
    use solana_sdk::signature::{Keypair, Signer};

    use crate::fixtures::{
        fixture::TestBuilder,
        vault_client::{assert_vault_error, VaultProgramClient, VaultRoot},
    };

    async fn setup() -> (VaultProgramClient, Pubkey, Keypair) {
        let fixture = TestBuilder::new().await;

        let mut vault_program_client = fixture.vault_program_client();

        let deposit_fee_bps = 99;
        let withdrawal_fee_bps = 100;
        let reward_fee_bps = 101;

        let (
            _config_admin,
            VaultRoot {
                vault_pubkey,
                vault_admin,
            },
        ) = vault_program_client
            .setup_config_and_vault(deposit_fee_bps, withdrawal_fee_bps, reward_fee_bps)
            .await
            .unwrap();

        (vault_program_client, vault_pubkey, vault_admin)
    }

    #[tokio::test]
    async fn test_set_is_paused_with_bad_admin() {
        let (mut vault_program_client, vault_pubkey, _) = setup().await;

        let bad_admin = Keypair::new();
        vault_program_client
            .airdrop(&bad_admin.pubkey(), 10.0)
            .await
            .unwrap();

        let response = vault_program_client
            .set_is_paused(&vault_pubkey, &bad_admin, true)
            .await;

        assert_vault_error(response, VaultError::VaultAdminInvalid);
    }

    #[tokio::test]
    async fn test_set_is_paused() {
        let (mut vault_program_client, vault_pubkey, vault_admin) = setup().await;

        let vault = vault_program_client.get_vault(&vault_pubkey).await.unwrap();

        assert!(!vault.is_paused());

        vault_program_client
            .set_is_paused(&vault_pubkey, &vault_admin, true)
            .await
            .unwrap();

        let vault = vault_program_client.get_vault(&vault_pubkey).await.unwrap();

        assert!(vault.is_paused());
    }
}
