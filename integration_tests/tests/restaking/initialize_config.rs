#[cfg(test)]
mod tests {
    use jito_restaking_core::config::Config;
    use solana_program::pubkey::Pubkey;
    use solana_sdk::signature::{Keypair, Signer};

    use crate::fixtures::fixture::TestBuilder;

    #[tokio::test]
    async fn test_initialize_config_ok() {
        let mut fixture = TestBuilder::new().await;

        let mut restaking_program_client = fixture.restaking_program_client();

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

        let config = restaking_program_client.get_config(&config).await.unwrap();
        assert_eq!(config.admin(), config_admin.pubkey());
        assert_eq!(config.vault_program(), jito_vault_program::id());
        assert_eq!(config.avs_count(), 0);
        assert_eq!(config.operators_count(), 0);
    }

    #[tokio::test]
    async fn test_initialize_config_double_init_fails() {
        let mut fixture = TestBuilder::new().await;
        let mut restaking_program_client = fixture.restaking_program_client();
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

        let result = restaking_program_client
            .initialize_config(&config, &config_admin)
            .await;

        // TODO (LB): check specific error here
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_initialize_config_bad_pda_fails() {
        let mut fixture = TestBuilder::new().await;
        let mut restaking_program_client = fixture.restaking_program_client();
        let config_admin = Keypair::new();

        fixture
            .transfer(&config_admin.pubkey(), 10.0)
            .await
            .unwrap();

        let result = restaking_program_client
            .initialize_config(&Pubkey::new_unique(), &config_admin)
            .await;

        // TODO (LB): check specific error
        assert!(result.is_err());
    }
}
