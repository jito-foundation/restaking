#[cfg(test)]
mod tests {
    use jito_restaking_core::{config::Config, operator::Operator};
    use solana_program::{instruction::InstructionError, pubkey::Pubkey};
    use solana_sdk::signature::{Keypair, Signer};

    use crate::fixtures::{assert_ix_error, fixture::TestBuilder};

    #[tokio::test]
    async fn test_initialize_operator_ok() {
        let fixture = TestBuilder::new().await;
        let mut restaking_program_client = fixture.restaking_program_client();

        restaking_program_client
            .do_initialize_config()
            .await
            .unwrap();
        let operator_root = restaking_program_client
            .do_initialize_operator()
            .await
            .unwrap();

        let operator = restaking_program_client
            .get_operator(&operator_root.operator_pubkey)
            .await
            .unwrap();
        assert_eq!(operator.admin, operator_root.operator_admin.pubkey());
        assert_eq!(operator.voter, operator_root.operator_admin.pubkey());
        assert_eq!(operator.ncn_admin, operator_root.operator_admin.pubkey());
        assert_eq!(operator.vault_admin, operator_root.operator_admin.pubkey());
        assert_eq!(operator.index(), 0);

        let updated_config = restaking_program_client
            .get_config(&Config::find_program_address(&jito_restaking_program::id()).0)
            .await
            .unwrap();
        assert_eq!(updated_config.operator_count(), 1);
    }

    #[tokio::test]
    async fn test_initialize_operator_bad_derivation_fails() {
        let mut fixture = TestBuilder::new().await;
        let mut restaking_program_client = fixture.restaking_program_client();

        restaking_program_client
            .do_initialize_config()
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

        let transaction_error = restaking_program_client
            .initialize_operator(
                &Config::find_program_address(&jito_restaking_program::id()).0,
                &incorrect_operator_pubkey,
                &operator_admin,
                &operator_base,
                0,
            )
            .await;

        assert_ix_error(transaction_error, InstructionError::InvalidAccountData);
    }

    #[tokio::test]
    async fn test_initialize_operator_already_initialized_fails() {
        let mut fixture = TestBuilder::new().await;
        let mut restaking_program_client = fixture.restaking_program_client();

        restaking_program_client
            .do_initialize_config()
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

        let config_pubkey = Config::find_program_address(&jito_restaking_program::id()).0;
        restaking_program_client
            .initialize_operator(
                &config_pubkey,
                &operator_pubkey,
                &operator_admin,
                &operator_base,
                0,
            )
            .await
            .unwrap();

        // get new blockhash
        fixture.warp_slot_incremental(1).await.unwrap();

        // Try to initialize the same Operator again
        let transaction_error = restaking_program_client
            .initialize_operator(
                &config_pubkey,
                &operator_pubkey,
                &operator_admin,
                &operator_base,
                0,
            )
            .await;

        assert_ix_error(transaction_error, InstructionError::InvalidAccountOwner);
    }

    #[tokio::test]
    async fn test_initialize_operator_with_uninitialized_config_fails() {
        let fixture = TestBuilder::new().await;
        let mut restaking_program_client = fixture.restaking_program_client();

        let transaction_error = restaking_program_client.do_initialize_operator().await;

        assert_ix_error(transaction_error, InstructionError::InvalidAccountOwner);
    }

    #[tokio::test]
    async fn test_initialize_multiple_operators_ok() {
        let fixture = TestBuilder::new().await;
        let mut restaking_program_client = fixture.restaking_program_client();

        restaking_program_client
            .do_initialize_config()
            .await
            .unwrap();

        let operator_root1 = restaking_program_client
            .do_initialize_operator()
            .await
            .unwrap();
        let operator_root2 = restaking_program_client
            .do_initialize_operator()
            .await
            .unwrap();

        // Verify operator accounts
        let operator1 = restaking_program_client
            .get_operator(&operator_root1.operator_pubkey)
            .await
            .unwrap();
        let operator2 = restaking_program_client
            .get_operator(&operator_root2.operator_pubkey)
            .await
            .unwrap();

        assert_eq!(operator1.index(), 0);
        assert_eq!(operator2.index(), 1);

        // Verify config update
        let updated_config = restaking_program_client
            .get_config(&Config::find_program_address(&jito_restaking_program::id()).0)
            .await
            .unwrap();
        assert_eq!(updated_config.operator_count(), 2);
    }
}
