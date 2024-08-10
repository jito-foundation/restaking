#[cfg(test)]
mod tests {
    use jito_restaking_core::{
        config::Config, ncn::Ncn, operator::Operator, operator_ncn_ticket::OperatorNcnTicket,
    };
    use solana_sdk::signature::{Keypair, Signer};

    use crate::fixtures::fixture::TestBuilder;

    #[tokio::test]
    async fn test_operator_add_ncn_ok() {
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

        // Initialize operator
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

        // Operator adds NCN
        let payer = Keypair::new();
        fixture.transfer(&payer.pubkey(), 10.0).await.unwrap();
        let operator_ncn_ticket = OperatorNcnTicket::find_program_address(
            &jito_restaking_program::id(),
            &operator_pubkey,
            &ncn_pubkey,
        )
        .0;

        restaking_program_client
            .operator_add_ncn(
                &config,
                &operator_pubkey,
                &ncn_pubkey,
                &operator_ncn_ticket,
                &operator_admin,
                &payer,
            )
            .await
            .unwrap();

        // Verify operator state
        let operator = restaking_program_client
            .get_operator(&operator_pubkey)
            .await
            .unwrap();
        assert_eq!(operator.ncn_count(), 1);

        // Verify operator NCN ticket
        let ticket = restaking_program_client
            .get_operator_ncn_ticket(&operator_pubkey, &ncn_pubkey)
            .await
            .unwrap();
        assert_eq!(ticket.operator, operator_pubkey);
        assert_eq!(ticket.ncn(), ncn_pubkey);
        assert_eq!(ticket.index(), 0);
        assert_eq!(ticket.state().slot_added(), 1);
    }

    #[tokio::test]
    async fn test_operator_add_multiple_ncn_ok() {
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

        // Initialize operator
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

        // Initialize two NCNs
        let ncn_admin1 = Keypair::new();
        let ncn_base1 = Keypair::new();
        fixture.transfer(&ncn_admin1.pubkey(), 10.0).await.unwrap();
        let ncn_pubkey1 =
            Ncn::find_program_address(&jito_restaking_program::id(), &ncn_base1.pubkey()).0;
        restaking_program_client
            .initialize_ncn(&config, &ncn_pubkey1, &ncn_admin1, &ncn_base1)
            .await
            .unwrap();

        let ncn_admin2 = Keypair::new();
        let ncn_base2 = Keypair::new();
        fixture.transfer(&ncn_admin2.pubkey(), 10.0).await.unwrap();
        let ncn_pubkey2 =
            Ncn::find_program_address(&jito_restaking_program::id(), &ncn_base2.pubkey()).0;
        restaking_program_client
            .initialize_ncn(&config, &ncn_pubkey2, &ncn_admin2, &ncn_base2)
            .await
            .unwrap();

        let payer = Keypair::new();
        fixture.transfer(&payer.pubkey(), 10.0).await.unwrap();

        // Operator adds first NCN
        let operator_ncn_ticket1 = OperatorNcnTicket::find_program_address(
            &jito_restaking_program::id(),
            &operator_pubkey,
            &ncn_pubkey1,
        )
        .0;

        restaking_program_client
            .operator_add_ncn(
                &config,
                &operator_pubkey,
                &ncn_pubkey1,
                &operator_ncn_ticket1,
                &operator_admin,
                &payer,
            )
            .await
            .unwrap();

        // Operator adds second NCN
        let operator_ncn_ticket2 = OperatorNcnTicket::find_program_address(
            &jito_restaking_program::id(),
            &operator_pubkey,
            &ncn_pubkey2,
        )
        .0;

        restaking_program_client
            .operator_add_ncn(
                &config,
                &operator_pubkey,
                &ncn_pubkey2,
                &operator_ncn_ticket2,
                &operator_admin,
                &payer,
            )
            .await
            .unwrap();

        // Verify operator state
        let operator = restaking_program_client
            .get_operator(&operator_pubkey)
            .await
            .unwrap();
        assert_eq!(operator.ncn_count(), 2);

        // Verify operator NCN tickets
        let ticket1 = restaking_program_client
            .get_operator_ncn_ticket(&operator_pubkey, &ncn_pubkey1)
            .await
            .unwrap();
        assert_eq!(ticket1.operator, operator_pubkey);
        assert_eq!(ticket1.ncn(), ncn_pubkey1);
        assert_eq!(ticket1.index(), 0);

        let ticket2 = restaking_program_client
            .get_operator_ncn_ticket(&operator_pubkey, &ncn_pubkey2)
            .await
            .unwrap();
        assert_eq!(ticket2.operator, operator_pubkey);
        assert_eq!(ticket2.ncn(), ncn_pubkey2);
        assert_eq!(ticket2.index(), 1);
    }

    #[tokio::test]
    async fn test_operator_add_ncn_duplicate_fails() {
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

        // Initialize operator
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

        let payer = Keypair::new();
        fixture.transfer(&payer.pubkey(), 10.0).await.unwrap();

        let operator_ncn_ticket = OperatorNcnTicket::find_program_address(
            &jito_restaking_program::id(),
            &operator_pubkey,
            &ncn_pubkey,
        )
        .0;

        // Operator adds NCN for the first time
        restaking_program_client
            .operator_add_ncn(
                &config,
                &operator_pubkey,
                &ncn_pubkey,
                &operator_ncn_ticket,
                &operator_admin,
                &payer,
            )
            .await
            .unwrap();

        // Attempt to add the same NCN again
        let result = restaking_program_client
            .operator_add_ncn(
                &config,
                &operator_pubkey,
                &ncn_pubkey,
                &operator_ncn_ticket,
                &operator_admin,
                &payer,
            )
            .await;

        // TODO (LB): check for specific error
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_operator_add_ncn_non_admin_fails() {
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

        // Initialize operator
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

        let payer = Keypair::new();
        fixture.transfer(&payer.pubkey(), 10.0).await.unwrap();

        let operator_ncn_ticket = OperatorNcnTicket::find_program_address(
            &jito_restaking_program::id(),
            &operator_pubkey,
            &ncn_pubkey,
        )
        .0;

        // Attempt to add NCN with non-admin signer
        let non_admin = Keypair::new();
        fixture.transfer(&non_admin.pubkey(), 10.0).await.unwrap();

        let result = restaking_program_client
            .operator_add_ncn(
                &config,
                &operator_pubkey,
                &ncn_pubkey,
                &operator_ncn_ticket,
                &non_admin,
                &payer,
            )
            .await;

        // TODO (LB): check for specific error
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_operator_add_ncn_uninitialized_operator_fails() {
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

        let payer = Keypair::new();
        fixture.transfer(&payer.pubkey(), 10.0).await.unwrap();

        // Use uninitialized operator
        let uninitialized_operator = Keypair::new();
        let uninitialized_operator_pubkey = Operator::find_program_address(
            &jito_restaking_program::id(),
            &uninitialized_operator.pubkey(),
        )
        .0;

        let operator_ncn_ticket = OperatorNcnTicket::find_program_address(
            &jito_restaking_program::id(),
            &uninitialized_operator_pubkey,
            &ncn_pubkey,
        )
        .0;

        let result = restaking_program_client
            .operator_add_ncn(
                &config,
                &uninitialized_operator_pubkey,
                &ncn_pubkey,
                &operator_ncn_ticket,
                &uninitialized_operator,
                &payer,
            )
            .await;

        // TODO (LB): check for specific error
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_operator_add_ncn_uninitialized_ncn_fails() {
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

        // Initialize operator
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

        let payer = Keypair::new();
        fixture.transfer(&payer.pubkey(), 10.0).await.unwrap();

        // Use uninitialized NCN
        let uninitialized_ncn = Keypair::new();
        let uninitialized_ncn_pubkey =
            Ncn::find_program_address(&jito_restaking_program::id(), &uninitialized_ncn.pubkey()).0;

        let operator_ncn_ticket = OperatorNcnTicket::find_program_address(
            &jito_restaking_program::id(),
            &operator_pubkey,
            &uninitialized_ncn_pubkey,
        )
        .0;

        let result = restaking_program_client
            .operator_add_ncn(
                &config,
                &operator_pubkey,
                &uninitialized_ncn_pubkey,
                &operator_ncn_ticket,
                &operator_admin,
                &payer,
            )
            .await;

        // TODO (LB): check for specific error
        assert!(result.is_err());
    }
}
