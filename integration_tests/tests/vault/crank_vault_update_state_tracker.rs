#[cfg(test)]
mod tests {
    use crate::fixtures::fixture::{ConfiguredVault, TestBuilder};
    use jito_vault_core::config::Config;
    use jito_vault_core::delegation_state::DelegationState;
    use jito_vault_core::vault_update_state_tracker::VaultUpdateStateTracker;
    use solana_sdk::signature::Keypair;

    #[tokio::test]
    async fn test_crank_vault_update_state_tracker_ok() {
        let mut fixture = TestBuilder::new().await;
        let ConfiguredVault {
            mut vault_program_client,
            restaking_program_client,
            vault_config_admin,
            vault_root,
            restaking_config_admin,
            ncn_root,
            operator_roots,
            slashers_amounts,
        } = fixture
            .setup_vault_with_ncn_and_operators(0, 0, 1, &[])
            .await
            .unwrap();

        let depositor = Keypair::new();
        vault_program_client
            .configure_depositor(&vault_root, &depositor, 100_000)
            .await
            .unwrap();
        vault_program_client
            .do_mint_to(&vault_root, &depositor, 100_000)
            .await
            .unwrap();

        let config = vault_program_client
            .get_config(&Config::find_program_address(&jito_vault_program::id()).0)
            .await
            .unwrap();

        // go to next epoch to force update
        fixture
            .warp_slot_incremental(config.epoch_length)
            .await
            .unwrap();

        let slot = fixture.get_current_slot().await.unwrap();
        let ncn_epoch = slot / config.epoch_length;
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
        assert_eq!(vault_update_state_tracker.ncn_epoch, ncn_epoch);
        assert_eq!(vault_update_state_tracker.last_updated_index, u64::MAX);
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
        assert_eq!(vault_update_state_tracker.last_updated_index, 0);
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
        assert_eq!(operator_delegation.last_update_slot, slot);
    }

    #[tokio::test]
    async fn test_crank_vault_update_state_tracker_multiple_operators_ok() {}

    #[tokio::test]
    async fn test_crank_vault_update_state_tracker_out_of_order_fails() {}

    #[tokio::test]
    async fn test_crank_vault_update_state_tracker_partial_update_previous_epoch_ok() {}

    #[tokio::test]
    async fn test_crank_vault_update_state_tracker_with_staked_operators_ok() {}
}
