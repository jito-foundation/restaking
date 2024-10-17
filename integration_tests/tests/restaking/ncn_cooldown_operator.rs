#[cfg(test)]
mod tests {
    use jito_jsm_core::slot_toggle::SlotToggleState;
    use jito_restaking_core::{config::Config, ncn_operator_state::NcnOperatorState};
    use jito_restaking_sdk::error::RestakingError;
    use solana_sdk::signature::Keypair;

    use crate::fixtures::{fixture::TestBuilder, restaking_client::assert_restaking_error};

    #[tokio::test]
    async fn test_ncn_cooldown_operator_ok() {
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
            .do_ncn_warmup_operator(&ncn_root, &operator_root.operator_pubkey)
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
            .do_ncn_cooldown_operator(&ncn_root, &operator_root.operator_pubkey)
            .await
            .unwrap();

        let ncn_operator_state = restaking_program_client
            .get_ncn_operator_state(&ncn_root.ncn_pubkey, &operator_root.operator_pubkey)
            .await
            .unwrap();

        assert_eq!(
            ncn_operator_state
                .ncn_opt_in_state
                .state(
                    fixture.get_current_slot().await.unwrap(),
                    config.epoch_length()
                )
                .unwrap(),
            SlotToggleState::Cooldown
        );
    }

    #[tokio::test]
    async fn test_ncn_cooldown_operator_wrong_admin_fails() {
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
            .do_ncn_warmup_operator(&ncn_root, &operator_root.operator_pubkey)
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

        let result = restaking_program_client
            .ncn_cooldown_operator(
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
        assert_restaking_error(result, RestakingError::NcnOperatorAdminInvalid);
    }

    #[tokio::test]
    async fn test_ncn_cooldown_operator_inactive_fails() {
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
            .do_ncn_cooldown_operator(&ncn_root, &operator_root.operator_pubkey)
            .await;
        assert_restaking_error(result, RestakingError::NcnCooldownOperatorFailed);
    }

    #[tokio::test]
    async fn test_ncn_cooldown_operator_warming_up_fails() {
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
            .do_ncn_warmup_operator(&ncn_root, &operator_root.operator_pubkey)
            .await
            .unwrap();

        let result = restaking_program_client
            .do_ncn_cooldown_operator(&ncn_root, &operator_root.operator_pubkey)
            .await;
        assert_restaking_error(result, RestakingError::NcnCooldownOperatorFailed);
    }

    #[tokio::test]
    async fn test_ncn_cooldown_operator_cooling_down_fails() {
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
            .do_ncn_warmup_operator(&ncn_root, &operator_root.operator_pubkey)
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
            .do_ncn_cooldown_operator(&ncn_root, &operator_root.operator_pubkey)
            .await
            .unwrap();

        // new blockhash
        fixture.warp_slot_incremental(1).await.unwrap();

        let result = restaking_program_client
            .do_ncn_cooldown_operator(&ncn_root, &operator_root.operator_pubkey)
            .await;
        assert_restaking_error(result, RestakingError::NcnCooldownOperatorFailed);
    }
}
