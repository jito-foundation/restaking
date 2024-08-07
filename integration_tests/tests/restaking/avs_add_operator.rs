#[cfg(test)]
mod tests {
    use jito_restaking_core::{
        avs::Avs, avs_operator_ticket::AvsOperatorTicket, config::Config, operator::Operator,
        operator_avs_ticket::OperatorAvsTicket,
    };
    use solana_sdk::signature::{Keypair, Signer};

    use crate::fixtures::fixture::TestBuilder;

    #[tokio::test]
    async fn test_avs_add_operator_ok() {
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

        // Operator adds AVS
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

        // AVS adds operator
        let avs_operator_ticket = AvsOperatorTicket::find_program_address(
            &jito_restaking_program::id(),
            &avs_pubkey,
            &operator_pubkey,
        )
        .0;
        restaking_program_client
            .avs_add_operator(
                &config,
                &avs_pubkey,
                &operator_pubkey,
                &avs_operator_ticket,
                &operator_avs_ticket,
                &avs_admin,
                &payer,
            )
            .await
            .unwrap();

        // Verify AVS state
        let avs = restaking_program_client.get_avs(&avs_pubkey).await.unwrap();
        assert_eq!(avs.operator_count(), 1);

        // Verify AVS operator ticket
        let ticket = restaking_program_client
            .get_avs_operator_ticket(&avs_pubkey, &operator_pubkey)
            .await
            .unwrap();
        assert_eq!(ticket.avs(), avs_pubkey);
        assert_eq!(ticket.operator(), operator_pubkey);
        assert_eq!(ticket.index(), 0);
        assert_eq!(ticket.state().slot_added(), 1);
    }

    #[tokio::test]
    async fn test_avs_add_operator_without_operator_opt_in_fails() {
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

        // Attempt to add operator without operator opting in first
        let avs_operator_ticket = AvsOperatorTicket::find_program_address(
            &jito_restaking_program::id(),
            &avs_pubkey,
            &operator_pubkey,
        )
        .0;
        let operator_avs_ticket = OperatorAvsTicket::find_program_address(
            &jito_restaking_program::id(),
            &operator_pubkey,
            &avs_pubkey,
        )
        .0;
        let result = restaking_program_client
            .avs_add_operator(
                &config,
                &avs_pubkey,
                &operator_pubkey,
                &avs_operator_ticket,
                &operator_avs_ticket,
                &avs_admin,
                &payer,
            )
            .await;

        // TODO (LB): check specific error
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_avs_add_operator_non_admin_fails() {
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

        // Operator adds AVS
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

        // Attempt to add operator with non-admin signer
        let non_admin = Keypair::new();
        fixture.transfer(&non_admin.pubkey(), 10.0).await.unwrap();

        let avs_operator_ticket = AvsOperatorTicket::find_program_address(
            &jito_restaking_program::id(),
            &avs_pubkey,
            &operator_pubkey,
        )
        .0;
        let result = restaking_program_client
            .avs_add_operator(
                &config,
                &avs_pubkey,
                &operator_pubkey,
                &avs_operator_ticket,
                &operator_avs_ticket,
                &non_admin,
                &payer,
            )
            .await;

        // TODO (LB): check specific error
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_avs_add_operator_duplicate_fails() {
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

        // Operator adds AVS
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

        // AVS adds operator (first time)
        let avs_operator_ticket = AvsOperatorTicket::find_program_address(
            &jito_restaking_program::id(),
            &avs_pubkey,
            &operator_pubkey,
        )
        .0;
        restaking_program_client
            .avs_add_operator(
                &config,
                &avs_pubkey,
                &operator_pubkey,
                &avs_operator_ticket,
                &operator_avs_ticket,
                &avs_admin,
                &payer,
            )
            .await
            .unwrap();

        // Attempt to add the same operator again
        let result = restaking_program_client
            .avs_add_operator(
                &config,
                &avs_pubkey,
                &operator_pubkey,
                &avs_operator_ticket,
                &operator_avs_ticket,
                &avs_admin,
                &payer,
            )
            .await;

        // TODO (LB): check specific error
        assert!(result.is_err());
    }
}
