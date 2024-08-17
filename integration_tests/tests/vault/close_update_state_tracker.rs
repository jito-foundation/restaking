#[cfg(test)]
mod tests {
    use crate::fixtures::fixture::{ConfiguredVault, TestBuilder};
    use jito_vault_core::config::Config;
    use jito_vault_core::vault_update_state_tracker::VaultUpdateStateTracker;

    #[tokio::test]
    async fn test_close_update_state_tracker_no_operators_ok() {
        let mut fixture = TestBuilder::new().await;

        let ConfiguredVault {
            mut vault_program_client,
            vault_root,
            ..
        } = fixture
            .setup_vault_with_ncn_and_operators(0, 0, 0, &[])
            .await
            .unwrap();

        let config = vault_program_client
            .get_config(&Config::find_program_address(&jito_vault_program::id()).0)
            .await
            .unwrap();

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

        vault_program_client
            .close_vault_update_state_tracker(
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
    }

    #[tokio::test]
    async fn test_close_update_state_tracker_not_finished_fails() {}

    #[tokio::test]
    async fn test_close_update_state_tracker_old_epoch_ok() {}
}
