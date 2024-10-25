#[cfg(test)]
mod tests {
    use jito_vault_core::config::Config;
    use jito_vault_sdk::error::VaultError;
    use solana_program::instruction::InstructionError;
    use solana_sdk::signature::{Keypair, Signer};

    use crate::fixtures::{assert_ix_error, fixture::TestBuilder};

    #[tokio::test]
    async fn test_set_config_admin_ok() {
        let fixture = TestBuilder::new().await;
        let mut vault_program_client = fixture.vault_program_client();

        // Initialize config
        let old_admin = vault_program_client.do_initialize_config().await.unwrap();

        // Create new admin
        let new_admin = Keypair::new();

        // Set new admin
        vault_program_client
            .set_config_admin(
                &Config::find_program_address(&jito_vault_program::id()).0,
                &old_admin,
                &new_admin,
            )
            .await
            .unwrap();

        // Verify new admin
        let config = vault_program_client
            .get_config(&Config::find_program_address(&jito_vault_program::id()).0)
            .await
            .unwrap();

        assert_eq!(config.admin, new_admin.pubkey());
        assert_eq!(config.fee_admin, new_admin.pubkey());
    }

    #[tokio::test]
    async fn test_set_config_admin_invalid_old_admin() {
        let fixture = TestBuilder::new().await;
        let mut vault_program_client = fixture.vault_program_client();

        // Initialize config
        let _old_admin = vault_program_client.do_initialize_config().await.unwrap();

        // Create invalid old admin and new admin
        let invalid_old_admin = Keypair::new();
        let new_admin = Keypair::new();

        vault_program_client
            .airdrop(&invalid_old_admin.pubkey(), 1.0)
            .await
            .unwrap();

        // Attempt to set new admin with invalid old admin
        let result = vault_program_client
            .set_config_admin(
                &Config::find_program_address(&jito_vault_program::id()).0,
                &invalid_old_admin,
                &new_admin,
            )
            .await;

        assert!(result.is_err());
        assert_ix_error(
            result,
            InstructionError::Custom(VaultError::VaultConfigAdminInvalid as u32),
        );
    }
}
