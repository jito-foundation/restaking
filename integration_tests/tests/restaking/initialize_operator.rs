#[cfg(test)]
mod tests {
    use jito_restaking_core::{config::Config, operator::Operator};
    use solana_program::pubkey::Pubkey;
    use solana_sdk::signature::{Keypair, Signer};

    use crate::fixtures::fixture::TestBuilder;

    #[tokio::test]
    async fn test_initialize_operator_ok() {
        let mut fixture = TestBuilder::new().await;
        let mut restaking_program_client = fixture.restaking_program_client();

        // Initialize config first
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

        // Initialize Operator
        let operator_admin = Keypair::new();
        let operator_base = Keypair::new();
        fixture
            .transfer(&operator_admin.pubkey(), 10.0)
            .await
            .unwrap();

        let operator_pubkey =
            Operator::find_program_address(&jito_restaking_program::id(), &operator_base.pubkey())
                .0;

        restaking_program_client
            .initialize_operator(&config, &operator_pubkey, &operator_admin, &operator_base)
            .await
            .unwrap();

        let operator = restaking_program_client
            .get_operator(&operator_pubkey)
            .await
            .unwrap();
        assert_eq!(operator.base, operator_base.pubkey());
        assert_eq!(operator.admin, operator_admin.pubkey());
        assert_eq!(operator.voter, operator_admin.pubkey());
        assert_eq!(operator.ncn_admin, operator_admin.pubkey());
        assert_eq!(operator.vault_admin, operator_admin.pubkey());
        assert_eq!(operator.index, 0);

        let updated_config = restaking_program_client.get_config(&config).await.unwrap();
        assert_eq!(updated_config.operator_count, 1);
    }

    #[tokio::test]
    async fn test_initialize_operator_bad_derivation_fails() {
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

        // Try to initialize Operator with incorrect derivation
        let operator_admin = Keypair::new();
        let operator_base = Keypair::new();
        fixture
            .transfer(&operator_admin.pubkey(), 10.0)
            .await
            .unwrap();

        let incorrect_operator_pubkey = Pubkey::new_unique(); // This is not derived correctly

        let result = restaking_program_client
            .initialize_operator(
                &config,
                &incorrect_operator_pubkey,
                &operator_admin,
                &operator_base,
            )
            .await;

        // TODO (LB): check specific error
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_initialize_operator_already_initialized_fails() {
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

        // Initialize Operator
        let operator_admin = Keypair::new();
        let operator_base = Keypair::new();
        fixture
            .transfer(&operator_admin.pubkey(), 10.0)
            .await
            .unwrap();

        let operator_pubkey =
            Operator::find_program_address(&jito_restaking_program::id(), &operator_base.pubkey())
                .0;

        restaking_program_client
            .initialize_operator(&config, &operator_pubkey, &operator_admin, &operator_base)
            .await
            .unwrap();

        // Try to initialize the same Operator again
        let result = restaking_program_client
            .initialize_operator(&config, &operator_pubkey, &operator_admin, &operator_base)
            .await;

        // TODO (LB): check specific error
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_initialize_operator_with_uninitialized_config_fails() {
        let mut fixture = TestBuilder::new().await;
        let mut restaking_program_client = fixture.restaking_program_client();

        // Try to initialize Operator without initializing config first
        let operator_admin = Keypair::new();
        let operator_base = Keypair::new();
        fixture
            .transfer(&operator_admin.pubkey(), 10.0)
            .await
            .unwrap();

        let config = Config::find_program_address(&jito_restaking_program::id()).0;
        let operator_pubkey =
            Operator::find_program_address(&jito_restaking_program::id(), &operator_base.pubkey())
                .0;

        let result = restaking_program_client
            .initialize_operator(&config, &operator_pubkey, &operator_admin, &operator_base)
            .await;

        // TODO (LB): check specific error
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_initialize_multiple_operators_ok() {
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

        // Initialize first operator
        let operator_admin1 = Keypair::new();
        let operator_base1 = Keypair::new();
        fixture
            .transfer(&operator_admin1.pubkey(), 10.0)
            .await
            .unwrap();
        let operator_pubkey1 =
            Operator::find_program_address(&jito_restaking_program::id(), &operator_base1.pubkey())
                .0;

        restaking_program_client
            .initialize_operator(
                &config,
                &operator_pubkey1,
                &operator_admin1,
                &operator_base1,
            )
            .await
            .unwrap();

        // Initialize second operator
        let operator_admin2 = Keypair::new();
        let operator_base2 = Keypair::new();
        fixture
            .transfer(&operator_admin2.pubkey(), 10.0)
            .await
            .unwrap();
        let operator_pubkey2 =
            Operator::find_program_address(&jito_restaking_program::id(), &operator_base2.pubkey())
                .0;

        restaking_program_client
            .initialize_operator(
                &config,
                &operator_pubkey2,
                &operator_admin2,
                &operator_base2,
            )
            .await
            .unwrap();

        // Verify operator accounts
        let operator1 = restaking_program_client
            .get_operator(&operator_pubkey1)
            .await
            .unwrap();
        let operator2 = restaking_program_client
            .get_operator(&operator_pubkey2)
            .await
            .unwrap();

        assert_eq!(operator1.index, 0);
        assert_eq!(operator2.index, 1);

        // Verify config update
        let updated_config = restaking_program_client.get_config(&config).await.unwrap();
        assert_eq!(updated_config.operator_count, 2);
    }
}
