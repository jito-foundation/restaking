#[cfg(test)]
mod tests {
    use jito_vault_core::{
        config::Config, vault::Vault, vault_update_state_tracker::VaultUpdateStateTracker,
    };
    use jito_vault_sdk::error::VaultError;
    use solana_program::instruction::InstructionError;
    use solana_sdk::{signature::Keypair, signer::Signer};

    use crate::fixtures::{
        assert_ix_error,
        fixture::{ConfiguredVault, TestBuilder},
        vault_client::assert_vault_error,
    };

    #[tokio::test]
    async fn test_initialize_vault_update_state_tracker_ok() {
        let mut fixture = TestBuilder::new().await;
        let mut vault_program_client = fixture.vault_program_client();
        let (_vault_config_admin, vault_root) = vault_program_client
            .setup_config_and_vault(0, 0, 0)
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

    #[tokio::test]
    async fn test_initialize_vault_update_state_tracker_no_operators_fails() {
        let mut fixture = TestBuilder::new().await;
        let mut vault_program_client = fixture.vault_program_client();
        let (_vault_config_admin, vault_root) = vault_program_client
            .setup_config_and_vault(0, 0, 0)
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

    #[tokio::test]
    async fn test_initialize_vault_update_state_tracker_already_initialized_fails() {
        let mut fixture = TestBuilder::new().await;
        let mut vault_program_client = fixture.vault_program_client();
        let (_vault_config_admin, vault_root) = vault_program_client
            .setup_config_and_vault(0, 0, 0)
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

    #[tokio::test]
    async fn test_initialize_vault_update_state_tracker_vault_is_paused_fails() {
        let mut fixture = TestBuilder::new().await;
        let mut vault_program_client = fixture.vault_program_client();
        let (_vault_config_admin, vault_root) = vault_program_client
            .setup_config_and_vault(0, 0, 0)
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

        vault_program_client
            .set_is_paused(&vault_root.vault_pubkey, &vault_root.vault_admin, true)
            .await
            .unwrap();

        let slot = fixture.get_current_slot().await.unwrap();
        let test_error = vault_program_client
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

        assert_vault_error(test_error, VaultError::VaultIsPaused);
    }

    #[tokio::test]
    async fn test_initialize_vault_update_state_tracker_fees_updated() {
        // Fees are updated if the vault update state tracker is fully updated
        let mut fixture = TestBuilder::new().await;
        let mut vault_program_client = fixture.vault_program_client();
        let (vault_config_admin, vault_root) = vault_program_client
            .setup_config_and_vault(0, 0, 0)
            .await
            .unwrap();
        let config_pubkey = Config::find_program_address(&jito_vault_program::id()).0;
        let vault_config = vault_program_client
            .get_config(&config_pubkey)
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

        // Check initial fees are 0
        let vault = vault_program_client
            .get_vault(&vault_root.vault_pubkey)
            .await
            .unwrap();
        assert_eq!(vault.withdrawal_fee_bps(), 0);
        assert_eq!(vault.next_withdrawal_fee_bps(), 0);
        assert_eq!(vault.program_fee_bps(), 0);

        fixture
            .warp_slot_incremental(2 * vault_config.epoch_length())
            .await
            .unwrap();

        // Do full update
        vault_program_client
            .do_full_vault_update(&vault_root.vault_pubkey, &[operator_root.operator_pubkey])
            .await
            .unwrap();

        // Set fees
        let new_withdrawal_fee_bps = 10;
        let new_program_fee_bps = 11;

        vault_program_client
            .set_fees(
                &config_pubkey,
                &vault_root.vault_pubkey,
                &vault_root.vault_admin,
                None,
                Some(new_withdrawal_fee_bps),
                None,
            )
            .await
            .unwrap();

        vault_program_client
            .set_program_fee(&vault_config_admin, new_program_fee_bps)
            .await
            .unwrap();

        // Vault fees not updated yet
        let vault = vault_program_client
            .get_vault(&vault_root.vault_pubkey)
            .await
            .unwrap();
        assert_eq!(vault.withdrawal_fee_bps(), 0);
        assert_eq!(vault.next_withdrawal_fee_bps(), new_withdrawal_fee_bps);
        assert_eq!(vault.program_fee_bps(), 0);

        fixture
            .warp_slot_incremental(1 * vault_config.epoch_length())
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
        // Check fees are updated
        let vault = vault_program_client
            .get_vault(&vault_root.vault_pubkey)
            .await
            .unwrap();
        assert_eq!(vault.withdrawal_fee_bps(), new_withdrawal_fee_bps);
        assert_eq!(vault.program_fee_bps(), new_program_fee_bps);
    }

    #[tokio::test]
    async fn test_initialize_vault_update_state_tracker_fees_not_updated() {
        // Fees are not updated if the vault update state tracker is not fully updated
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
            vault_config_admin,
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
            .do_add_delegation(&vault_root, &operator_roots[0].operator_pubkey, 100_000)
            .await
            .unwrap();

        vault_program_client
            .do_enqueue_withdrawal(&vault_root, &depositor, None, 75_000)
            .await
            .unwrap();

        let config_address = Config::find_program_address(&jito_vault_program::id()).0;
        let config = vault_program_client
            .get_config(&config_address)
            .await
            .unwrap();

        fixture
            .warp_slot_incremental(1 * config.epoch_length())
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

        let vault = vault_program_client
            .get_vault(&vault_root.vault_pubkey)
            .await
            .unwrap();

        assert_eq!(
            vault.additional_assets_need_unstaking(),
            75_000 - Vault::DEFAULT_INITIALIZATION_TOKEN_AMOUNT
        );

        // skip cranking operator 0, advance to next epoch

        fixture
            .warp_slot_incremental(config.epoch_length())
            .await
            .unwrap();

        // Update fees
        let new_withdrawal_fee_bps = 10;
        let new_program_fee_bps = 11;
        vault_program_client
            .set_fees(
                &config_address,
                &vault_root.vault_pubkey,
                &vault_root.vault_admin,
                None,
                Some(new_withdrawal_fee_bps),
                None,
            )
            .await
            .unwrap();
        vault_program_client
            .set_program_fee(&vault_config_admin, new_program_fee_bps)
            .await
            .unwrap();

        let slot = fixture.get_current_slot().await.unwrap();
        let ncn_epoch = slot / config.epoch_length();
        // no assets cooled down, additional_assets_need_unstaking = 75_000
        let vault = vault_program_client
            .get_vault(&vault_root.vault_pubkey)
            .await
            .unwrap();

        // no assets cooled down, additional_assets_need_unstaking = 75_000
        assert_eq!(
            vault.additional_assets_need_unstaking(),
            75_000 - Vault::DEFAULT_INITIALIZATION_TOKEN_AMOUNT
        );

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

        let vault = vault_program_client
            .get_vault(&vault_root.vault_pubkey)
            .await
            .unwrap();

        // Check fees not updated because of partial update
        assert_eq!(vault.withdrawal_fee_bps(), 0);
        assert_eq!(vault.program_fee_bps(), 0);

        vault_program_client
            .do_crank_vault_update_state_tracker(
                &vault_root.vault_pubkey,
                &operator_roots[0].operator_pubkey,
            )
            .await
            .unwrap();

        let vault = vault_program_client
            .get_vault(&vault_root.vault_pubkey)
            .await
            .unwrap();

        // Assets now cooled down
        assert_eq!(vault.additional_assets_need_unstaking(), 0);

        vault_program_client
            .close_vault_update_state_tracker(
                &vault_root.vault_pubkey,
                &vault_update_state_tracker_pubkey,
                ncn_epoch,
            )
            .await
            .unwrap();

        let vault = vault_program_client
            .get_vault(&vault_root.vault_pubkey)
            .await
            .unwrap();

        assert_eq!(
            vault.delegation_state.staked_amount(),
            25_000 + Vault::DEFAULT_INITIALIZATION_TOKEN_AMOUNT
        );
        assert_eq!(vault.delegation_state.enqueued_for_cooldown_amount(), 0);
        assert_eq!(vault.vrt_ready_to_claim_amount(), 75_000);
    }
}
