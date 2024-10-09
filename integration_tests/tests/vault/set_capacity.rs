#[cfg(test)]
mod tests {
    use jito_vault_core::config::Config;
    use jito_vault_sdk::error::VaultError;
    use solana_sdk::{signature::Keypair, signer::Signer};

    use crate::fixtures::{fixture::TestBuilder, vault_client::assert_vault_error};

    #[tokio::test]
    async fn test_set_capacity_ok() {
        let fixture = TestBuilder::new().await;
        let mut vault_program_client = fixture.vault_program_client();

        let (_config_admin, vault_root) = vault_program_client
            .setup_config_and_vault(0, 0, 0, 0)
            .await
            .unwrap();

        let vault = vault_program_client
            .get_vault(&vault_root.vault_pubkey)
            .await
            .unwrap();
        assert_eq!(vault.deposit_capacity(), u64::MAX);

        let config_pubkey = Config::find_program_address(&jito_vault_program::id()).0;
        vault_program_client
            .set_capacity(
                &config_pubkey,
                &vault_root.vault_pubkey,
                &vault_root.vault_admin,
                100,
            )
            .await
            .unwrap();

        let vault = vault_program_client
            .get_vault(&vault_root.vault_pubkey)
            .await
            .unwrap();
        assert_eq!(vault.deposit_capacity(), 100);
    }

    #[tokio::test]
    async fn test_set_capacity_wrong_admin() {
        let fixture = TestBuilder::new().await;
        let mut vault_program_client = fixture.vault_program_client();

        let (_config_admin, vault_root) = vault_program_client
            .setup_config_and_vault(0, 0, 0, 0)
            .await
            .unwrap();

        let wrong_admin = Keypair::new();
        vault_program_client
            .airdrop(&wrong_admin.pubkey(), 1.0)
            .await
            .unwrap();

        let config_pubkey = Config::find_program_address(&jito_vault_program::id()).0;
        let result = vault_program_client
            .set_capacity(&config_pubkey, &vault_root.vault_pubkey, &wrong_admin, 100)
            .await;
        assert_vault_error(result, VaultError::VaultCapacityAdminInvalid);
    }
}
