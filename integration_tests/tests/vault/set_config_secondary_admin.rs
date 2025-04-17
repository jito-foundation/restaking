#[cfg(test)]
mod tests {
    use jito_vault_core::config::Config;
    use jito_vault_sdk::{error::VaultError, instruction::ConfigAdminRole};
    use solana_sdk::{
        pubkey::Pubkey,
        signature::{Keypair, Signer},
    };

    use crate::fixtures::{fixture::TestBuilder, vault_client::assert_vault_error};

    #[tokio::test]
    async fn test_set_config_secondary_admin_ok() {
        let fixture = TestBuilder::new().await;
        let mut vault_program_client = fixture.vault_program_client();
        let config_address = Config::find_program_address(&jito_vault_program::id()).0;

        // Initialize config
        let old_admin = vault_program_client.do_initialize_config().await.unwrap();

        {
            // Fee Admin
            let new_admin = Pubkey::new_unique();
            vault_program_client
                .set_config_secondary_admin(
                    &config_address,
                    &old_admin,
                    &new_admin,
                    ConfigAdminRole::FeeAdmin,
                )
                .await
                .unwrap();

            let config = vault_program_client
                .get_config(&config_address)
                .await
                .unwrap();
            assert_eq!(config.fee_admin, new_admin);
        }
    }

    #[tokio::test]
    async fn test_set_config_secondary_admin_invalid_old_admin() {
        let fixture = TestBuilder::new().await;
        let mut vault_program_client = fixture.vault_program_client();
        let config_address = Config::find_program_address(&jito_vault_program::id()).0;

        // Initialize config
        let _old_admin = vault_program_client.do_initialize_config().await.unwrap();

        // Create invalid old admin and new admin
        let invalid_old_admin = Keypair::new();

        vault_program_client
            .airdrop(&invalid_old_admin.pubkey(), 1.0)
            .await
            .unwrap();

        // Attempt to set new admin with invalid old admin
        {
            // Fee Admin
            let new_admin = Pubkey::new_unique();
            let response = vault_program_client
                .set_config_secondary_admin(
                    &config_address,
                    &invalid_old_admin,
                    &new_admin,
                    ConfigAdminRole::FeeAdmin,
                )
                .await;

            assert_vault_error(response, VaultError::ConfigAdminInvalid);
        }
    }
}
