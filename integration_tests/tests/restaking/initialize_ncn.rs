#[cfg(test)]
mod tests {
    use jito_restaking_core::{config::Config, ncn::Ncn};
    use solana_program::{instruction::InstructionError, pubkey::Pubkey};
    use solana_sdk::signature::{Keypair, Signer};

    use crate::fixtures::{assert_ix_error, fixture::TestBuilder};

    #[tokio::test]
    async fn test_initialize_ncn_ok() {
        let fixture = TestBuilder::new().await;
        let mut restaking_program_client = fixture.restaking_program_client();

        restaking_program_client
            .do_initialize_config()
            .await
            .unwrap();

        let ncn_root = restaking_program_client.do_initialize_ncn().await.unwrap();

        // Verify NCN account
        let ncn = restaking_program_client
            .get_ncn(&ncn_root.ncn_pubkey)
            .await
            .unwrap();
        assert_eq!(ncn.admin, ncn_root.ncn_admin.pubkey());
        assert_eq!(ncn.operator_admin, ncn_root.ncn_admin.pubkey());
        assert_eq!(ncn.vault_admin, ncn_root.ncn_admin.pubkey());
        assert_eq!(ncn.slasher_admin, ncn_root.ncn_admin.pubkey());
        assert_eq!(ncn.withdraw_admin, ncn_root.ncn_admin.pubkey());
        assert_eq!(ncn.index(), 0);
        assert_eq!(ncn.operator_count(), 0);
        assert_eq!(ncn.slasher_count(), 0);
        assert_eq!(ncn.vault_count(), 0);

        let updated_config = restaking_program_client
            .get_config(&Config::find_program_address(&jito_restaking_program::id()).0)
            .await
            .unwrap();
        assert_eq!(updated_config.ncn_count(), 1);
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

        assert_ix_error(result, InstructionError::InvalidAccountData);
    }

    #[tokio::test]
    async fn test_initialize_ncn_already_initialized_fails() {
        let mut fixture = TestBuilder::new().await;
        let mut restaking_program_client = fixture.restaking_program_client();

        restaking_program_client
            .do_initialize_config()
            .await
            .unwrap();

        // Initialize NCN
        let ncn_admin = Keypair::new();
        let ncn_base = Keypair::new();
        fixture.transfer(&ncn_admin.pubkey(), 10.0).await.unwrap();

        let ncn_pubkey =
            Ncn::find_program_address(&jito_restaking_program::id(), &ncn_base.pubkey()).0;

        let config_pubkey = Config::find_program_address(&jito_restaking_program::id()).0;
        restaking_program_client
            .initialize_ncn(&config_pubkey, &ncn_pubkey, &ncn_admin, &ncn_base)
            .await
            .unwrap();

        // get new blockhash
        fixture.warp_slot_incremental(1).await.unwrap();

        let transaction_error = restaking_program_client
            .initialize_ncn(&config_pubkey, &ncn_pubkey, &ncn_admin, &ncn_base)
            .await;

        // expected ncn is system program during initialization
        assert_ix_error(transaction_error, InstructionError::InvalidAccountOwner);
    }

    #[tokio::test]
    async fn test_initialize_ncn_no_config_fails() {
        let fixture = TestBuilder::new().await;
        let mut restaking_program_client = fixture.restaking_program_client();

        let ncn_root = restaking_program_client.do_initialize_ncn().await;

        // config isn't initialized, so owned by system program
        assert_ix_error(ncn_root, InstructionError::InvalidAccountOwner);
    }

    #[tokio::test]
    async fn test_initialize_multiple_ncn_ok() {
        let fixture = TestBuilder::new().await;
        let mut restaking_program_client = fixture.restaking_program_client();

        restaking_program_client
            .do_initialize_config()
            .await
            .unwrap();

        let ncn_root_1 = restaking_program_client.do_initialize_ncn().await.unwrap();
        let ncn_root_2 = restaking_program_client.do_initialize_ncn().await.unwrap();

        // Verify NCN accounts
        let ncn1 = restaking_program_client
            .get_ncn(&ncn_root_1.ncn_pubkey)
            .await
            .unwrap();
        let ncn2 = restaking_program_client
            .get_ncn(&ncn_root_2.ncn_pubkey)
            .await
            .unwrap();

        assert_eq!(ncn1.index(), 0);
        assert_eq!(ncn2.index(), 1);

        // Verify config update
        let updated_config = restaking_program_client
            .get_config(&Config::find_program_address(&jito_restaking_program::id()).0)
            .await
            .unwrap();
        assert_eq!(updated_config.ncn_count(), 2);
    }
}
