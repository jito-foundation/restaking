#[cfg(test)]
mod tests {
    use jito_vault_core::{
        config::Config, delegation_state::DelegationState,
        vault_update_state_tracker::VaultUpdateStateTracker,
    };
    use jito_vault_sdk::error::VaultError;
    use rstest::rstest;
    use solana_sdk::{
        pubkey::Pubkey,
        signature::{Keypair, Signer},
    };

    use crate::fixtures::{
        fixture::{ConfiguredVault, TestBuilder},
        vault_client::assert_vault_error,
    };

    #[rstest]
    #[case(spl_token::id())]
    #[case(spl_token_2022::id())]
    #[tokio::test]
    async fn test_crank_vault_update_state_tracker_ok(#[case] token_program: Pubkey) {
        let mut fixture = TestBuilder::new().await;

        let deposit_fee_bps = 0;
        let withdraw_fee_bps = 0;
        let reward_fee_bps = 0;
        let num_operators = 1;
        let slasher_amounts = vec![];

        let ConfiguredVault {
            mut vault_program_client,
            vault_root,
            operator_roots,
            ..
        } = fixture
            .setup_vault_with_ncn_and_operators(
                &token_program,
                deposit_fee_bps,
                withdraw_fee_bps,
                reward_fee_bps,
                num_operators,
                &slasher_amounts,
            )
            .await
            .unwrap();

        let depositor = Keypair::new();
        vault_program_client
            .configure_depositor(&vault_root, &depositor.pubkey(), &token_program, 100_000)
            .await
            .unwrap();
        vault_program_client
            .do_mint_to(&vault_root, &depositor, &token_program, 100_000, 100_000)
            .await
            .unwrap();

        let config = vault_program_client
            .get_config(&Config::find_program_address(&jito_vault_program::id()).0)
            .await
            .unwrap();

        // go to next epoch to force update
        fixture
            .warp_slot_incremental(config.epoch_length())
            .await
            .unwrap();

        let slot = fixture.get_current_slot().await.unwrap();
        let ncn_epoch = slot / config.epoch_length();
        vault_program_client
            .initialize_vault_update_state_tracker(
                &vault_root.vault_pubkey,
                &VaultUpdateStateTracker::find_program_address(
                    &jito_vault_program::id(),
                    &vault_root.vault_pubkey,
                    ncn_epoch,
                )
                .0,
            )
            .await
            .unwrap();

        let vault_update_state_tracker = vault_program_client
            .get_vault_update_state_tracker(&vault_root.vault_pubkey, ncn_epoch)
            .await
            .unwrap();
        assert_eq!(vault_update_state_tracker.vault, vault_root.vault_pubkey);
        assert_eq!(vault_update_state_tracker.ncn_epoch(), ncn_epoch);
        assert_eq!(vault_update_state_tracker.last_updated_index(), u64::MAX);
        assert_eq!(
            vault_update_state_tracker.delegation_state,
            DelegationState::default()
        );

        vault_program_client
            .do_crank_vault_update_state_tracker(
                &vault_root.vault_pubkey,
                &operator_roots[0].operator_pubkey,
            )
            .await
            .unwrap();
        let vault_update_state_tracker = vault_program_client
            .get_vault_update_state_tracker(&vault_root.vault_pubkey, ncn_epoch)
            .await
            .unwrap();
        assert_eq!(vault_update_state_tracker.last_updated_index(), 0);
        assert_eq!(
            vault_update_state_tracker.delegation_state,
            DelegationState::default()
        );

        let operator_delegation = vault_program_client
            .get_vault_operator_delegation(
                &vault_root.vault_pubkey,
                &operator_roots[0].operator_pubkey,
            )
            .await
            .unwrap();
        let slot = fixture.get_current_slot().await.unwrap();
        assert_eq!(operator_delegation.vault, vault_root.vault_pubkey);
        assert_eq!(
            operator_delegation.operator,
            operator_roots[0].operator_pubkey
        );
        assert_eq!(
            operator_delegation.delegation_state,
            DelegationState::default()
        );
        assert_eq!(operator_delegation.last_update_slot(), slot);
    }

    #[rstest]
    #[case(spl_token::id())]
    #[case(spl_token_2022::id())]
    #[tokio::test]
    async fn test_crank_vault_update_state_tracker_multiple_operators_ok(
        #[case] token_program: Pubkey,
    ) {
        let mut fixture = TestBuilder::new().await;

        let deposit_fee_bps = 0;
        let withdraw_fee_bps = 0;
        let reward_fee_bps = 0;
        let num_operators = 2;
        let slasher_amounts = vec![];

        let ConfiguredVault {
            mut vault_program_client,
            vault_root,
            operator_roots,
            ..
        } = fixture
            .setup_vault_with_ncn_and_operators(
                &token_program,
                deposit_fee_bps,
                withdraw_fee_bps,
                reward_fee_bps,
                num_operators,
                &slasher_amounts,
            )
            .await
            .unwrap();

        let depositor = Keypair::new();
        vault_program_client
            .configure_depositor(&vault_root, &depositor.pubkey(), &token_program, 100_000)
            .await
            .unwrap();
        vault_program_client
            .do_mint_to(&vault_root, &depositor, &token_program, 100_000, 100_000)
            .await
            .unwrap();

        vault_program_client
            .do_add_delegation(&vault_root, &operator_roots[0].operator_pubkey, 50_000)
            .await
            .unwrap();
        vault_program_client
            .do_add_delegation(&vault_root, &operator_roots[1].operator_pubkey, 50_000)
            .await
            .unwrap();

        let config = vault_program_client
            .get_config(&Config::find_program_address(&jito_vault_program::id()).0)
            .await
            .unwrap();
        fixture
            .warp_slot_incremental(config.epoch_length())
            .await
            .unwrap();

        let slot = fixture.get_current_slot().await.unwrap();
        let ncn_epoch = slot / config.epoch_length();

        let vault_update_state_tracker_pubkey = VaultUpdateStateTracker::find_program_address(
            &jito_vault_program::id(),
            &vault_root.vault_pubkey,
            ncn_epoch,
        )
        .0;
        vault_program_client
            .initialize_vault_update_state_tracker(
                &vault_root.vault_pubkey,
                &vault_update_state_tracker_pubkey,
            )
            .await
            .unwrap();

        vault_program_client
            .do_crank_vault_update_state_tracker(
                &vault_root.vault_pubkey,
                &operator_roots[0].operator_pubkey,
            )
            .await
            .unwrap();
        let vault_update_state_tracker = vault_program_client
            .get_vault_update_state_tracker(&vault_root.vault_pubkey, ncn_epoch)
            .await
            .unwrap();
        assert_eq!(vault_update_state_tracker.last_updated_index(), 0);
        assert_eq!(
            vault_update_state_tracker.delegation_state,
            DelegationState::new(50000, 0, 0)
        );

        vault_program_client
            .do_crank_vault_update_state_tracker(
                &vault_root.vault_pubkey,
                &operator_roots[1].operator_pubkey,
            )
            .await
            .unwrap();
        let vault_update_state_tracker = vault_program_client
            .get_vault_update_state_tracker(&vault_root.vault_pubkey, ncn_epoch)
            .await
            .unwrap();
        assert_eq!(vault_update_state_tracker.last_updated_index(), 1);
        assert_eq!(
            vault_update_state_tracker.delegation_state,
            DelegationState::new(100000, 0, 0)
        );
    }

    #[rstest]
    #[case(spl_token::id())]
    #[case(spl_token_2022::id())]
    #[tokio::test]
    async fn test_crank_vault_update_state_tracker_same_index_twice_fails(
        #[case] token_program: Pubkey,
    ) {
        let mut fixture = TestBuilder::new().await;

        let deposit_fee_bps = 0;
        let withdraw_fee_bps = 0;
        let reward_fee_bps = 0;
        let num_operators = 2;
        let slasher_amounts = vec![];

        let ConfiguredVault {
            mut vault_program_client,
            vault_root,
            operator_roots,
            ..
        } = fixture
            .setup_vault_with_ncn_and_operators(
                &token_program,
                deposit_fee_bps,
                withdraw_fee_bps,
                reward_fee_bps,
                num_operators,
                &slasher_amounts,
            )
            .await
            .unwrap();

        let depositor = Keypair::new();
        vault_program_client
            .configure_depositor(&vault_root, &depositor.pubkey(), &token_program, 100_000)
            .await
            .unwrap();
        vault_program_client
            .do_mint_to(&vault_root, &depositor, &token_program, 100_000, 100_000)
            .await
            .unwrap();

        vault_program_client
            .do_add_delegation(&vault_root, &operator_roots[0].operator_pubkey, 50_000)
            .await
            .unwrap();
        vault_program_client
            .do_add_delegation(&vault_root, &operator_roots[1].operator_pubkey, 50_000)
            .await
            .unwrap();

        let config = vault_program_client
            .get_config(&Config::find_program_address(&jito_vault_program::id()).0)
            .await
            .unwrap();
        fixture
            .warp_slot_incremental(config.epoch_length())
            .await
            .unwrap();

        let slot = fixture.get_current_slot().await.unwrap();
        let ncn_epoch = slot / config.epoch_length();

        let vault_update_state_tracker_pubkey = VaultUpdateStateTracker::find_program_address(
            &jito_vault_program::id(),
            &vault_root.vault_pubkey,
            ncn_epoch,
        )
        .0;
        vault_program_client
            .initialize_vault_update_state_tracker(
                &vault_root.vault_pubkey,
                &vault_update_state_tracker_pubkey,
            )
            .await
            .unwrap();

        vault_program_client
            .do_crank_vault_update_state_tracker(
                &vault_root.vault_pubkey,
                &operator_roots[0].operator_pubkey,
            )
            .await
            .unwrap();
        fixture.warp_slot_incremental(1).await.unwrap();
        let result = vault_program_client
            .do_crank_vault_update_state_tracker(
                &vault_root.vault_pubkey,
                &operator_roots[0].operator_pubkey,
            )
            .await;
        assert_vault_error(result, VaultError::VaultUpdateIncorrectIndex);
    }

    #[rstest]
    #[case(spl_token::id())]
    #[case(spl_token_2022::id())]
    #[tokio::test]
    async fn test_crank_vault_update_state_tracker_skip_zero_fails(#[case] token_program: Pubkey) {
        let mut fixture = TestBuilder::new().await;

        let deposit_fee_bps = 0;
        let withdraw_fee_bps = 0;
        let reward_fee_bps = 0;
        let num_operators = 2;
        let slasher_amounts = vec![];

        let ConfiguredVault {
            mut vault_program_client,
            vault_root,
            operator_roots,
            ..
        } = fixture
            .setup_vault_with_ncn_and_operators(
                &token_program,
                deposit_fee_bps,
                withdraw_fee_bps,
                reward_fee_bps,
                num_operators,
                &slasher_amounts,
            )
            .await
            .unwrap();

        let depositor = Keypair::new();
        vault_program_client
            .configure_depositor(&vault_root, &depositor.pubkey(), &token_program, 100_000)
            .await
            .unwrap();
        vault_program_client
            .do_mint_to(&vault_root, &depositor, &token_program, 100_000, 100_000)
            .await
            .unwrap();

        vault_program_client
            .do_add_delegation(&vault_root, &operator_roots[0].operator_pubkey, 50_000)
            .await
            .unwrap();
        vault_program_client
            .do_add_delegation(&vault_root, &operator_roots[1].operator_pubkey, 50_000)
            .await
            .unwrap();

        let config = vault_program_client
            .get_config(&Config::find_program_address(&jito_vault_program::id()).0)
            .await
            .unwrap();
        fixture
            .warp_slot_incremental(config.epoch_length())
            .await
            .unwrap();

        let slot = fixture.get_current_slot().await.unwrap();
        let ncn_epoch = slot / config.epoch_length();

        let vault_update_state_tracker_pubkey = VaultUpdateStateTracker::find_program_address(
            &jito_vault_program::id(),
            &vault_root.vault_pubkey,
            ncn_epoch,
        )
        .0;
        vault_program_client
            .initialize_vault_update_state_tracker(
                &vault_root.vault_pubkey,
                &vault_update_state_tracker_pubkey,
            )
            .await
            .unwrap();

        let result = vault_program_client
            .do_crank_vault_update_state_tracker(
                &vault_root.vault_pubkey,
                &operator_roots[1].operator_pubkey,
            )
            .await;
        assert_vault_error(result, VaultError::VaultUpdateIncorrectIndex);
    }

    #[rstest]
    #[case(spl_token::id())]
    #[case(spl_token_2022::id())]
    #[tokio::test]
    async fn test_crank_vault_update_state_tracker_skip_index_fails(#[case] token_program: Pubkey) {
        let mut fixture = TestBuilder::new().await;

        let deposit_fee_bps = 0;
        let withdraw_fee_bps = 0;
        let reward_fee_bps = 0;
        let num_operators = 3;
        let slasher_amounts = vec![];

        let ConfiguredVault {
            mut vault_program_client,
            vault_root,
            operator_roots,
            ..
        } = fixture
            .setup_vault_with_ncn_and_operators(
                &token_program,
                deposit_fee_bps,
                withdraw_fee_bps,
                reward_fee_bps,
                num_operators,
                &slasher_amounts,
            )
            .await
            .unwrap();

        let depositor = Keypair::new();
        vault_program_client
            .configure_depositor(&vault_root, &depositor.pubkey(), &token_program, 100_000)
            .await
            .unwrap();
        vault_program_client
            .do_mint_to(&vault_root, &depositor, &token_program, 100_000, 100_000)
            .await
            .unwrap();

        vault_program_client
            .do_add_delegation(&vault_root, &operator_roots[0].operator_pubkey, 50_000)
            .await
            .unwrap();
        vault_program_client
            .do_add_delegation(&vault_root, &operator_roots[1].operator_pubkey, 50_000)
            .await
            .unwrap();

        let config = vault_program_client
            .get_config(&Config::find_program_address(&jito_vault_program::id()).0)
            .await
            .unwrap();
        fixture
            .warp_slot_incremental(config.epoch_length())
            .await
            .unwrap();

        let slot = fixture.get_current_slot().await.unwrap();
        let ncn_epoch = slot / config.epoch_length();

        let vault_update_state_tracker_pubkey = VaultUpdateStateTracker::find_program_address(
            &jito_vault_program::id(),
            &vault_root.vault_pubkey,
            ncn_epoch,
        )
        .0;
        vault_program_client
            .initialize_vault_update_state_tracker(
                &vault_root.vault_pubkey,
                &vault_update_state_tracker_pubkey,
            )
            .await
            .unwrap();

        vault_program_client
            .do_crank_vault_update_state_tracker(
                &vault_root.vault_pubkey,
                &operator_roots[0].operator_pubkey,
            )
            .await
            .unwrap();

        let result = vault_program_client
            .do_crank_vault_update_state_tracker(
                &vault_root.vault_pubkey,
                &operator_roots[2].operator_pubkey,
            )
            .await;
        assert_vault_error(result, VaultError::VaultUpdateIncorrectIndex);
    }

    #[rstest]
    #[case(spl_token::id())]
    #[case(spl_token_2022::id())]
    #[tokio::test]
    async fn test_crank_vault_update_state_tracker_partial_update_previous_epoch_ok(
        #[case] token_program: Pubkey,
    ) {
        let mut fixture = TestBuilder::new().await;

        let deposit_fee_bps = 0;
        let withdraw_fee_bps = 0;
        let reward_fee_bps = 0;
        let num_operators = 2;
        let slasher_amounts = vec![];

        let ConfiguredVault {
            mut vault_program_client,
            vault_root,
            operator_roots,
            ..
        } = fixture
            .setup_vault_with_ncn_and_operators(
                &token_program,
                deposit_fee_bps,
                withdraw_fee_bps,
                reward_fee_bps,
                num_operators,
                &slasher_amounts,
            )
            .await
            .unwrap();

        let depositor = Keypair::new();
        vault_program_client
            .configure_depositor(&vault_root, &depositor.pubkey(), &token_program, 100_000)
            .await
            .unwrap();
        vault_program_client
            .do_mint_to(&vault_root, &depositor, &token_program, 100_000, 100_000)
            .await
            .unwrap();

        // 25k active, 25k cooling down on operator 0 and 1
        vault_program_client
            .do_add_delegation(&vault_root, &operator_roots[0].operator_pubkey, 50_000)
            .await
            .unwrap();
        vault_program_client
            .do_cooldown_delegation(&vault_root, &operator_roots[0].operator_pubkey, 25_000)
            .await
            .unwrap();
        vault_program_client
            .do_add_delegation(&vault_root, &operator_roots[1].operator_pubkey, 50_000)
            .await
            .unwrap();
        vault_program_client
            .do_cooldown_delegation(&vault_root, &operator_roots[1].operator_pubkey, 25_000)
            .await
            .unwrap();

        let config = vault_program_client
            .get_config(&Config::find_program_address(&jito_vault_program::id()).0)
            .await
            .unwrap();
        fixture
            .warp_slot_incremental(config.epoch_length())
            .await
            .unwrap();

        // enqueued for cool down assets are now cooling down

        let slot = fixture.get_current_slot().await.unwrap();
        let ncn_epoch = slot / config.epoch_length();

        let vault_update_state_tracker_pubkey = VaultUpdateStateTracker::find_program_address(
            &jito_vault_program::id(),
            &vault_root.vault_pubkey,
            ncn_epoch,
        )
        .0;
        vault_program_client
            .initialize_vault_update_state_tracker(
                &vault_root.vault_pubkey,
                &vault_update_state_tracker_pubkey,
            )
            .await
            .unwrap();

        vault_program_client
            .do_crank_vault_update_state_tracker(
                &vault_root.vault_pubkey,
                &operator_roots[0].operator_pubkey,
            )
            .await
            .unwrap();
        // skip index 1, advance to next epoch

        fixture
            .warp_slot_incremental(config.epoch_length())
            .await
            .unwrap();

        // cooldown assets are now inactive

        let slot = fixture.get_current_slot().await.unwrap();
        let ncn_epoch = slot / config.epoch_length();
        let vault_update_state_tracker_pubkey = VaultUpdateStateTracker::find_program_address(
            &jito_vault_program::id(),
            &vault_root.vault_pubkey,
            ncn_epoch,
        )
        .0;
        vault_program_client
            .initialize_vault_update_state_tracker(
                &vault_root.vault_pubkey,
                &vault_update_state_tracker_pubkey,
            )
            .await
            .unwrap();
        vault_program_client
            .do_crank_vault_update_state_tracker(
                &vault_root.vault_pubkey,
                &operator_roots[0].operator_pubkey,
            )
            .await
            .unwrap();
        let vault_update_state_tracker = vault_program_client
            .get_vault_update_state_tracker(&vault_root.vault_pubkey, ncn_epoch)
            .await
            .unwrap();
        assert_eq!(vault_update_state_tracker.last_updated_index(), 0);
        assert_eq!(
            vault_update_state_tracker.delegation_state,
            DelegationState::new(25000, 0, 0)
        );

        // active -> cooldown (2 epochs since last update)
        vault_program_client
            .do_crank_vault_update_state_tracker(
                &vault_root.vault_pubkey,
                &operator_roots[1].operator_pubkey,
            )
            .await
            .unwrap();
        let vault_update_state_tracker = vault_program_client
            .get_vault_update_state_tracker(&vault_root.vault_pubkey, ncn_epoch)
            .await
            .unwrap();
        assert_eq!(vault_update_state_tracker.last_updated_index(), 1);
        assert_eq!(
            vault_update_state_tracker.delegation_state,
            DelegationState::new(50000, 0, 0)
        );
    }
}
