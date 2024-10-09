#[cfg(test)]
mod tests {
    use jito_vault_core::{
        config::Config, vault::Vault, vault_update_state_tracker::VaultUpdateStateTracker,
    };
    use jito_vault_sdk::error::VaultError;
    use solana_sdk::signature::{Keypair, Signer};

    use crate::fixtures::{
        fixture::{ConfiguredVault, TestBuilder},
        vault_client::{assert_vault_error, VaultStakerWithdrawalTicketRoot},
    };

    #[tokio::test]
    async fn test_close_update_state_tracker_no_operators_ok() {
        let mut fixture = TestBuilder::new().await;

        let deposit_fee_bps = 0;
        let withdraw_fee_bps = 0;
        let reward_fee_bps = 0;
        let num_operators = 0;
        let slasher_amounts = vec![];

        let ConfiguredVault {
            mut vault_program_client,
            vault_root,
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

        vault_program_client
            .close_vault_update_state_tracker(
                &vault_root.vault_pubkey,
                &VaultUpdateStateTracker::find_program_address(
                    &jito_vault_program::id(),
                    &vault_root.vault_pubkey,
                    ncn_epoch,
                )
                .0,
                ncn_epoch,
            )
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_close_update_state_tracker_not_finished_fails() {
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
                deposit_fee_bps,
                withdraw_fee_bps,
                reward_fee_bps,
                num_operators,
                &slasher_amounts,
            )
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

        vault_program_client
            .do_crank_vault_update_state_tracker(
                &vault_root.vault_pubkey,
                &operator_roots[0].operator_pubkey,
            )
            .await
            .unwrap();

        // skip index 1

        let result = vault_program_client
            .close_vault_update_state_tracker(
                &vault_root.vault_pubkey,
                &VaultUpdateStateTracker::find_program_address(
                    &jito_vault_program::id(),
                    &vault_root.vault_pubkey,
                    ncn_epoch,
                )
                .0,
                ncn_epoch,
            )
            .await;
        assert_vault_error(result, VaultError::VaultUpdateStateNotFinishedUpdating);
    }

    #[tokio::test]
    async fn test_close_update_state_tracker_old_epoch_ok() {
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

        let config = vault_program_client
            .get_config(&Config::find_program_address(&jito_vault_program::id()).0)
            .await
            .unwrap();

        fixture
            .warp_slot_incremental(config.epoch_length())
            .await
            .unwrap();

        let slot = fixture.get_current_slot().await.unwrap();
        let old_ncn_epoch = slot / config.epoch_length();
        let old_vault_update_state_tracker = VaultUpdateStateTracker::find_program_address(
            &jito_vault_program::id(),
            &vault_root.vault_pubkey,
            old_ncn_epoch,
        )
        .0;
        vault_program_client
            .initialize_vault_update_state_tracker(
                &vault_root.vault_pubkey,
                &old_vault_update_state_tracker,
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

        vault_program_client
            .do_crank_vault_update_state_tracker(
                &vault_root.vault_pubkey,
                &operator_roots[1].operator_pubkey,
            )
            .await
            .unwrap();

        // leave the update state tracker open
        // advance epoch

        fixture
            .warp_slot_incremental(5 * config.epoch_length())
            .await
            .unwrap();

        vault_program_client
            .close_vault_update_state_tracker(
                &vault_root.vault_pubkey,
                &old_vault_update_state_tracker,
                old_ncn_epoch,
            )
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_close_update_state_tracker_vrt_enqueued_ok() {
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
            .do_add_delegation(&vault_root, &operator_roots[0].operator_pubkey, 100_000)
            .await
            .unwrap();

        let vault = vault_program_client
            .get_vault(&vault_root.vault_pubkey)
            .await
            .unwrap();

        let min_amount_out = vault
            .calculate_min_supported_mint_out(100_000, Vault::MIN_WITHDRAWAL_SLIPPAGE_BPS)
            .unwrap();

        let VaultStakerWithdrawalTicketRoot { base: _ } = vault_program_client
            .do_enqueue_withdraw(&vault_root, &depositor, 100_000, min_amount_out)
            .await
            .unwrap();
        let vault = vault_program_client
            .get_vault(&vault_root.vault_pubkey)
            .await
            .unwrap();
        assert_eq!(vault.vrt_enqueued_for_cooldown_amount(), 100_000);
        assert_eq!(vault.vrt_cooling_down_amount(), 0);
        assert_eq!(vault.vrt_ready_to_claim_amount(), 0);

        let config = vault_program_client
            .get_config(&Config::find_program_address(&jito_vault_program::id()).0)
            .await
            .unwrap();
        fixture
            .warp_slot_incremental(config.epoch_length())
            .await
            .unwrap();

        let operator_pubkeys: Vec<_> = operator_roots
            .iter()
            .map(|root| root.operator_pubkey)
            .collect();
        vault_program_client
            .do_full_vault_update(&vault_root.vault_pubkey, &operator_pubkeys)
            .await
            .unwrap();
        let vault = vault_program_client
            .get_vault(&vault_root.vault_pubkey)
            .await
            .unwrap();
        assert_eq!(vault.vrt_enqueued_for_cooldown_amount(), 0);
        assert_eq!(vault.vrt_cooling_down_amount(), 100_000);
        assert_eq!(vault.vrt_ready_to_claim_amount(), 0);

        fixture
            .warp_slot_incremental(config.epoch_length())
            .await
            .unwrap();
        vault_program_client
            .do_full_vault_update(&vault_root.vault_pubkey, &operator_pubkeys)
            .await
            .unwrap();
        let vault = vault_program_client
            .get_vault(&vault_root.vault_pubkey)
            .await
            .unwrap();
        assert_eq!(vault.vrt_enqueued_for_cooldown_amount(), 0);
        assert_eq!(vault.vrt_cooling_down_amount(), 0);
        assert_eq!(vault.vrt_ready_to_claim_amount(), 100_000);
    }
}
