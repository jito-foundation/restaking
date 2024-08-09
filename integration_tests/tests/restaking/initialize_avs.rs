#[cfg(test)]
mod tests {
    use jito_restaking_core::{avs::Avs, config::Config};
    use solana_program::pubkey::Pubkey;
    use solana_sdk::signature::{Keypair, Signer};

    use crate::fixtures::fixture::TestBuilder;

    #[tokio::test]
    async fn test_initialize_avs_ok() {
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

        // Initialize AVS
        let avs_admin = Keypair::new();
        let avs_base = Keypair::new();
        fixture.transfer(&avs_admin.pubkey(), 10.0).await.unwrap();

        let avs_pubkey =
            Avs::find_program_address(&jito_restaking_program::id(), &avs_base.pubkey()).0;

        restaking_program_client
            .initialize_avs(&config, &avs_pubkey, &avs_admin, &avs_base)
            .await
            .unwrap();

        // Verify AVS account
        let avs = restaking_program_client.get_avs(&avs_pubkey).await.unwrap();
        assert_eq!(avs.base(), avs_base.pubkey());
        assert_eq!(avs.admin(), avs_admin.pubkey());
        assert_eq!(avs.operator_admin(), avs_admin.pubkey());
        assert_eq!(avs.vault_admin(), avs_admin.pubkey());
        assert_eq!(avs.slasher_admin(), avs_admin.pubkey());
        assert_eq!(avs.withdraw_admin(), avs_admin.pubkey());
        assert_eq!(avs.index(), 0);
        assert_eq!(avs.operator_count(), 0);
        assert_eq!(avs.slasher_count(), 0);
        assert_eq!(avs.vault_count(), 0);

        let updated_config = restaking_program_client.get_config(&config).await.unwrap();
        assert_eq!(updated_config.avs_count(), 1);
    }

    #[tokio::test]
    async fn test_initialize_avs_bad_derivation_fails() {
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

        // Try to initialize AVS with incorrect derivation
        let avs_admin = Keypair::new();
        let avs_base = Keypair::new();
        fixture.transfer(&avs_admin.pubkey(), 10.0).await.unwrap();

        let incorrect_avs_pubkey = Pubkey::new_unique(); // This is not derived correctly

        let result = restaking_program_client
            .initialize_avs(&config, &incorrect_avs_pubkey, &avs_admin, &avs_base)
            .await;

        // TODO (LB): check for specific error
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_initialize_avs_already_initialized_fails() {
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

        // Initialize AVS
        let avs_admin = Keypair::new();
        let avs_base = Keypair::new();
        fixture.transfer(&avs_admin.pubkey(), 10.0).await.unwrap();

        let avs_pubkey =
            Avs::find_program_address(&jito_restaking_program::id(), &avs_base.pubkey()).0;

        restaking_program_client
            .initialize_avs(&config, &avs_pubkey, &avs_admin, &avs_base)
            .await
            .unwrap();

        let result = restaking_program_client
            .initialize_avs(&config, &avs_pubkey, &avs_admin, &avs_base)
            .await;

        // TODO (LB): check for specific error
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_initialize_avs_with_uninitialized_config_fails() {
        let mut fixture = TestBuilder::new().await;
        let mut restaking_program_client = fixture.restaking_program_client();

        // Try to initialize AVS without initializing config first
        let avs_admin = Keypair::new();
        let avs_base = Keypair::new();
        fixture.transfer(&avs_admin.pubkey(), 10.0).await.unwrap();

        let config = Config::find_program_address(&jito_restaking_program::id()).0;
        let avs_pubkey =
            Avs::find_program_address(&jito_restaking_program::id(), &avs_base.pubkey()).0;

        let result = restaking_program_client
            .initialize_avs(&config, &avs_pubkey, &avs_admin, &avs_base)
            .await;

        // TODO (LB): check for specific error
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_initialize_multiple_avs_ok() {
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

        // Initialize first AVS
        let avs_admin1 = Keypair::new();
        let avs_base1 = Keypair::new();
        fixture.transfer(&avs_admin1.pubkey(), 10.0).await.unwrap();
        let avs_pubkey1 =
            Avs::find_program_address(&jito_restaking_program::id(), &avs_base1.pubkey()).0;

        restaking_program_client
            .initialize_avs(&config, &avs_pubkey1, &avs_admin1, &avs_base1)
            .await
            .unwrap();

        // Initialize second AVS
        let avs_admin2 = Keypair::new();
        let avs_base2 = Keypair::new();
        fixture.transfer(&avs_admin2.pubkey(), 10.0).await.unwrap();
        let avs_pubkey2 =
            Avs::find_program_address(&jito_restaking_program::id(), &avs_base2.pubkey()).0;

        restaking_program_client
            .initialize_avs(&config, &avs_pubkey2, &avs_admin2, &avs_base2)
            .await
            .unwrap();

        // Verify AVS accounts
        let avs1 = restaking_program_client
            .get_avs(&avs_pubkey1)
            .await
            .unwrap();
        let avs2 = restaking_program_client
            .get_avs(&avs_pubkey2)
            .await
            .unwrap();

        assert_eq!(avs1.index(), 0);
        assert_eq!(avs2.index(), 1);

        // Verify config update
        let updated_config = restaking_program_client.get_config(&config).await.unwrap();
        assert_eq!(updated_config.avs_count(), 2);
    }
}
