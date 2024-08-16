#[cfg(test)]
mod tests {
    use crate::fixtures::fixture::TestBuilder;
    use jito_vault_core::config::Config;

    struct TestState {}

    async fn setup_test(num_operators: u64) -> TestState {
        let mut fixture = TestBuilder::new().await;
        let mut restaking_program_client = fixture.restaking_program_client();
        let mut vault_program_client = fixture.vault_program_client();

        restaking_program_client
            .do_initialize_config()
            .await
            .unwrap();
        let (_, vault_root) = vault_program_client
            .setup_config_and_vault(0, 0)
            .await
            .unwrap();

        let ncn_root = restaking_program_client.do_initialize_ncn().await.unwrap();
        restaking_program_client
            .do_initialize_ncn_vault_ticket(&ncn_root, &vault_root.vault_pubkey)
            .await
            .unwrap();

        let mut operator_roots = Vec::with_capacity(num_operators as usize);
        for _ in 0..num_operators {
            let operator_root = restaking_program_client
                .do_initialize_operator()
                .await
                .unwrap();
            restaking_program_client
                .do_initialize_operator_vault_ticket(&operator_root, &vault_root.vault_pubkey)
                .await
                .unwrap();
            restaking_program_client
                .do_initialize_ncn_operator_state(&ncn_root, &operator_root.operator_pubkey)
                .await
                .unwrap();
            restaking_program_client
                .do_ncn_warmup_operator(&ncn_root, &operator_root.operator_pubkey)
                .await
                .unwrap();
            restaking_program_client
                .do_operator_warmup_ncn(&operator_root, &ncn_root.ncn_pubkey)
                .await
                .unwrap();
            operator_roots.push(operator_root);
        }

        let config = vault_program_client
            .get_config(&Config::find_program_address(&jito_vault_program::id()).0)
            .await
            .unwrap();
        fixture
            .warp_slot_incremental(2 * config.epoch_length)
            .await
            .unwrap();

        for operator_root in operator_roots {
            vault_program_client
                .do_initialize_vault_operator_ticket(&vault_root, &operator_root.operator_pubkey)
                .await
                .unwrap();
            vault_program_client
                .do_initialize_vault_ncn_ticket(&vault_root, &ncn_root.ncn_pubkey)
                .await
                .unwrap();
        }

        TestState {}
    }

    #[tokio::test]
    async fn test_crank_vault_update_state_tracker_ok() {
        let _state = setup_test(2).await;
    }

    #[tokio::test]
    async fn test_crank_vault_update_state_tracker_multiple_operators_ok() {
        let _state = setup_test(2).await;
    }

    #[tokio::test]
    async fn test_crank_vault_update_state_tracker_out_of_order_fails() {
        let _state = setup_test(2).await;
    }

    #[tokio::test]
    async fn test_crank_vault_update_state_tracker_partial_update_previous_epoch_ok() {
        let _state = setup_test(2).await;
    }
}
