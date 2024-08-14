#[cfg(test)]
mod tests {
    use jito_vault_core::config::Config;
    use jito_vault_sdk::{error::VaultError, instruction::VaultAdminRole};
    use solana_sdk::{
        pubkey::Pubkey,
        signature::{Keypair, Signer},
    };

    use crate::fixtures::{
        fixture::TestBuilder,
        vault_client::{assert_vault_error, VaultRoot},
        TestError,
    };

    async fn setup_test_vault(
        fixture: &mut TestBuilder,
        deposit_fee_bps: u16,
        withdrawal_fee_bps: u16,
    ) -> Result<(Pubkey, Pubkey, Keypair), TestError> {
        let mut vault_program_client = fixture.vault_program_client();

        let result = vault_program_client
            .setup_config_and_vault(deposit_fee_bps, withdrawal_fee_bps, 0)
            .await;

        match result {
            Ok((
                _,
                VaultRoot {
                    vault_pubkey,
                    vault_admin,
                },
            )) => {
                let vault = vault_program_client.get_vault(&vault_pubkey).await.unwrap();
                assert_eq!(vault.deposit_fee_bps, deposit_fee_bps);
                assert_eq!(vault.withdrawal_fee_bps, withdrawal_fee_bps);
                let config_address = Config::find_program_address(&jito_vault_program::id()).0;

                Ok((config_address, vault_pubkey, vault_admin))
            }
            Err(err) => Err(err),
        }
    }

    #[tokio::test]
    async fn test_initialize_vault_with_bad_fees() {
        let mut fixture = TestBuilder::new().await;

        let deposit_fee_bps = u16::MAX;
        let withdrawal_fee_bps = u16::MAX;

        let result = setup_test_vault(&mut fixture, deposit_fee_bps, withdrawal_fee_bps).await;

        assert_vault_error(result, VaultError::VaultFeeCapExceeded);
    }

    #[tokio::test]
    async fn test_initial_fee_setup() {
        let mut fixture = TestBuilder::new().await;
        let deposit_fee_bps = 99;
        let withdrawal_fee_bps = 100;

        let (_, vault_pubkey, _) =
            setup_test_vault(&mut fixture, deposit_fee_bps, withdrawal_fee_bps)
                .await
                .unwrap();

        let vault = fixture
            .vault_program_client()
            .get_vault(&vault_pubkey)
            .await
            .unwrap();

        assert_eq!(vault.deposit_fee_bps, deposit_fee_bps);
        assert_eq!(vault.withdrawal_fee_bps, withdrawal_fee_bps);
    }

    #[tokio::test]
    async fn test_change_fees_after_two_epochs() {
        let mut fixture = TestBuilder::new().await;
        let (config_pubkey, vault_pubkey, vault_admin) =
            setup_test_vault(&mut fixture, 99, 100).await.unwrap();

        let config = fixture
            .vault_program_client()
            .get_config(&config_pubkey)
            .await
            .unwrap();

        fixture
            .warp_slot_incremental(config.epoch_length * 2 + 1)
            .await
            .unwrap();

        let new_deposit_fee_bps = 100;
        let new_withdrawal_fee_bps = 101;

        fixture
            .vault_program_client()
            .set_fees(
                &config_pubkey,
                &vault_pubkey,
                &vault_admin,
                new_deposit_fee_bps,
                new_withdrawal_fee_bps,
            )
            .await
            .unwrap();

        let vault = fixture
            .vault_program_client()
            .get_vault(&vault_pubkey)
            .await
            .unwrap();

        assert_eq!(vault.deposit_fee_bps, new_deposit_fee_bps);
        assert_eq!(vault.withdrawal_fee_bps, new_withdrawal_fee_bps);
    }

    #[tokio::test]
    async fn test_cannot_change_fees_before_epoch_passes() {
        let mut fixture = TestBuilder::new().await;
        let (config_pubkey, vault_pubkey, vault_admin) =
            setup_test_vault(&mut fixture, 99, 100).await.unwrap();

        let result = fixture
            .vault_program_client()
            .set_fees(&config_pubkey, &vault_pubkey, &vault_admin, 150, 150)
            .await;

        assert_vault_error(result, VaultError::VaultFeeChangeTooSoon);
    }

    #[tokio::test]
    async fn test_can_change_fees_after_another_epoch() {
        let mut fixture = TestBuilder::new().await;
        let (config_pubkey, vault_pubkey, vault_admin) =
            setup_test_vault(&mut fixture, 99, 100).await.unwrap();

        let config = fixture
            .vault_program_client()
            .get_config(&config_pubkey)
            .await
            .unwrap();
        fixture
            .warp_slot_incremental(config.epoch_length * 2)
            .await
            .unwrap();

        // First fee change
        fixture
            .vault_program_client()
            .set_fees(&config_pubkey, &vault_pubkey, &vault_admin, 100, 101)
            .await
            .unwrap();

        // Warp again
        fixture
            .warp_slot_incremental(config.epoch_length * 2)
            .await
            .unwrap();

        // Second fee change
        let new_deposit_fee_bps = 101;
        let new_withdrawal_fee_bps = 102;

        fixture
            .vault_program_client()
            .set_fees(
                &config_pubkey,
                &vault_pubkey,
                &vault_admin,
                new_deposit_fee_bps,
                new_withdrawal_fee_bps,
            )
            .await
            .unwrap();

        let vault = fixture
            .vault_program_client()
            .get_vault(&vault_pubkey)
            .await
            .unwrap();
        assert_eq!(vault.deposit_fee_bps, new_deposit_fee_bps);
        assert_eq!(vault.withdrawal_fee_bps, new_withdrawal_fee_bps);
    }

    #[tokio::test]
    async fn test_cannot_change_fees_with_invalid_admin() {
        let mut fixture = TestBuilder::new().await;
        let (config_pubkey, vault_pubkey, _) =
            setup_test_vault(&mut fixture, 99, 100).await.unwrap();

        let bad_admin = Keypair::new();
        fixture
            .vault_program_client()
            ._airdrop(&bad_admin.pubkey(), 10.0)
            .await
            .unwrap();

        let config = fixture
            .vault_program_client()
            .get_config(&config_pubkey)
            .await
            .unwrap();
        fixture
            .warp_slot_incremental(config.epoch_length * 2)
            .await
            .unwrap();

        let result = fixture
            .vault_program_client()
            .set_fees(&config_pubkey, &vault_pubkey, &bad_admin, 150, 150)
            .await;

        assert_vault_error(result, VaultError::VaultFeeAdminInvalid);
    }

    #[tokio::test]
    async fn test_set_fees_regularly() {
        let mut fixture = TestBuilder::new().await;
        let (config_pubkey, vault_pubkey, vault_admin) =
            setup_test_vault(&mut fixture, 99, 100).await.unwrap();

        let config = fixture
            .vault_program_client()
            .get_config(&config_pubkey)
            .await
            .unwrap();
        fixture
            .warp_slot_incremental(config.epoch_length * 2)
            .await
            .unwrap();

        let new_deposit_fee_bps = 100;
        let new_withdrawal_fee_bps = 101;

        fixture
            .vault_program_client()
            .set_fees(
                &config_pubkey,
                &vault_pubkey,
                &vault_admin,
                new_deposit_fee_bps,
                new_withdrawal_fee_bps,
            )
            .await
            .unwrap();

        let vault = fixture
            .vault_program_client()
            .get_vault(&vault_pubkey)
            .await
            .unwrap();
        assert_eq!(vault.deposit_fee_bps, new_deposit_fee_bps);
        assert_eq!(vault.withdrawal_fee_bps, new_withdrawal_fee_bps);
    }

    #[tokio::test]
    async fn test_set_secondary_admin() {
        let mut fixture = TestBuilder::new().await;
        let (config_pubkey, vault_pubkey, vault_admin) =
            setup_test_vault(&mut fixture, 99, 100).await.unwrap();

        let new_admin = Keypair::new();
        fixture
            .vault_program_client()
            ._airdrop(&new_admin.pubkey(), 10.0)
            .await
            .unwrap();

        fixture
            .vault_program_client()
            .set_secondary_admin(
                &config_pubkey,
                &vault_pubkey,
                &vault_admin,
                &new_admin.pubkey(),
                VaultAdminRole::FeeAdmin,
            )
            .await
            .unwrap();

        let vault = fixture
            .vault_program_client()
            .get_vault(&vault_pubkey)
            .await
            .unwrap();
        assert_eq!(vault.fee_admin, new_admin.pubkey());
    }

    #[tokio::test]
    async fn test_set_fees_with_old_admin_fails() {
        let mut fixture = TestBuilder::new().await;
        let (config_pubkey, vault_pubkey, vault_admin) =
            setup_test_vault(&mut fixture, 99, 100).await.unwrap();

        let new_admin = Keypair::new();
        fixture
            .vault_program_client()
            ._airdrop(&new_admin.pubkey(), 10.0)
            .await
            .unwrap();

        fixture
            .vault_program_client()
            .set_secondary_admin(
                &config_pubkey,
                &vault_pubkey,
                &vault_admin,
                &new_admin.pubkey(),
                VaultAdminRole::FeeAdmin,
            )
            .await
            .unwrap();

        let config = fixture
            .vault_program_client()
            .get_config(&config_pubkey)
            .await
            .unwrap();
        fixture
            .warp_slot_incremental(config.epoch_length * 2)
            .await
            .unwrap();

        let result = fixture
            .vault_program_client()
            .set_fees(&config_pubkey, &vault_pubkey, &vault_admin, 299, 300)
            .await;

        assert_vault_error(result, VaultError::VaultFeeAdminInvalid);
    }

    #[tokio::test]
    async fn test_set_fees_with_new_admin_succeeds() {
        let mut fixture = TestBuilder::new().await;
        let (config_pubkey, vault_pubkey, vault_admin) =
            setup_test_vault(&mut fixture, 99, 100).await.unwrap();

        let new_admin = Keypair::new();
        fixture
            .vault_program_client()
            ._airdrop(&new_admin.pubkey(), 10.0)
            .await
            .unwrap();

        fixture
            .vault_program_client()
            .set_secondary_admin(
                &config_pubkey,
                &vault_pubkey,
                &vault_admin,
                &new_admin.pubkey(),
                VaultAdminRole::FeeAdmin,
            )
            .await
            .unwrap();

        let config = fixture
            .vault_program_client()
            .get_config(&config_pubkey)
            .await
            .unwrap();
        fixture
            .warp_slot_incremental(config.epoch_length * 2)
            .await
            .unwrap();

        let new_deposit_fee_bps = 100;
        let new_withdrawal_fee_bps = 101;

        fixture
            .vault_program_client()
            .set_fees(
                &config_pubkey,
                &vault_pubkey,
                &new_admin,
                new_deposit_fee_bps,
                new_withdrawal_fee_bps,
            )
            .await
            .unwrap();

        let vault = fixture
            .vault_program_client()
            .get_vault(&vault_pubkey)
            .await
            .unwrap();
        assert_eq!(vault.deposit_fee_bps, new_deposit_fee_bps);
        assert_eq!(vault.withdrawal_fee_bps, new_withdrawal_fee_bps);
    }

    #[tokio::test]
    async fn test_set_fees_larger_than_cap() {
        let mut fixture = TestBuilder::new().await;
        let (config_pubkey, vault_pubkey, vault_admin) =
            setup_test_vault(&mut fixture, 100, 200).await.unwrap();

        let config = fixture
            .vault_program_client()
            .get_config(&config_pubkey)
            .await
            .unwrap();
        fixture
            .warp_slot_incremental(config.epoch_length * 2)
            .await
            .unwrap();

        let deposit_fee_bps = config.fee_cap_bps + 1;
        let withdrawal_fee_bps = config.fee_cap_bps + 1;

        let result = fixture
            .vault_program_client()
            .set_fees(
                &config_pubkey,
                &vault_pubkey,
                &vault_admin,
                deposit_fee_bps,
                withdrawal_fee_bps,
            )
            .await;

        assert_vault_error(result, VaultError::VaultFeeCapExceeded);
    }

    #[tokio::test]
    async fn test_set_fees_with_ok_change() {
        let mut fixture = TestBuilder::new().await;
        let (config_pubkey, vault_pubkey, vault_admin) =
            setup_test_vault(&mut fixture, 100, 200).await.unwrap();

        let config = fixture
            .vault_program_client()
            .get_config(&config_pubkey)
            .await
            .unwrap();
        let vault = fixture
            .vault_program_client()
            .get_vault(&vault_pubkey)
            .await
            .unwrap();

        let deposit_fee_bps = vault.deposit_fee_bps
            + (config.fee_rate_of_change_bps as u64 * vault.deposit_fee_bps as u64 / 10_000) as u16;
        let withdrawal_fee_bps = vault.withdrawal_fee_bps
            + (config.fee_rate_of_change_bps as u64 * vault.withdrawal_fee_bps as u64 / 10_000)
                as u16;

        fixture
            .warp_slot_incremental(config.epoch_length * 2)
            .await
            .unwrap();

        fixture
            .vault_program_client()
            .set_fees(
                &config_pubkey,
                &vault_pubkey,
                &vault_admin,
                deposit_fee_bps,
                withdrawal_fee_bps,
            )
            .await
            .unwrap();

        let vault = fixture
            .vault_program_client()
            .get_vault(&vault_pubkey)
            .await
            .unwrap();

        assert_eq!(vault.deposit_fee_bps, deposit_fee_bps);
        assert_eq!(vault.withdrawal_fee_bps, withdrawal_fee_bps);
    }

    #[tokio::test]
    async fn test_set_fees_with_too_large_change() {
        let mut fixture = TestBuilder::new().await;
        let (config_pubkey, vault_pubkey, vault_admin) =
            setup_test_vault(&mut fixture, 100, 200).await.unwrap();

        let config = fixture
            .vault_program_client()
            .get_config(&config_pubkey)
            .await
            .unwrap();
        let vault = fixture
            .vault_program_client()
            .get_vault(&vault_pubkey)
            .await
            .unwrap();

        let deposit_fee_bps = vault.deposit_fee_bps
            + (config.fee_rate_of_change_bps as u64 * vault.deposit_fee_bps as u64 / 10_000) as u16
            + 1;
        let withdrawal_fee_bps = vault.withdrawal_fee_bps
            + (config.fee_rate_of_change_bps as u64 * vault.withdrawal_fee_bps as u64 / 10_000)
                as u16
            + 1;

        fixture
            .warp_slot_incremental(config.epoch_length * 2)
            .await
            .unwrap();

        let result = fixture
            .vault_program_client()
            .set_fees(
                &config_pubkey,
                &vault_pubkey,
                &vault_admin,
                deposit_fee_bps,
                withdrawal_fee_bps,
            )
            .await;

        assert_vault_error(result, VaultError::VaultFeeBumpTooLarge);
    }

    #[tokio::test]
    async fn test_set_fees_to_max_bump() {
        let mut fixture = TestBuilder::new().await;
        let (config_pubkey, vault_pubkey, vault_admin) =
            setup_test_vault(&mut fixture, 100, 200).await.unwrap();

        let config = fixture
            .vault_program_client()
            .get_config(&config_pubkey)
            .await
            .unwrap();
        let vault = fixture
            .vault_program_client()
            .get_vault(&vault_pubkey)
            .await
            .unwrap();

        fixture
            .warp_slot_incremental(config.epoch_length * 2)
            .await
            .unwrap();

        let deposit_fee_bps = vault.deposit_fee_bps + config.fee_bump_bps;
        let withdrawal_fee_bps = vault.withdrawal_fee_bps + config.fee_bump_bps;

        fixture
            .vault_program_client()
            .set_fees(
                &config_pubkey,
                &vault_pubkey,
                &vault_admin,
                deposit_fee_bps,
                withdrawal_fee_bps,
            )
            .await
            .unwrap();

        let updated_vault = fixture
            .vault_program_client()
            .get_vault(&vault_pubkey)
            .await
            .unwrap();
        assert_eq!(updated_vault.deposit_fee_bps, deposit_fee_bps);
        assert_eq!(updated_vault.withdrawal_fee_bps, withdrawal_fee_bps);
    }

    #[tokio::test]
    async fn test_set_fees_to_zero() {
        let mut fixture = TestBuilder::new().await;
        let (config_pubkey, vault_pubkey, vault_admin) =
            setup_test_vault(&mut fixture, 100, 200).await.unwrap();

        let config = fixture
            .vault_program_client()
            .get_config(&config_pubkey)
            .await
            .unwrap();
        fixture
            .warp_slot_incremental(config.epoch_length * 2)
            .await
            .unwrap();

        let deposit_fee_bps = 0;
        let withdrawal_fee_bps = 0;

        fixture
            .vault_program_client()
            .set_fees(
                &config_pubkey,
                &vault_pubkey,
                &vault_admin,
                deposit_fee_bps,
                withdrawal_fee_bps,
            )
            .await
            .unwrap();

        let updated_vault = fixture
            .vault_program_client()
            .get_vault(&vault_pubkey)
            .await
            .unwrap();
        assert_eq!(updated_vault.deposit_fee_bps, deposit_fee_bps);
        assert_eq!(updated_vault.withdrawal_fee_bps, withdrawal_fee_bps);
    }
}
