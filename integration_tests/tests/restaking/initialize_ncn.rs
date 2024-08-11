#[cfg(test)]
mod tests {
    use jito_restaking_core::{config::Config, ncn::Ncn};
    use solana_program::pubkey::Pubkey;
    use solana_sdk::signature::{Keypair, Signer};

    use crate::fixtures::fixture::TestBuilder;

    #[tokio::test]
    async fn test_initialize_ncn_ok() {
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

        // Initialize NCN
        let ncn_admin = Keypair::new();
        let ncn_base = Keypair::new();
        fixture.transfer(&ncn_admin.pubkey(), 10.0).await.unwrap();

        let ncn_pubkey =
            Ncn::find_program_address(&jito_restaking_program::id(), &ncn_base.pubkey()).0;

        restaking_program_client
            .initialize_ncn(&config, &ncn_pubkey, &ncn_admin, &ncn_base)
            .await
            .unwrap();

        // Verify NCN account
        let ncn = restaking_program_client.get_ncn(&ncn_pubkey).await.unwrap();
        assert_eq!(ncn.base, ncn_base.pubkey());
        assert_eq!(ncn.admin, ncn_admin.pubkey());
        assert_eq!(ncn.operator_admin, ncn_admin.pubkey());
        assert_eq!(ncn.vault_admin, ncn_admin.pubkey());
        assert_eq!(ncn.slasher_admin, ncn_admin.pubkey());
        assert_eq!(ncn.withdraw_admin, ncn_admin.pubkey());
        assert_eq!(ncn.index, 0);
        assert_eq!(ncn.operator_count, 0);
        assert_eq!(ncn.slasher_count, 0);
        assert_eq!(ncn.vault_count, 0);

        let updated_config = restaking_program_client.get_config(&config).await.unwrap();
        assert_eq!(updated_config.ncn_count, 1);
    }

    #[tokio::test]
    async fn test_initialize_ncn_bad_derivation_fails() {
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

        // Try to initialize NCN with incorrect derivation
        let ncn_admin = Keypair::new();
        let ncn_base = Keypair::new();
        fixture.transfer(&ncn_admin.pubkey(), 10.0).await.unwrap();

        let incorrect_ncn_pubkey = Pubkey::new_unique(); // This is not derived correctly

        let result = restaking_program_client
            .initialize_ncn(&config, &incorrect_ncn_pubkey, &ncn_admin, &ncn_base)
            .await;

        // TODO (LB): check for specific error
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_initialize_ncn_already_initialized_fails() {
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

        // Initialize NCN
        let ncn_admin = Keypair::new();
        let ncn_base = Keypair::new();
        fixture.transfer(&ncn_admin.pubkey(), 10.0).await.unwrap();

        let ncn_pubkey =
            Ncn::find_program_address(&jito_restaking_program::id(), &ncn_base.pubkey()).0;

        restaking_program_client
            .initialize_ncn(&config, &ncn_pubkey, &ncn_admin, &ncn_base)
            .await
            .unwrap();

        let result = restaking_program_client
            .initialize_ncn(&config, &ncn_pubkey, &ncn_admin, &ncn_base)
            .await;

        // TODO (LB): check for specific error
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_initialize_ncn_with_uninitialized_config_fails() {
        let mut fixture = TestBuilder::new().await;
        let mut restaking_program_client = fixture.restaking_program_client();

        // Try to initialize NCN without initializing config first
        let ncn_admin = Keypair::new();
        let ncn_base = Keypair::new();
        fixture.transfer(&ncn_admin.pubkey(), 10.0).await.unwrap();

        let config = Config::find_program_address(&jito_restaking_program::id()).0;
        let ncn_pubkey =
            Ncn::find_program_address(&jito_restaking_program::id(), &ncn_base.pubkey()).0;

        let result = restaking_program_client
            .initialize_ncn(&config, &ncn_pubkey, &ncn_admin, &ncn_base)
            .await;

        // TODO (LB): check for specific error
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_initialize_multiple_ncn_ok() {
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

        // Initialize first NCN
        let ncn_admin1 = Keypair::new();
        let ncn_base1 = Keypair::new();
        fixture.transfer(&ncn_admin1.pubkey(), 10.0).await.unwrap();
        let ncn_pubkey1 =
            Ncn::find_program_address(&jito_restaking_program::id(), &ncn_base1.pubkey()).0;

        restaking_program_client
            .initialize_ncn(&config, &ncn_pubkey1, &ncn_admin1, &ncn_base1)
            .await
            .unwrap();

        // Initialize second NCN
        let ncn_admin2 = Keypair::new();
        let ncn_base2 = Keypair::new();
        fixture.transfer(&ncn_admin2.pubkey(), 10.0).await.unwrap();
        let ncn_pubkey2 =
            Ncn::find_program_address(&jito_restaking_program::id(), &ncn_base2.pubkey()).0;

        restaking_program_client
            .initialize_ncn(&config, &ncn_pubkey2, &ncn_admin2, &ncn_base2)
            .await
            .unwrap();

        // Verify NCN accounts
        let ncn1 = restaking_program_client
            .get_ncn(&ncn_pubkey1)
            .await
            .unwrap();
        let ncn2 = restaking_program_client
            .get_ncn(&ncn_pubkey2)
            .await
            .unwrap();

        assert_eq!(ncn1.index, 0);
        assert_eq!(ncn2.index, 1);

        // Verify config update
        let updated_config = restaking_program_client.get_config(&config).await.unwrap();
        assert_eq!(updated_config.ncn_count, 2);
    }
}
