#[cfg(test)]
mod tests {
    use jito_restaking_core::config::Config;
    use jito_restaking_sdk::error::RestakingError;
    use solana_program::instruction::InstructionError;
    use solana_sdk::signature::{Keypair, Signer};

    use crate::fixtures::{assert_ix_error, fixture::TestBuilder};

    #[tokio::test]
    async fn test_set_config_admin_ok() {
        let mut fixture = TestBuilder::new().await;
        let mut restaking_program_client = fixture.restaking_program_client();

        // Initialize config
        let config_admin = Keypair::new();
        let config = Config::find_program_address(&jito_restaking_program::id()).0;

        fixture
            .transfer(&config_admin.pubkey(), 10.0)
            .await
            .unwrap();

        restaking_program_client
            .initialize_config(&config, &config_admin)
            .await
            .unwrap();

        // Set new admin
        let new_admin = Keypair::new();

        restaking_program_client
            .set_config_admin(&config, &config_admin, &new_admin)
            .await
            .unwrap();

        // Verify new admin
        let updated_config = restaking_program_client.get_config(&config).await.unwrap();
        assert_eq!(updated_config.admin, new_admin.pubkey());
    }

    #[tokio::test]
    async fn test_set_config_admin_invalid_old_admin() {
        let mut fixture = TestBuilder::new().await;
        let mut restaking_program_client = fixture.restaking_program_client();

        // Initialize config
        let config_admin = Keypair::new();
        let config = Config::find_program_address(&jito_restaking_program::id()).0;

        fixture
            .transfer(&config_admin.pubkey(), 10.0)
            .await
            .unwrap();

        restaking_program_client
            .initialize_config(&config, &config_admin)
            .await
            .unwrap();

        // Attempt to set new admin with invalid old admin
        let invalid_admin = Keypair::new();
        let new_admin = Keypair::new();
        fixture
            .transfer(&invalid_admin.pubkey(), 10.0)
            .await
            .unwrap();

        let transaction_error = restaking_program_client
            .set_config_admin(&config, &invalid_admin, &new_admin)
            .await;

        assert_ix_error(
            transaction_error,
            InstructionError::Custom(RestakingError::ConfigAdminInvalid as u32),
        );
    }
}
