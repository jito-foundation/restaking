#[cfg(test)]
mod tests {
    use jito_vault_core::{config::Config, vault_update_state_tracker::VaultUpdateStateTracker};
    use jito_vault_sdk::error::VaultError;
    use rstest::rstest;
    use solana_program::instruction::InstructionError;
    use solana_sdk::pubkey::Pubkey;

    use crate::fixtures::{
        assert_ix_error, fixture::TestBuilder, vault_client::assert_vault_error,
    };

    #[rstest]
    #[case(spl_token::id())]
    #[case(spl_token_2022::id())]
    #[tokio::test]
    async fn test_initialize_vault_update_state_tracker_ok(#[case] token_program: Pubkey) {
        let mut fixture = TestBuilder::new().await;

        let mut vault_program_client = fixture.vault_program_client();
        let (_vault_config_admin, vault_root) = vault_program_client
            .setup_config_and_vault(&token_program, 0, 0, 0)
            .await
            .unwrap();
        let vault_config = vault_program_client
            .get_config(&Config::find_program_address(&jito_vault_program::id()).0)
            .await
            .unwrap();

        let mut restaking_program_client = fixture.restaking_program_client();
        restaking_program_client
            .do_initialize_config()
            .await
            .unwrap();

        // create ncn operator state + warmup the ncn <> operator relationship
        let operator_root = restaking_program_client
            .do_initialize_operator()
            .await
            .unwrap();
        restaking_program_client
            .do_initialize_operator_vault_ticket(&operator_root, &vault_root.vault_pubkey)
            .await
            .unwrap();
        vault_program_client
            .do_initialize_vault_operator_delegation(&vault_root, &operator_root.operator_pubkey)
            .await
            .unwrap();

        fixture
            .warp_slot_incremental(2 * vault_config.epoch_length())
            .await
            .unwrap();

        let slot = fixture.get_current_slot().await.unwrap();
        vault_program_client
            .initialize_vault_update_state_tracker(
                &vault_root.vault_pubkey,
                &VaultUpdateStateTracker::find_program_address(
                    &jito_vault_program::id(),
                    &vault_root.vault_pubkey,
                    slot / vault_config.epoch_length(),
                )
                .0,
            )
            .await
            .unwrap();

        let vault_update_state_tracker = vault_program_client
            .get_vault_update_state_tracker(
                &vault_root.vault_pubkey,
                slot / vault_config.epoch_length(),
            )
            .await
            .unwrap();

        assert_eq!(vault_update_state_tracker.vault, vault_root.vault_pubkey);
        assert_eq!(
            vault_update_state_tracker.ncn_epoch(),
            slot / vault_config.epoch_length()
        );
        assert_eq!(vault_update_state_tracker.last_updated_index(), u64::MAX);
        assert_eq!(
            vault_update_state_tracker
                .delegation_state
                .total_security()
                .unwrap(),
            0
        );
    }

    #[rstest]
    #[case(spl_token::id())]
    #[case(spl_token_2022::id())]
    #[tokio::test]
    async fn test_initialize_vault_update_state_tracker_no_operators_fails(
        #[case] token_program: Pubkey,
    ) {
        let mut fixture = TestBuilder::new().await;

        let mut vault_program_client = fixture.vault_program_client();
        let (_vault_config_admin, vault_root) = vault_program_client
            .setup_config_and_vault(&token_program, 0, 0, 0)
            .await
            .unwrap();
        let vault_config = vault_program_client
            .get_config(&Config::find_program_address(&jito_vault_program::id()).0)
            .await
            .unwrap();

        let slot = fixture.get_current_slot().await.unwrap();
        let result = vault_program_client
            .initialize_vault_update_state_tracker(
                &vault_root.vault_pubkey,
                &VaultUpdateStateTracker::find_program_address(
                    &jito_vault_program::id(),
                    &vault_root.vault_pubkey,
                    slot / vault_config.epoch_length(),
                )
                .0,
            )
            .await;
        assert_vault_error(result, VaultError::VaultIsUpdated);
    }

    #[rstest]
    #[case(spl_token::id())]
    #[case(spl_token_2022::id())]
    #[tokio::test]
    async fn test_initialize_vault_update_state_tracker_already_initialized_fails(
        #[case] token_program: Pubkey,
    ) {
        let mut fixture = TestBuilder::new().await;

        let mut vault_program_client = fixture.vault_program_client();
        let (_vault_config_admin, vault_root) = vault_program_client
            .setup_config_and_vault(&token_program, 0, 0, 0)
            .await
            .unwrap();
        let vault_config = vault_program_client
            .get_config(&Config::find_program_address(&jito_vault_program::id()).0)
            .await
            .unwrap();

        let mut restaking_program_client = fixture.restaking_program_client();
        restaking_program_client
            .do_initialize_config()
            .await
            .unwrap();

        // create ncn operator state + warmup the ncn <> operator relationship
        let operator_root = restaking_program_client
            .do_initialize_operator()
            .await
            .unwrap();
        restaking_program_client
            .do_initialize_operator_vault_ticket(&operator_root, &vault_root.vault_pubkey)
            .await
            .unwrap();
        vault_program_client
            .do_initialize_vault_operator_delegation(&vault_root, &operator_root.operator_pubkey)
            .await
            .unwrap();

        fixture
            .warp_slot_incremental(2 * vault_config.epoch_length())
            .await
            .unwrap();

        let slot = fixture.get_current_slot().await.unwrap();
        vault_program_client
            .initialize_vault_update_state_tracker(
                &vault_root.vault_pubkey,
                &VaultUpdateStateTracker::find_program_address(
                    &jito_vault_program::id(),
                    &vault_root.vault_pubkey,
                    slot / vault_config.epoch_length(),
                )
                .0,
            )
            .await
            .unwrap();

        fixture.warp_slot_incremental(1).await.unwrap();

        let result = vault_program_client
            .initialize_vault_update_state_tracker(
                &vault_root.vault_pubkey,
                &VaultUpdateStateTracker::find_program_address(
                    &jito_vault_program::id(),
                    &vault_root.vault_pubkey,
                    slot / vault_config.epoch_length(),
                )
                .0,
            )
            .await;

        assert_ix_error(result, InstructionError::InvalidAccountOwner);
    }
}
