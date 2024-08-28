#[cfg(test)]
mod tests {
    use jito_jsm_core::slot_toggle::SlotToggleState;
    use jito_restaking_core::{config::Config, ncn_operator_state::NcnOperatorState};
    use jito_restaking_sdk::error::RestakingError;
    use solana_sdk::signature::Keypair;

    use crate::fixtures::{fixture::TestBuilder, restaking_client::assert_restaking_error};

    #[tokio::test]
    async fn test_operator_warmup_ncn_ok() {
        let mut fixture = TestBuilder::new().await;
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

        restaking_program_client
            .do_initialize_ncn_operator_state(&ncn_root, &operator_root.operator_pubkey)
            .await
            .unwrap();
        fixture.warp_slot_incremental(1).await.unwrap();
        restaking_program_client
            .do_operator_warmup_ncn(&operator_root, &ncn_root.ncn_pubkey)
            .await
            .unwrap();

        let ncn_operator_state = restaking_program_client
            .get_ncn_operator_state(&ncn_root.ncn_pubkey, &operator_root.operator_pubkey)
            .await
            .unwrap();
        let slot = fixture.get_current_slot().await.unwrap();
        let config = restaking_program_client
            .get_config(&Config::find_program_address(&jito_restaking_program::id()).0)
            .await
            .unwrap();
        assert_eq!(
            ncn_operator_state
                .operator_opt_in_state
                .state(slot, config.epoch_length()),
            SlotToggleState::WarmUp
        );
    }

    #[tokio::test]
    async fn test_operator_warmup_ncn_wrong_admin_fails() {
        let mut fixture = TestBuilder::new().await;
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

        restaking_program_client
            .do_initialize_ncn_operator_state(&ncn_root, &operator_root.operator_pubkey)
            .await
            .unwrap();
        fixture.warp_slot_incremental(1).await.unwrap();
        let result = restaking_program_client
            .operator_warmup_ncn(
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
            )
            .await;
        assert_restaking_error(result, RestakingError::OperatorNcnAdminInvalid);
    }

    #[tokio::test]
    async fn test_operator_warmup_operator_warming_up_fails() {
        let mut fixture = TestBuilder::new().await;
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

        restaking_program_client
            .do_initialize_ncn_operator_state(&ncn_root, &operator_root.operator_pubkey)
            .await
            .unwrap();
        fixture.warp_slot_incremental(1).await.unwrap();
        restaking_program_client
            .do_operator_warmup_ncn(&operator_root, &ncn_root.ncn_pubkey)
            .await
            .unwrap();

        // get new blockhash
        fixture.warp_slot_incremental(1).await.unwrap();

        // already warming up above, can't warm up again
        let result = restaking_program_client
            .do_operator_warmup_ncn(&operator_root, &ncn_root.ncn_pubkey)
            .await;
        assert_restaking_error(result, RestakingError::OperatorWarmupNcnFailed);
    }

    #[tokio::test]
    async fn test_operator_warmup_ncn_operator_active_fails() {
        let mut fixture = TestBuilder::new().await;
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

        restaking_program_client
            .do_initialize_ncn_operator_state(&ncn_root, &operator_root.operator_pubkey)
            .await
            .unwrap();
        fixture.warp_slot_incremental(1).await.unwrap();
        restaking_program_client
            .do_operator_warmup_ncn(&operator_root, &ncn_root.ncn_pubkey)
            .await
            .unwrap();

        let config = restaking_program_client
            .get_config(&Config::find_program_address(&jito_restaking_program::id()).0)
            .await
            .unwrap();
        fixture
            .warp_slot_incremental(2 * config.epoch_length())
            .await
            .unwrap();

        // already active up above, can't warm up again
        let result = restaking_program_client
            .do_operator_warmup_ncn(&operator_root, &ncn_root.ncn_pubkey)
            .await;
        assert_restaking_error(result, RestakingError::OperatorWarmupNcnFailed);
    }

    #[tokio::test]
    async fn test_operator_warmup_ncn_operator_cooling_down_fails() {
        let mut fixture = TestBuilder::new().await;
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

        restaking_program_client
            .do_initialize_ncn_operator_state(&ncn_root, &operator_root.operator_pubkey)
            .await
            .unwrap();
        fixture.warp_slot_incremental(1).await.unwrap();
        restaking_program_client
            .do_operator_warmup_ncn(&operator_root, &ncn_root.ncn_pubkey)
            .await
            .unwrap();

        let config = restaking_program_client
            .get_config(&Config::find_program_address(&jito_restaking_program::id()).0)
            .await
            .unwrap();
        fixture
            .warp_slot_incremental(2 * config.epoch_length())
            .await
            .unwrap();

        restaking_program_client
            .do_operator_cooldown_ncn(&operator_root, &ncn_root.ncn_pubkey)
            .await
            .unwrap();

        // already cooling down above, can't warm up again
        let result = restaking_program_client
            .do_operator_warmup_ncn(&operator_root, &ncn_root.ncn_pubkey)
            .await;
        assert_restaking_error(result, RestakingError::OperatorWarmupNcnFailed);
    }
}
