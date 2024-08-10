#[cfg(test)]
mod tests {
    use jito_restaking_core::{
        config::Config, ncn::Ncn, ncn_operator_ticket::NcnOperatorTicket, operator::Operator,
        operator_ncn_ticket::OperatorNcnTicket,
    };
    use solana_sdk::signature::{Keypair, Signer};

    use crate::fixtures::fixture::TestBuilder;

    #[tokio::test]
    async fn test_ncn_add_operator_ok() {
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

        // Operator adds NCN
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

        let config_account = restaking_program_client.get_config(&config).await.unwrap();
        fixture
            .warp_slot_incremental(2 * config_account.epoch_length())
            .await
            .unwrap();

        // NCN adds operator
        let ncn_operator_ticket = NcnOperatorTicket::find_program_address(
            &jito_restaking_program::id(),
            &ncn_pubkey,
            &operator_pubkey,
        )
        .0;
        restaking_program_client
            .ncn_add_operator(
                &config,
                &ncn_pubkey,
                &operator_pubkey,
                &ncn_operator_ticket,
                &operator_ncn_ticket,
                &ncn_admin,
                &payer,
            )
            .await
            .unwrap();

        // Verify NCN state
        let ncn = restaking_program_client.get_ncn(&ncn_pubkey).await.unwrap();
        assert_eq!(ncn.operator_count(), 1);

        // Verify NCN operator ticket
        let ticket = restaking_program_client
            .get_ncn_operator_ticket(&ncn_pubkey, &operator_pubkey)
            .await
            .unwrap();
        assert_eq!(ticket.ncn(), ncn_pubkey);
        assert_eq!(ticket.operator, operator_pubkey);
        assert_eq!(ticket.index(), 0);
        assert_eq!(
            ticket.state().slot_added(),
            fixture.get_current_slot().await.unwrap()
        );
    }

    #[tokio::test]
    async fn test_ncn_add_operator_without_operator_opt_in_fails() {
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

        // Attempt to add operator without operator opting in first
        let ncn_operator_ticket = NcnOperatorTicket::find_program_address(
            &jito_restaking_program::id(),
            &ncn_pubkey,
            &operator_pubkey,
        )
        .0;
        let operator_ncn_ticket = OperatorNcnTicket::find_program_address(
            &jito_restaking_program::id(),
            &operator_pubkey,
            &ncn_pubkey,
        )
        .0;
        let result = restaking_program_client
            .ncn_add_operator(
                &config,
                &ncn_pubkey,
                &operator_pubkey,
                &ncn_operator_ticket,
                &operator_ncn_ticket,
                &ncn_admin,
                &payer,
            )
            .await;

        // TODO (LB): check specific error
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_ncn_add_operator_non_admin_fails() {
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

        // Operator adds NCN
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

        // Attempt to add operator with non-admin signer
        let non_admin = Keypair::new();
        fixture.transfer(&non_admin.pubkey(), 10.0).await.unwrap();

        let ncn_operator_ticket = NcnOperatorTicket::find_program_address(
            &jito_restaking_program::id(),
            &ncn_pubkey,
            &operator_pubkey,
        )
        .0;
        let result = restaking_program_client
            .ncn_add_operator(
                &config,
                &ncn_pubkey,
                &operator_pubkey,
                &ncn_operator_ticket,
                &operator_ncn_ticket,
                &non_admin,
                &payer,
            )
            .await;

        // TODO (LB): check specific error
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_ncn_add_operator_duplicate_fails() {
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

        // Operator adds NCN
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

        let config_account = restaking_program_client.get_config(&config).await.unwrap();
        fixture
            .warp_slot_incremental(2 * config_account.epoch_length())
            .await
            .unwrap();

        // NCN adds operator (first time)
        let ncn_operator_ticket = NcnOperatorTicket::find_program_address(
            &jito_restaking_program::id(),
            &ncn_pubkey,
            &operator_pubkey,
        )
        .0;
        restaking_program_client
            .ncn_add_operator(
                &config,
                &ncn_pubkey,
                &operator_pubkey,
                &ncn_operator_ticket,
                &operator_ncn_ticket,
                &ncn_admin,
                &payer,
            )
            .await
            .unwrap();

        // Attempt to add the same operator again
        let result = restaking_program_client
            .ncn_add_operator(
                &config,
                &ncn_pubkey,
                &operator_pubkey,
                &ncn_operator_ticket,
                &operator_ncn_ticket,
                &ncn_admin,
                &payer,
            )
            .await;

        // TODO (LB): check specific error
        assert!(result.is_err());
    }
}
