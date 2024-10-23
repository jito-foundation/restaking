#[cfg(test)]
mod tests {
    use jito_vault_core::{
        config::Config, delegation_state::DelegationState,
        vault_update_state_tracker::VaultUpdateStateTracker,
    };
    use jito_vault_sdk::error::VaultError;
    use solana_sdk::signature::{Keypair, Signer};
    use spl_associated_token_account::get_associated_token_address;

    use crate::fixtures::{
        fixture::{ConfiguredVault, TestBuilder},
        vault_client::{assert_vault_error, VaultStakerWithdrawalTicketRoot},
    };

    #[tokio::test]
    async fn test_crank_vault_update_state_tracker_ok() {
        let mut fixture = TestBuilder::new().await;

        let deposit_fee_bps = 0;
        let withdrawal_fee_bps = 0;
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
                deposit_fee_bps,
                withdrawal_fee_bps,
                reward_fee_bps,
                num_operators,
                &slasher_amounts,
            )
            .await
            .unwrap();

        let depositor = Keypair::new();
        vault_program_client
            .configure_depositor(&vault_root, &depositor.pubkey(), 100_000)
            .await
            .unwrap();
        vault_program_client
            .do_mint_to(&vault_root, &depositor, 100_000, 100_000)
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

    #[tokio::test]
    async fn test_crank_vault_update_state_tracker_multiple_operators_ok() {
        let mut fixture = TestBuilder::new().await;

        let deposit_fee_bps = 0;
        let withdrawal_fee_bps = 0;
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
                deposit_fee_bps,
                withdrawal_fee_bps,
                reward_fee_bps,
                num_operators,
                &slasher_amounts,
            )
            .await
            .unwrap();

        let depositor = Keypair::new();
        vault_program_client
            .configure_depositor(&vault_root, &depositor.pubkey(), 100_000)
            .await
            .unwrap();
        vault_program_client
            .do_mint_to(&vault_root, &depositor, 100_000, 100_000)
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

    #[tokio::test]
    async fn test_crank_vault_update_state_tracker_same_index_twice_fails() {
        let mut fixture = TestBuilder::new().await;

        let deposit_fee_bps = 0;
        let withdrawal_fee_bps = 0;
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
                deposit_fee_bps,
                withdrawal_fee_bps,
                reward_fee_bps,
                num_operators,
                &slasher_amounts,
            )
            .await
            .unwrap();

        let depositor = Keypair::new();
        vault_program_client
            .configure_depositor(&vault_root, &depositor.pubkey(), 100_000)
            .await
            .unwrap();
        vault_program_client
            .do_mint_to(&vault_root, &depositor, 100_000, 100_000)
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
        assert_vault_error(result, VaultError::VaultOperatorDelegationIsUpdated);
    }

    #[tokio::test]
    async fn test_crank_vault_update_state_tracker_skip_zero_fails() {
        let mut fixture = TestBuilder::new().await;

        let deposit_fee_bps = 0;
        let withdrawal_fee_bps = 0;
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
                deposit_fee_bps,
                withdrawal_fee_bps,
                reward_fee_bps,
                num_operators,
                &slasher_amounts,
            )
            .await
            .unwrap();

        let depositor = Keypair::new();
        vault_program_client
            .configure_depositor(&vault_root, &depositor.pubkey(), 100_000)
            .await
            .unwrap();
        vault_program_client
            .do_mint_to(&vault_root, &depositor, 100_000, 100_000)
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

    #[tokio::test]
    async fn test_crank_vault_update_state_tracker_skip_index_fails() {
        let mut fixture = TestBuilder::new().await;

        let deposit_fee_bps = 0;
        let withdrawal_fee_bps = 0;
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
                deposit_fee_bps,
                withdrawal_fee_bps,
                reward_fee_bps,
                num_operators,
                &slasher_amounts,
            )
            .await
            .unwrap();

        let depositor = Keypair::new();
        vault_program_client
            .configure_depositor(&vault_root, &depositor.pubkey(), 100_000)
            .await
            .unwrap();
        vault_program_client
            .do_mint_to(&vault_root, &depositor, 100_000, 100_000)
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

    #[tokio::test]
    async fn test_crank_vault_update_state_tracker_partial_update_previous_epoch_ok() {
        let mut fixture = TestBuilder::new().await;

        let deposit_fee_bps = 0;
        let withdrawal_fee_bps = 0;
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
                deposit_fee_bps,
                withdrawal_fee_bps,
                reward_fee_bps,
                num_operators,
                &slasher_amounts,
            )
            .await
            .unwrap();

        let depositor = Keypair::new();
        vault_program_client
            .configure_depositor(&vault_root, &depositor.pubkey(), 100_000)
            .await
            .unwrap();
        vault_program_client
            .do_mint_to(&vault_root, &depositor, 100_000, 100_000)
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

    /// Test that the vrt withdrawal process if updating with normal crank operations
    #[tokio::test]
    async fn test_withdraw_state_crank_ok() {
        const MINT_AMOUNT: u64 = 100_000;

        let deposit_fee_bps = 0;
        let withdraw_fee_bps = 0;
        let reward_fee_bps = 0;
        let num_operators = 1;
        let slasher_amounts = vec![];

        let mut fixture = TestBuilder::new().await;
        let ConfiguredVault {
            mut vault_program_client,
            restaking_program_client: _,
            vault_config_admin: _,
            vault_root,
            restaking_config_admin: _,
            operator_roots,
        } = fixture
            .setup_vault_with_ncn_and_operators(
                deposit_fee_bps,
                withdraw_fee_bps,
                reward_fee_bps,
                num_operators,
                &slasher_amounts,
            )
            .await
            .unwrap();

        // Initial deposit + mint
        let depositor = Keypair::new();
        vault_program_client
            .configure_depositor(&vault_root, &depositor.pubkey(), MINT_AMOUNT)
            .await
            .unwrap();
        vault_program_client
            .do_mint_to(&vault_root, &depositor, MINT_AMOUNT, MINT_AMOUNT)
            .await
            .unwrap();

        let config = vault_program_client
            .get_config(&Config::find_program_address(&jito_vault_program::id()).0)
            .await
            .unwrap();

        // Delegate all funds to the operator
        vault_program_client
            .do_add_delegation(&vault_root, &operator_roots[0].operator_pubkey, MINT_AMOUNT)
            .await
            .unwrap();

        let VaultStakerWithdrawalTicketRoot { base } = vault_program_client
            .do_enqueue_withdrawal(&vault_root, &depositor, MINT_AMOUNT)
            .await
            .unwrap();

        vault_program_client
            .do_cooldown_delegation(&vault_root, &operator_roots[0].operator_pubkey, MINT_AMOUNT)
            .await
            .unwrap();

        let vault = vault_program_client
            .get_vault(&vault_root.vault_pubkey)
            .await
            .unwrap();

        assert_eq!(vault.vrt_enqueued_for_cooldown_amount(), MINT_AMOUNT);

        fixture
            .warp_slot_incremental(config.epoch_length())
            .await
            .unwrap();
        vault_program_client
            .do_full_vault_update(
                &vault_root.vault_pubkey,
                &[operator_roots[0].operator_pubkey],
            )
            .await
            .unwrap();

        let vault = vault_program_client
            .get_vault(&vault_root.vault_pubkey)
            .await
            .unwrap();

        assert_eq!(vault.vrt_enqueued_for_cooldown_amount(), 0);
        assert_eq!(vault.vrt_cooling_down_amount(), MINT_AMOUNT);

        fixture
            .warp_slot_incremental(config.epoch_length())
            .await
            .unwrap();
        vault_program_client
            .do_full_vault_update(
                &vault_root.vault_pubkey,
                &[operator_roots[0].operator_pubkey],
            )
            .await
            .unwrap();

        let vault = vault_program_client
            .get_vault(&vault_root.vault_pubkey)
            .await
            .unwrap();

        assert_eq!(vault.vrt_enqueued_for_cooldown_amount(), 0);
        assert_eq!(vault.vrt_cooling_down_amount(), 0);
        assert_eq!(vault.vrt_ready_to_claim_amount(), MINT_AMOUNT);

        vault_program_client
            .do_burn_withdrawal_ticket(&vault_root, &depositor, &base, &config.program_fee_wallet)
            .await
            .unwrap();

        let vault = vault_program_client
            .get_vault(&vault_root.vault_pubkey)
            .await
            .unwrap();
        assert_eq!(vault.tokens_deposited(), 0);
        assert_eq!(vault.vrt_supply(), 0);
        assert_eq!(vault.delegation_state, DelegationState::default());
        assert_eq!(vault.vrt_enqueued_for_cooldown_amount(), 0);
        assert_eq!(vault.vrt_ready_to_claim_amount(), 0);
        assert_eq!(vault.vrt_cooling_down_amount(), 0);

        let depositor_token_account = fixture
            .get_token_account(&get_associated_token_address(
                &depositor.pubkey(),
                &vault.supported_mint,
            ))
            .await
            .unwrap();
        assert_eq!(depositor_token_account.amount, MINT_AMOUNT);
    }

    /// Test that the vrt withdrawal process if updating when skipping one epoch
    #[tokio::test]
    async fn test_withdraw_state_crank_skip_one_update_epoch_ok() {
        const MINT_AMOUNT: u64 = 100_000;

        let deposit_fee_bps = 0;
        let withdraw_fee_bps = 0;
        let reward_fee_bps = 0;
        let num_operators = 1;
        let slasher_amounts = vec![];

        let mut fixture = TestBuilder::new().await;
        let ConfiguredVault {
            mut vault_program_client,
            restaking_program_client: _,
            vault_config_admin: _,
            vault_root,
            restaking_config_admin: _,
            operator_roots,
        } = fixture
            .setup_vault_with_ncn_and_operators(
                deposit_fee_bps,
                withdraw_fee_bps,
                reward_fee_bps,
                num_operators,
                &slasher_amounts,
            )
            .await
            .unwrap();

        // Initial deposit + mint
        let depositor = Keypair::new();
        vault_program_client
            .configure_depositor(&vault_root, &depositor.pubkey(), MINT_AMOUNT)
            .await
            .unwrap();
        vault_program_client
            .do_mint_to(&vault_root, &depositor, MINT_AMOUNT, MINT_AMOUNT)
            .await
            .unwrap();

        let config = vault_program_client
            .get_config(&Config::find_program_address(&jito_vault_program::id()).0)
            .await
            .unwrap();

        // Delegate all funds to the operator
        vault_program_client
            .do_add_delegation(&vault_root, &operator_roots[0].operator_pubkey, MINT_AMOUNT)
            .await
            .unwrap();

        let VaultStakerWithdrawalTicketRoot { base } = vault_program_client
            .do_enqueue_withdrawal(&vault_root, &depositor, MINT_AMOUNT)
            .await
            .unwrap();

        vault_program_client
            .do_cooldown_delegation(&vault_root, &operator_roots[0].operator_pubkey, MINT_AMOUNT)
            .await
            .unwrap();

        let vault = vault_program_client
            .get_vault(&vault_root.vault_pubkey)
            .await
            .unwrap();

        assert_eq!(vault.vrt_enqueued_for_cooldown_amount(), MINT_AMOUNT);

        fixture
            .warp_slot_incremental(config.epoch_length() * 2)
            .await
            .unwrap();
        vault_program_client
            .do_full_vault_update(
                &vault_root.vault_pubkey,
                &[operator_roots[0].operator_pubkey],
            )
            .await
            .unwrap();

        let vault = vault_program_client
            .get_vault(&vault_root.vault_pubkey)
            .await
            .unwrap();

        assert_eq!(vault.vrt_enqueued_for_cooldown_amount(), 0);
        assert_eq!(vault.vrt_cooling_down_amount(), 0);
        assert_eq!(vault.vrt_ready_to_claim_amount(), MINT_AMOUNT);

        vault_program_client
            .do_burn_withdrawal_ticket(&vault_root, &depositor, &base, &config.program_fee_wallet)
            .await
            .unwrap();

        let vault = vault_program_client
            .get_vault(&vault_root.vault_pubkey)
            .await
            .unwrap();
        assert_eq!(vault.tokens_deposited(), 0);
        assert_eq!(vault.vrt_supply(), 0);
        assert_eq!(vault.delegation_state, DelegationState::default());
        assert_eq!(vault.vrt_enqueued_for_cooldown_amount(), 0);
        assert_eq!(vault.vrt_ready_to_claim_amount(), 0);
        assert_eq!(vault.vrt_cooling_down_amount(), 0);

        let depositor_token_account = fixture
            .get_token_account(&get_associated_token_address(
                &depositor.pubkey(),
                &vault.supported_mint,
            ))
            .await
            .unwrap();
        assert_eq!(depositor_token_account.amount, MINT_AMOUNT);
    }

    /// Test that the vrt withdrawal process if updating with normal crank operations
    #[tokio::test]
    async fn test_withdraw_state_crank_skip_many_update_epochs_ok() {
        const MINT_AMOUNT: u64 = 100_000;

        let deposit_fee_bps = 0;
        let withdraw_fee_bps = 0;
        let reward_fee_bps = 0;
        let num_operators = 1;
        let slasher_amounts = vec![];

        let mut fixture = TestBuilder::new().await;
        let ConfiguredVault {
            mut vault_program_client,
            restaking_program_client: _,
            vault_config_admin: _,
            vault_root,
            restaking_config_admin: _,
            operator_roots,
        } = fixture
            .setup_vault_with_ncn_and_operators(
                deposit_fee_bps,
                withdraw_fee_bps,
                reward_fee_bps,
                num_operators,
                &slasher_amounts,
            )
            .await
            .unwrap();

        // Initial deposit + mint
        let depositor = Keypair::new();
        vault_program_client
            .configure_depositor(&vault_root, &depositor.pubkey(), MINT_AMOUNT)
            .await
            .unwrap();
        vault_program_client
            .do_mint_to(&vault_root, &depositor, MINT_AMOUNT, MINT_AMOUNT)
            .await
            .unwrap();

        let config = vault_program_client
            .get_config(&Config::find_program_address(&jito_vault_program::id()).0)
            .await
            .unwrap();

        // Delegate all funds to the operator
        vault_program_client
            .do_add_delegation(&vault_root, &operator_roots[0].operator_pubkey, MINT_AMOUNT)
            .await
            .unwrap();

        let VaultStakerWithdrawalTicketRoot { base } = vault_program_client
            .do_enqueue_withdrawal(&vault_root, &depositor, MINT_AMOUNT)
            .await
            .unwrap();

        vault_program_client
            .do_cooldown_delegation(&vault_root, &operator_roots[0].operator_pubkey, MINT_AMOUNT)
            .await
            .unwrap();

        let vault = vault_program_client
            .get_vault(&vault_root.vault_pubkey)
            .await
            .unwrap();

        assert_eq!(vault.vrt_enqueued_for_cooldown_amount(), MINT_AMOUNT);

        fixture
            .warp_slot_incremental(config.epoch_length() * 500)
            .await
            .unwrap();
        vault_program_client
            .do_full_vault_update(
                &vault_root.vault_pubkey,
                &[operator_roots[0].operator_pubkey],
            )
            .await
            .unwrap();

        let vault = vault_program_client
            .get_vault(&vault_root.vault_pubkey)
            .await
            .unwrap();

        assert_eq!(vault.vrt_enqueued_for_cooldown_amount(), 0);
        assert_eq!(vault.vrt_cooling_down_amount(), 0);
        assert_eq!(vault.vrt_ready_to_claim_amount(), MINT_AMOUNT);

        vault_program_client
            .do_burn_withdrawal_ticket(&vault_root, &depositor, &base, &config.program_fee_wallet)
            .await
            .unwrap();

        let vault = vault_program_client
            .get_vault(&vault_root.vault_pubkey)
            .await
            .unwrap();
        assert_eq!(vault.tokens_deposited(), 0);
        assert_eq!(vault.vrt_supply(), 0);
        assert_eq!(vault.delegation_state, DelegationState::default());
        assert_eq!(vault.vrt_enqueued_for_cooldown_amount(), 0);
        assert_eq!(vault.vrt_ready_to_claim_amount(), 0);
        assert_eq!(vault.vrt_cooling_down_amount(), 0);

        let depositor_token_account = fixture
            .get_token_account(&get_associated_token_address(
                &depositor.pubkey(),
                &vault.supported_mint,
            ))
            .await
            .unwrap();
        assert_eq!(depositor_token_account.amount, MINT_AMOUNT);
    }

    #[tokio::test]
    async fn test_crank_vault_update_state_tracker_vault_is_paused_fails() {
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
            .configure_depositor(&vault_root, &depositor.pubkey(), 100_000)
            .await
            .unwrap();
        vault_program_client
            .do_mint_to(&vault_root, &depositor, 100_000, 100_000)
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
            .set_is_paused(&vault_root.vault_pubkey, &vault_root.vault_admin, true)
            .await
            .unwrap();

        let test_error = vault_program_client
            .do_crank_vault_update_state_tracker(
                &vault_root.vault_pubkey,
                &operator_roots[2].operator_pubkey,
            )
            .await;

        assert_vault_error(test_error, VaultError::VaultIsPaused);
    }
}
