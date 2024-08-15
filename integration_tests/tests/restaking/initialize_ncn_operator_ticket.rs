#[cfg(test)]
mod tests {
    use jito_jsm_core::slot_toggle::SlotToggleState;
    use jito_restaking_core::{
        config::Config, ncn_operator_ticket::NcnOperatorTicket,
        operator_ncn_ticket::OperatorNcnTicket,
    };
    use jito_restaking_sdk::error::RestakingError;
    use solana_program::instruction::InstructionError;
    use solana_sdk::{signature::Keypair, transaction::TransactionError};

    use crate::fixtures::{
        fixture::TestBuilder,
        restaking_client::{NcnRoot, OperatorRoot, RestakingProgramClient},
    };

    async fn setup_config_ncn_operator(
        restaking_program_client: &mut RestakingProgramClient,
    ) -> (NcnRoot, OperatorRoot) {
        restaking_program_client
            .do_initialize_config()
            .await
            .unwrap();
        let ncn_root = restaking_program_client.do_initialize_ncn().await.unwrap();
        let operator_root = restaking_program_client
            .do_initialize_operator()
            .await
            .unwrap();
        (ncn_root, operator_root)
    }

    #[tokio::test]
    async fn test_initialize_ncn_operator_ticket_ok() {
        let mut fixture = TestBuilder::new().await;
        let mut restaking_program_client = fixture.restaking_program_client();

        let (ncn_root, operator_root) =
            setup_config_ncn_operator(&mut restaking_program_client).await;

        restaking_program_client
            .do_initialize_operator_ncn_ticket(&operator_root, &ncn_root.ncn_pubkey)
            .await
            .unwrap();

        let config = restaking_program_client
            .get_config(&Config::find_program_address(&jito_restaking_program::id()).0)
            .await
            .unwrap();
        // transition to active epoch (one full epoch)
        fixture
            .warp_slot_incremental(2 * config.epoch_length)
            .await
            .unwrap();

        restaking_program_client
            .do_initialize_ncn_operator_ticket(&ncn_root, &operator_root.operator_pubkey)
            .await
            .unwrap();

        let ncn = restaking_program_client
            .get_ncn(&ncn_root.ncn_pubkey)
            .await
            .unwrap();
        assert_eq!(ncn.operator_count, 1);

        // Verify NCN operator ticket
        let ticket = restaking_program_client
            .get_ncn_operator_ticket(&ncn_root.ncn_pubkey, &operator_root.operator_pubkey)
            .await
            .unwrap();
        assert_eq!(ticket.ncn, ncn_root.ncn_pubkey);
        assert_eq!(ticket.operator, operator_root.operator_pubkey);
        assert_eq!(ticket.index, 0);
        assert_eq!(
            ticket.state.state(
                fixture.get_current_slot().await.unwrap(),
                config.epoch_length
            ),
            SlotToggleState::WarmUp
        );
    }

    #[tokio::test]
    async fn test_initialize_ncn_operator_ticket_without_operator_opt_in_fails() {
        let fixture = TestBuilder::new().await;
        let mut restaking_program_client = fixture.restaking_program_client();

        let (ncn_root, operator_root) =
            setup_config_ncn_operator(&mut restaking_program_client).await;

        let transaction_error = restaking_program_client
            .do_initialize_ncn_operator_ticket(&ncn_root, &operator_root.operator_pubkey)
            .await
            .unwrap_err()
            .to_transaction_error()
            .unwrap();

        // OperatorNcnTicket doesn't exist yet, so owned by system program
        assert_eq!(
            transaction_error,
            TransactionError::InstructionError(0, InstructionError::InvalidAccountOwner)
        );
    }

    #[tokio::test]
    async fn test_initialize_ncn_operator_ticket_bad_admin_fails() {
        let fixture = TestBuilder::new().await;
        let mut restaking_program_client = fixture.restaking_program_client();

        let (ncn_root, operator_root) =
            setup_config_ncn_operator(&mut restaking_program_client).await;

        restaking_program_client
            .do_initialize_operator_ncn_ticket(&operator_root, &ncn_root.ncn_pubkey)
            .await
            .unwrap();

        let tx_error = restaking_program_client
            .initialize_ncn_operator_ticket(
                &Config::find_program_address(&jito_restaking_program::id()).0,
                &ncn_root.ncn_pubkey,
                &operator_root.operator_pubkey,
                &NcnOperatorTicket::find_program_address(
                    &jito_restaking_program::id(),
                    &ncn_root.ncn_pubkey,
                    &operator_root.operator_pubkey,
                )
                .0,
                &OperatorNcnTicket::find_program_address(
                    &jito_restaking_program::id(),
                    &operator_root.operator_pubkey,
                    &ncn_root.ncn_pubkey,
                )
                .0,
                &Keypair::new(),
                &ncn_root.ncn_admin,
            )
            .await
            .unwrap_err()
            .to_transaction_error()
            .unwrap();

        assert_eq!(
            tx_error,
            TransactionError::InstructionError(
                0,
                InstructionError::Custom(RestakingError::NcnOperatorAdminInvalid as u32)
            )
        );
    }

    #[tokio::test]
    async fn test_initialize_ncn_operator_ticket_bad_pda_fails() {
        let fixture = TestBuilder::new().await;
        let mut restaking_program_client = fixture.restaking_program_client();

        let (ncn_root, operator_root) =
            setup_config_ncn_operator(&mut restaking_program_client).await;

        let ncn_root_2 = restaking_program_client.do_initialize_ncn().await.unwrap();
        let operator_root_2 = restaking_program_client
            .do_initialize_operator()
            .await
            .unwrap();

        restaking_program_client
            .do_initialize_operator_ncn_ticket(&operator_root_2, &ncn_root_2.ncn_pubkey)
            .await
            .unwrap();

        restaking_program_client
            .do_initialize_operator_ncn_ticket(&operator_root, &ncn_root.ncn_pubkey)
            .await
            .unwrap();

        let transaction_error = restaking_program_client
            .initialize_ncn_operator_ticket(
                &Config::find_program_address(&jito_restaking_program::id()).0,
                &ncn_root.ncn_pubkey,
                &operator_root.operator_pubkey,
                &NcnOperatorTicket::find_program_address(
                    &jito_restaking_program::id(),
                    &operator_root_2.operator_pubkey,
                    &ncn_root_2.ncn_pubkey,
                )
                .0,
                &OperatorNcnTicket::find_program_address(
                    &jito_restaking_program::id(),
                    &operator_root.operator_pubkey,
                    &ncn_root.ncn_pubkey,
                )
                .0,
                &ncn_root.ncn_admin,
                &ncn_root.ncn_admin,
            )
            .await
            .unwrap_err()
            .to_transaction_error()
            .unwrap();
        assert_eq!(
            transaction_error,
            TransactionError::InstructionError(0, InstructionError::InvalidAccountData)
        );
    }

    #[tokio::test]
    async fn test_initialize_ncn_operator_ticket_operator_ncn_warming_up_fails() {
        let mut fixture = TestBuilder::new().await;
        let mut restaking_program_client = fixture.restaking_program_client();

        let (ncn_root, operator_root) =
            setup_config_ncn_operator(&mut restaking_program_client).await;

        restaking_program_client
            .do_initialize_operator_ncn_ticket(&operator_root, &ncn_root.ncn_pubkey)
            .await
            .unwrap();

        let config = restaking_program_client
            .get_config(&Config::find_program_address(&jito_restaking_program::id()).0)
            .await
            .unwrap();
        fixture
            .warp_slot_incremental(config.epoch_length)
            .await
            .unwrap();

        let transaction_error = restaking_program_client
            .do_initialize_ncn_operator_ticket(&ncn_root, &operator_root.operator_pubkey)
            .await
            .unwrap_err()
            .to_transaction_error()
            .unwrap();
        assert_eq!(
            transaction_error,
            TransactionError::InstructionError(
                0,
                InstructionError::Custom(RestakingError::OperatorNcnTicketNotActive as u32)
            )
        );
    }

    #[tokio::test]
    async fn test_initialize_ncn_operator_ticket_operator_ncn_cooling_down_fails() {
        let mut fixture = TestBuilder::new().await;
        let mut restaking_program_client = fixture.restaking_program_client();

        let (ncn_root, operator_root) =
            setup_config_ncn_operator(&mut restaking_program_client).await;

        restaking_program_client
            .do_initialize_operator_ncn_ticket(&operator_root, &ncn_root.ncn_pubkey)
            .await
            .unwrap();

        let config = restaking_program_client
            .get_config(&Config::find_program_address(&jito_restaking_program::id()).0)
            .await
            .unwrap();
        fixture
            .warp_slot_incremental(2 * config.epoch_length)
            .await
            .unwrap();

        restaking_program_client
            .do_cooldown_operator_ncn_ticket(&operator_root, &ncn_root.ncn_pubkey)
            .await
            .unwrap();

        let transaction_error = restaking_program_client
            .do_initialize_ncn_operator_ticket(&ncn_root, &operator_root.operator_pubkey)
            .await
            .unwrap_err()
            .to_transaction_error()
            .unwrap();
        assert_eq!(
            transaction_error,
            TransactionError::InstructionError(
                0,
                InstructionError::Custom(RestakingError::OperatorNcnTicketNotActive as u32)
            )
        );
    }

    #[tokio::test]
    async fn test_initialize_ncn_operator_ticket_operator_ncn_inactive_fails() {
        let mut fixture = TestBuilder::new().await;
        let mut restaking_program_client = fixture.restaking_program_client();

        let (ncn_root, operator_root) =
            setup_config_ncn_operator(&mut restaking_program_client).await;

        restaking_program_client
            .do_initialize_operator_ncn_ticket(&operator_root, &ncn_root.ncn_pubkey)
            .await
            .unwrap();

        let config = restaking_program_client
            .get_config(&Config::find_program_address(&jito_restaking_program::id()).0)
            .await
            .unwrap();
        fixture
            .warp_slot_incremental(2 * config.epoch_length)
            .await
            .unwrap();

        restaking_program_client
            .do_cooldown_operator_ncn_ticket(&operator_root, &ncn_root.ncn_pubkey)
            .await
            .unwrap();

        let transaction_error = restaking_program_client
            .do_initialize_ncn_operator_ticket(&ncn_root, &operator_root.operator_pubkey)
            .await
            .unwrap_err()
            .to_transaction_error()
            .unwrap();

        fixture
            .warp_slot_incremental(2 * config.epoch_length)
            .await
            .unwrap();

        assert_eq!(
            transaction_error,
            TransactionError::InstructionError(
                0,
                InstructionError::Custom(RestakingError::OperatorNcnTicketNotActive as u32)
            )
        );
    }

    #[tokio::test]
    async fn test_initialize_ncn_operator_ticket_twice_fails() {
        let mut fixture = TestBuilder::new().await;
        let mut restaking_program_client = fixture.restaking_program_client();

        let (ncn_root, operator_root) =
            setup_config_ncn_operator(&mut restaking_program_client).await;

        restaking_program_client
            .do_initialize_operator_ncn_ticket(&operator_root, &ncn_root.ncn_pubkey)
            .await
            .unwrap();

        // get new blockhash for tx
        fixture.warp_slot_incremental(1).await.unwrap();

        let transaction_error = restaking_program_client
            .do_initialize_operator_ncn_ticket(&operator_root, &ncn_root.ncn_pubkey)
            .await
            .unwrap_err()
            .to_transaction_error()
            .unwrap();
        // expected the account to be initialized owned by system program
        assert_eq!(
            transaction_error,
            TransactionError::InstructionError(0, InstructionError::InvalidAccountOwner)
        );
    }
}
