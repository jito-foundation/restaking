#[cfg(test)]
mod tests {
    use jito_restaking_core::{
        avs::Avs, config::Config, operator::Operator, operator_avs_ticket::OperatorAvsTicket,
    };
    use solana_sdk::signature::{Keypair, Signer};

    use crate::fixtures::fixture::TestBuilder;

    #[tokio::test]
    async fn test_operator_add_avs_ok() {
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

        // Operator adds AVS
        let payer = Keypair::new();
        fixture.transfer(&payer.pubkey(), 10.0).await.unwrap();
        let operator_avs_ticket = OperatorAvsTicket::find_program_address(
            &jito_restaking_program::id(),
            &operator_pubkey,
            &avs_pubkey,
        )
        .0;

        restaking_program_client
            .operator_add_avs(
                &config,
                &operator_pubkey,
                &avs_pubkey,
                &operator_avs_ticket,
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
        assert_eq!(operator.avs_count(), 1);

        // Verify operator AVS ticket
        let ticket = restaking_program_client
            .get_operator_avs_ticket(&operator_pubkey, &avs_pubkey)
            .await
            .unwrap();
        assert_eq!(ticket.operator(), operator_pubkey);
        assert_eq!(ticket.avs(), avs_pubkey);
        assert_eq!(ticket.index(), 0);
        assert_eq!(ticket.state().slot_added(), 1);
    }

    #[tokio::test]
    async fn test_operator_add_multiple_avs_ok() {
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

        // Initialize two AVSs
        let avs_admin1 = Keypair::new();
        let avs_base1 = Keypair::new();
        fixture.transfer(&avs_admin1.pubkey(), 10.0).await.unwrap();
        let avs_pubkey1 =
            Avs::find_program_address(&jito_restaking_program::id(), &avs_base1.pubkey()).0;
        restaking_program_client
            .initialize_avs(&config, &avs_pubkey1, &avs_admin1, &avs_base1)
            .await
            .unwrap();

        let avs_admin2 = Keypair::new();
        let avs_base2 = Keypair::new();
        fixture.transfer(&avs_admin2.pubkey(), 10.0).await.unwrap();
        let avs_pubkey2 =
            Avs::find_program_address(&jito_restaking_program::id(), &avs_base2.pubkey()).0;
        restaking_program_client
            .initialize_avs(&config, &avs_pubkey2, &avs_admin2, &avs_base2)
            .await
            .unwrap();

        let payer = Keypair::new();
        fixture.transfer(&payer.pubkey(), 10.0).await.unwrap();

        // Operator adds first AVS
        let operator_avs_ticket1 = OperatorAvsTicket::find_program_address(
            &jito_restaking_program::id(),
            &operator_pubkey,
            &avs_pubkey1,
        )
        .0;

        restaking_program_client
            .operator_add_avs(
                &config,
                &operator_pubkey,
                &avs_pubkey1,
                &operator_avs_ticket1,
                &operator_admin,
                &payer,
            )
            .await
            .unwrap();

        // Operator adds second AVS
        let operator_avs_ticket2 = OperatorAvsTicket::find_program_address(
            &jito_restaking_program::id(),
            &operator_pubkey,
            &avs_pubkey2,
        )
        .0;

        restaking_program_client
            .operator_add_avs(
                &config,
                &operator_pubkey,
                &avs_pubkey2,
                &operator_avs_ticket2,
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
        assert_eq!(operator.avs_count(), 2);

        // Verify operator AVS tickets
        let ticket1 = restaking_program_client
            .get_operator_avs_ticket(&operator_pubkey, &avs_pubkey1)
            .await
            .unwrap();
        assert_eq!(ticket1.operator(), operator_pubkey);
        assert_eq!(ticket1.avs(), avs_pubkey1);
        assert_eq!(ticket1.index(), 0);

        let ticket2 = restaking_program_client
            .get_operator_avs_ticket(&operator_pubkey, &avs_pubkey2)
            .await
            .unwrap();
        assert_eq!(ticket2.operator(), operator_pubkey);
        assert_eq!(ticket2.avs(), avs_pubkey2);
        assert_eq!(ticket2.index(), 1);
    }

    #[tokio::test]
    async fn test_operator_add_avs_duplicate_fails() {
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

        let payer = Keypair::new();
        fixture.transfer(&payer.pubkey(), 10.0).await.unwrap();

        let operator_avs_ticket = OperatorAvsTicket::find_program_address(
            &jito_restaking_program::id(),
            &operator_pubkey,
            &avs_pubkey,
        )
        .0;

        // Operator adds AVS for the first time
        restaking_program_client
            .operator_add_avs(
                &config,
                &operator_pubkey,
                &avs_pubkey,
                &operator_avs_ticket,
                &operator_admin,
                &payer,
            )
            .await
            .unwrap();

        // Attempt to add the same AVS again
        let result = restaking_program_client
            .operator_add_avs(
                &config,
                &operator_pubkey,
                &avs_pubkey,
                &operator_avs_ticket,
                &operator_admin,
                &payer,
            )
            .await;

        // TODO (LB): check for specific error
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_operator_add_avs_non_admin_fails() {
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

        let payer = Keypair::new();
        fixture.transfer(&payer.pubkey(), 10.0).await.unwrap();

        let operator_avs_ticket = OperatorAvsTicket::find_program_address(
            &jito_restaking_program::id(),
            &operator_pubkey,
            &avs_pubkey,
        )
        .0;

        // Attempt to add AVS with non-admin signer
        let non_admin = Keypair::new();
        fixture.transfer(&non_admin.pubkey(), 10.0).await.unwrap();

        let result = restaking_program_client
            .operator_add_avs(
                &config,
                &operator_pubkey,
                &avs_pubkey,
                &operator_avs_ticket,
                &non_admin,
                &payer,
            )
            .await;

        // TODO (LB): check for specific error
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_operator_add_avs_uninitialized_operator_fails() {
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

        let payer = Keypair::new();
        fixture.transfer(&payer.pubkey(), 10.0).await.unwrap();

        // Use uninitialized operator
        let uninitialized_operator = Keypair::new();
        let uninitialized_operator_pubkey = Operator::find_program_address(
            &jito_restaking_program::id(),
            &uninitialized_operator.pubkey(),
        )
        .0;

        let operator_avs_ticket = OperatorAvsTicket::find_program_address(
            &jito_restaking_program::id(),
            &uninitialized_operator_pubkey,
            &avs_pubkey,
        )
        .0;

        let result = restaking_program_client
            .operator_add_avs(
                &config,
                &uninitialized_operator_pubkey,
                &avs_pubkey,
                &operator_avs_ticket,
                &uninitialized_operator,
                &payer,
            )
            .await;

        // TODO (LB): check for specific error
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_operator_add_avs_uninitialized_avs_fails() {
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

        // Use uninitialized AVS
        let uninitialized_avs = Keypair::new();
        let uninitialized_avs_pubkey =
            Avs::find_program_address(&jito_restaking_program::id(), &uninitialized_avs.pubkey()).0;

        let operator_avs_ticket = OperatorAvsTicket::find_program_address(
            &jito_restaking_program::id(),
            &operator_pubkey,
            &uninitialized_avs_pubkey,
        )
        .0;

        let result = restaking_program_client
            .operator_add_avs(
                &config,
                &operator_pubkey,
                &uninitialized_avs_pubkey,
                &operator_avs_ticket,
                &operator_admin,
                &payer,
            )
            .await;

        // TODO (LB): check for specific error
        assert!(result.is_err());
    }
}
