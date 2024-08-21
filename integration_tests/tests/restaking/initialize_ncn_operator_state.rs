#[cfg(test)]
mod tests {
    use jito_jsm_core::slot_toggle::SlotToggleState;
    use jito_restaking_core::{config::Config, ncn_operator_state::NcnOperatorState};
    use jito_restaking_sdk::error::RestakingError;
    use solana_program::instruction::InstructionError;
    use solana_sdk::{signature::Keypair, transaction::TransactionError};

    use crate::fixtures::{fixture::TestBuilder, restaking_client::assert_restaking_error};

    #[tokio::test]
    async fn test_initialize_ncn_operator_state_ok() {
        let mut fixture = TestBuilder::new().await;
        let mut restaking_program_client = fixture.restaking_program_client();

        let _config_admin = restaking_program_client
            .do_initialize_config()
            .await
            .unwrap();

        let config = restaking_program_client
            .get_config(&Config::find_program_address(&jito_restaking_program::id()).0)
            .await
            .unwrap();

        let ncn_root = restaking_program_client.do_initialize_ncn().await.unwrap();
        let operator_root = restaking_program_client
            .do_initialize_operator()
            .await
            .unwrap();

        restaking_program_client
            .do_initialize_ncn_operator_state(&ncn_root, &operator_root.operator_pubkey)
            .await
            .unwrap();

        let ncn = restaking_program_client
            .get_ncn(&ncn_root.ncn_pubkey)
            .await
            .unwrap();
        assert_eq!(ncn.operator_count(), 1);

        let operator = restaking_program_client
            .get_operator(&operator_root.operator_pubkey)
            .await
            .unwrap();
        assert_eq!(operator.ncn_count(), 1);

        let ncn_operator_state = restaking_program_client
            .get_ncn_operator_state(&ncn_root.ncn_pubkey, &operator_root.operator_pubkey)
            .await
            .unwrap();

        assert_eq!(ncn_operator_state.ncn, ncn_root.ncn_pubkey);
        assert_eq!(ncn_operator_state.operator, operator_root.operator_pubkey);
        assert_eq!(ncn_operator_state.index(), 0);

        let slot = fixture.get_current_slot().await.unwrap();
        assert_eq!(
            ncn_operator_state
                .ncn_opt_in_state
                .state(slot, config.epoch_length()),
            SlotToggleState::Inactive
        );
        assert_eq!(
            ncn_operator_state
                .operator_opt_in_state
                .state(slot, config.epoch_length()),
            SlotToggleState::Inactive
        );
    }

    #[tokio::test]
    async fn test_initialize_ncn_operator_state_wrong_admin_fails() {
        let fixture = TestBuilder::new().await;
        let mut restaking_program_client = fixture.restaking_program_client();

        let _config_admin = restaking_program_client
            .do_initialize_config()
            .await
            .unwrap();

        let ncn_root = restaking_program_client.do_initialize_ncn().await.unwrap();
        let operator_root = restaking_program_client
            .do_initialize_operator()
            .await
            .unwrap();

        let transaction_error = restaking_program_client
            .initialize_ncn_operator_state(
                &Config::find_program_address(&jito_restaking_program::id()).0,
                &ncn_root.ncn_pubkey,
                &operator_root.operator_pubkey,
                &NcnOperatorState::find_program_address(
                    &jito_restaking_program::id(),
                    &ncn_root.ncn_pubkey,
                    &operator_root.operator_pubkey,
                )
                .0,
                &Keypair::new(),
                &ncn_root.ncn_admin,
            )
            .await;

        assert_restaking_error(transaction_error, RestakingError::NcnOperatorAdminInvalid);
    }

    #[tokio::test]
    async fn test_initialize_ncn_operator_state_bad_pda_fails() {
        let fixture = TestBuilder::new().await;
        let mut restaking_program_client = fixture.restaking_program_client();

        let _config_admin = restaking_program_client
            .do_initialize_config()
            .await
            .unwrap();

        let ncn_root = restaking_program_client.do_initialize_ncn().await.unwrap();
        let operator_root = restaking_program_client
            .do_initialize_operator()
            .await
            .unwrap();

        let transaction_error = restaking_program_client
            .initialize_ncn_operator_state(
                &Config::find_program_address(&jito_restaking_program::id()).0,
                &ncn_root.ncn_pubkey,
                &operator_root.operator_pubkey,
                &NcnOperatorState::find_program_address(
                    &jito_restaking_program::id(),
                    // switched up
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
    async fn test_initialize_ncn_operator_state_multiple_operators_for_ncn_ok() {
        let fixture = TestBuilder::new().await;
        let mut restaking_program_client = fixture.restaking_program_client();

        let _config_admin = restaking_program_client
            .do_initialize_config()
            .await
            .unwrap();

        let ncn_root = restaking_program_client.do_initialize_ncn().await.unwrap();
        let operator_root_1 = restaking_program_client
            .do_initialize_operator()
            .await
            .unwrap();
        let operator_root_2 = restaking_program_client
            .do_initialize_operator()
            .await
            .unwrap();

        restaking_program_client
            .do_initialize_ncn_operator_state(&ncn_root, &operator_root_1.operator_pubkey)
            .await
            .unwrap();

        restaking_program_client
            .do_initialize_ncn_operator_state(&ncn_root, &operator_root_2.operator_pubkey)
            .await
            .unwrap();

        let ncn = restaking_program_client
            .get_ncn(&ncn_root.ncn_pubkey)
            .await
            .unwrap();
        assert_eq!(ncn.operator_count(), 2);

        let operator_1 = restaking_program_client
            .get_operator(&operator_root_1.operator_pubkey)
            .await
            .unwrap();
        assert_eq!(operator_1.ncn_count(), 1);

        let operator_2 = restaking_program_client
            .get_operator(&operator_root_2.operator_pubkey)
            .await
            .unwrap();
        assert_eq!(operator_2.ncn_count(), 1);
    }

    #[tokio::test]
    async fn test_initialize_ncn_operator_state_multiple_ncns_for_operator_ok() {
        let fixture = TestBuilder::new().await;
        let mut restaking_program_client = fixture.restaking_program_client();

        let _config_admin = restaking_program_client
            .do_initialize_config()
            .await
            .unwrap();

        let ncn_root_1 = restaking_program_client.do_initialize_ncn().await.unwrap();
        let ncn_root_2 = restaking_program_client.do_initialize_ncn().await.unwrap();
        let operator_root = restaking_program_client
            .do_initialize_operator()
            .await
            .unwrap();

        restaking_program_client
            .do_initialize_ncn_operator_state(&ncn_root_1, &operator_root.operator_pubkey)
            .await
            .unwrap();
        restaking_program_client
            .do_initialize_ncn_operator_state(&ncn_root_2, &operator_root.operator_pubkey)
            .await
            .unwrap();

        let ncn_1 = restaking_program_client
            .get_ncn(&ncn_root_1.ncn_pubkey)
            .await
            .unwrap();
        assert_eq!(ncn_1.operator_count(), 1);

        let ncn_2 = restaking_program_client
            .get_ncn(&ncn_root_2.ncn_pubkey)
            .await
            .unwrap();
        assert_eq!(ncn_2.operator_count(), 1);

        let operator = restaking_program_client
            .get_operator(&operator_root.operator_pubkey)
            .await
            .unwrap();
        assert_eq!(operator.ncn_count(), 2);
    }
}
