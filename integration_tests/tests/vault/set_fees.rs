#[cfg(test)]
mod tests {
    use jito_vault_core::{config::Config, MAX_FEE_BPS};
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
        reward_fee_bps: u16,
    ) -> Result<(Pubkey, Pubkey, Keypair), TestError> {
        let mut vault_program_client = fixture.vault_program_client();

        let result = vault_program_client
            .setup_config_and_vault(deposit_fee_bps, withdrawal_fee_bps, reward_fee_bps)
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
                assert_eq!(vault.deposit_fee_bps(), deposit_fee_bps);
                assert_eq!(vault.withdrawal_fee_bps(), withdrawal_fee_bps);
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
        let reward_fee_bps = u16::MAX;

        let result = setup_test_vault(
            &mut fixture,
            deposit_fee_bps,
            withdrawal_fee_bps,
            reward_fee_bps,
        )
        .await;

        assert_vault_error(result, VaultError::VaultFeeCapExceeded);
    }

    #[tokio::test]
    async fn test_initial_fee_setup() {
        let mut fixture = TestBuilder::new().await;
        let deposit_fee_bps = 99;
        let withdrawal_fee_bps = 100;
        let reward_fee_bps = 101;

        let (_, vault_pubkey, _) = setup_test_vault(
            &mut fixture,
            deposit_fee_bps,
            withdrawal_fee_bps,
            reward_fee_bps,
        )
        .await
        .unwrap();

        let vault = fixture
            .vault_program_client()
            .get_vault(&vault_pubkey)
            .await
            .unwrap();

        assert_eq!(vault.deposit_fee_bps(), deposit_fee_bps);
        assert_eq!(vault.withdrawal_fee_bps(), withdrawal_fee_bps);
    }

    #[tokio::test]
    async fn test_change_fees_after_two_epochs() {
        let mut fixture = TestBuilder::new().await;

        let deposit_fee_bps = 99;
        let withdrawal_fee_bps = 100;
        let reward_fee_bps = 101;

        let (config_pubkey, vault_pubkey, vault_admin) = setup_test_vault(
            &mut fixture,
            deposit_fee_bps,
            withdrawal_fee_bps,
            reward_fee_bps,
        )
        .await
        .unwrap();

        let config = fixture
            .vault_program_client()
            .get_config(&config_pubkey)
            .await
            .unwrap();

        fixture
            .warp_slot_incremental(config.epoch_length() * 2 + 1)
            .await
            .unwrap();

        let new_deposit_fee_bps = 100;
        let new_withdrawal_fee_bps = 101;
        let new_reward_fee_bps = 102;

        fixture
            .vault_program_client()
            .set_fees(
                &config_pubkey,
                &vault_pubkey,
                &vault_admin,
                Some(new_deposit_fee_bps),
                Some(new_withdrawal_fee_bps),
                Some(new_reward_fee_bps),
            )
            .await
            .unwrap();

        let vault = fixture
            .vault_program_client()
            .get_vault(&vault_pubkey)
            .await
            .unwrap();

        assert_eq!(vault.deposit_fee_bps(), new_deposit_fee_bps);
        assert_eq!(vault.withdrawal_fee_bps(), new_withdrawal_fee_bps);
        assert_eq!(vault.reward_fee_bps(), new_reward_fee_bps);
    }

    #[tokio::test]
    async fn test_cannot_change_fees_before_epoch_passes() {
        let mut fixture = TestBuilder::new().await;

        let deposit_fee_bps = 99;
        let withdrawal_fee_bps = 100;
        let reward_fee_bps = 101;

        let (config_pubkey, vault_pubkey, vault_admin) = setup_test_vault(
            &mut fixture,
            deposit_fee_bps,
            withdrawal_fee_bps,
            reward_fee_bps,
        )
        .await
        .unwrap();

        let new_deposit_fee_bps = 100;
        let new_withdrawal_fee_bps = 101;
        let new_reward_fee_bps = 102;

        let result = fixture
            .vault_program_client()
            .set_fees(
                &config_pubkey,
                &vault_pubkey,
                &vault_admin,
                Some(new_deposit_fee_bps),
                Some(new_withdrawal_fee_bps),
                Some(new_reward_fee_bps),
            )
            .await;

        assert_vault_error(result, VaultError::VaultFeeChangeTooSoon);
    }

    #[tokio::test]
    async fn test_can_change_fees_after_another_epoch() {
        let mut fixture = TestBuilder::new().await;

        let deposit_fee_bps = 99;
        let withdrawal_fee_bps = 100;
        let reward_fee_bps = 101;

        let (config_pubkey, vault_pubkey, vault_admin) = setup_test_vault(
            &mut fixture,
            deposit_fee_bps,
            withdrawal_fee_bps,
            reward_fee_bps,
        )
        .await
        .unwrap();

        let config = fixture
            .vault_program_client()
            .get_config(&config_pubkey)
            .await
            .unwrap();
        fixture
            .warp_slot_incremental(config.epoch_length() * 2)
            .await
            .unwrap();

        let new_deposit_fee_bps = 100;
        let new_withdrawal_fee_bps = 101;
        let new_reward_fee_bps = 102;

        // First fee change
        fixture
            .vault_program_client()
            .set_fees(
                &config_pubkey,
                &vault_pubkey,
                &vault_admin,
                Some(new_deposit_fee_bps),
                Some(new_withdrawal_fee_bps),
                Some(new_reward_fee_bps),
            )
            .await
            .unwrap();

        let vault = fixture
            .vault_program_client()
            .get_vault(&vault_pubkey)
            .await
            .unwrap();

        assert_eq!(vault.deposit_fee_bps(), new_deposit_fee_bps);
        assert_eq!(vault.withdrawal_fee_bps(), new_withdrawal_fee_bps);
        assert_eq!(vault.reward_fee_bps(), new_reward_fee_bps);

        // Warp again
        fixture
            .warp_slot_incremental(config.epoch_length() * 2)
            .await
            .unwrap();

        // Second fee change
        let new_deposit_fee_bps = 101;
        let new_withdrawal_fee_bps = 102;
        let new_reward_fee_bps = 103;

        fixture
            .vault_program_client()
            .set_fees(
                &config_pubkey,
                &vault_pubkey,
                &vault_admin,
                Some(new_deposit_fee_bps),
                Some(new_withdrawal_fee_bps),
                Some(new_reward_fee_bps),
            )
            .await
            .unwrap();

        let vault = fixture
            .vault_program_client()
            .get_vault(&vault_pubkey)
            .await
            .unwrap();
        assert_eq!(vault.deposit_fee_bps(), new_deposit_fee_bps);
        assert_eq!(vault.withdrawal_fee_bps(), new_withdrawal_fee_bps);
        assert_eq!(vault.reward_fee_bps(), new_reward_fee_bps);
    }

    #[tokio::test]
    async fn test_cannot_change_fees_with_invalid_admin() {
        let mut fixture = TestBuilder::new().await;

        let deposit_fee_bps = 99;
        let withdrawal_fee_bps = 100;
        let reward_fee_bps = 101;

        let (config_pubkey, vault_pubkey, _) = setup_test_vault(
            &mut fixture,
            deposit_fee_bps,
            withdrawal_fee_bps,
            reward_fee_bps,
        )
        .await
        .unwrap();

        let bad_admin = Keypair::new();
        fixture
            .vault_program_client()
            .airdrop(&bad_admin.pubkey(), 10.0)
            .await
            .unwrap();

        let config = fixture
            .vault_program_client()
            .get_config(&config_pubkey)
            .await
            .unwrap();
        fixture
            .warp_slot_incremental(config.epoch_length() * 2)
            .await
            .unwrap();

        let new_deposit_fee_bps = 150;

        let result = fixture
            .vault_program_client()
            .set_fees(
                &config_pubkey,
                &vault_pubkey,
                &bad_admin,
                Some(new_deposit_fee_bps),
                None,
                None,
            )
            .await;

        assert_vault_error(result, VaultError::VaultFeeAdminInvalid);
    }

    #[tokio::test]
    async fn test_set_fees_regularly() {
        let mut fixture = TestBuilder::new().await;

        let deposit_fee_bps = 99;
        let withdrawal_fee_bps = 100;
        let reward_fee_bps = 101;

        let (config_pubkey, vault_pubkey, vault_admin) = setup_test_vault(
            &mut fixture,
            deposit_fee_bps,
            withdrawal_fee_bps,
            reward_fee_bps,
        )
        .await
        .unwrap();

        let config = fixture
            .vault_program_client()
            .get_config(&config_pubkey)
            .await
            .unwrap();
        fixture
            .warp_slot_incremental(config.epoch_length() * 2)
            .await
            .unwrap();

        let new_deposit_fee_bps = 100;
        let new_withdrawal_fee_bps = 101;
        let new_reward_fee_bps = 102;

        fixture
            .vault_program_client()
            .set_fees(
                &config_pubkey,
                &vault_pubkey,
                &vault_admin,
                Some(new_deposit_fee_bps),
                Some(new_withdrawal_fee_bps),
                Some(new_reward_fee_bps),
            )
            .await
            .unwrap();

        let vault = fixture
            .vault_program_client()
            .get_vault(&vault_pubkey)
            .await
            .unwrap();
        assert_eq!(vault.deposit_fee_bps(), new_deposit_fee_bps);
        assert_eq!(vault.withdrawal_fee_bps(), new_withdrawal_fee_bps);
        assert_eq!(vault.reward_fee_bps(), new_reward_fee_bps);
    }

    #[tokio::test]
    async fn test_set_secondary_admin() {
        let mut fixture = TestBuilder::new().await;

        let deposit_fee_bps = 99;
        let withdrawal_fee_bps = 100;
        let reward_fee_bps = 101;

        let (config_pubkey, vault_pubkey, vault_admin) = setup_test_vault(
            &mut fixture,
            deposit_fee_bps,
            withdrawal_fee_bps,
            reward_fee_bps,
        )
        .await
        .unwrap();

        let new_admin = Keypair::new();
        fixture
            .vault_program_client()
            .airdrop(&new_admin.pubkey(), 10.0)
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

        let deposit_fee_bps = 99;
        let withdrawal_fee_bps = 100;
        let reward_fee_bps = 101;

        let (config_pubkey, vault_pubkey, vault_admin) = setup_test_vault(
            &mut fixture,
            deposit_fee_bps,
            withdrawal_fee_bps,
            reward_fee_bps,
        )
        .await
        .unwrap();

        let new_admin = Keypair::new();
        fixture
            .vault_program_client()
            .airdrop(&new_admin.pubkey(), 10.0)
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
            .warp_slot_incremental(config.epoch_length() * 2)
            .await
            .unwrap();

        let new_deposit_fee_bps = 300;

        let result = fixture
            .vault_program_client()
            .set_fees(
                &config_pubkey,
                &vault_pubkey,
                &vault_admin,
                Some(new_deposit_fee_bps),
                None,
                None,
            )
            .await;

        assert_vault_error(result, VaultError::VaultFeeAdminInvalid);
    }

    #[tokio::test]
    async fn test_set_fees_with_new_admin_succeeds() {
        let mut fixture = TestBuilder::new().await;

        let deposit_fee_bps = 99;
        let withdrawal_fee_bps = 100;
        let reward_fee_bps = 101;

        let (config_pubkey, vault_pubkey, vault_admin) = setup_test_vault(
            &mut fixture,
            deposit_fee_bps,
            withdrawal_fee_bps,
            reward_fee_bps,
        )
        .await
        .unwrap();

        let new_admin = Keypair::new();
        fixture
            .vault_program_client()
            .airdrop(&new_admin.pubkey(), 10.0)
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
            .warp_slot_incremental(config.epoch_length() * 2)
            .await
            .unwrap();

        let new_deposit_fee_bps = 100;
        let new_withdrawal_fee_bps = 101;
        let new_reward_fee_bps = 102;

        fixture
            .vault_program_client()
            .set_fees(
                &config_pubkey,
                &vault_pubkey,
                &new_admin,
                Some(new_deposit_fee_bps),
                Some(new_withdrawal_fee_bps),
                Some(new_reward_fee_bps),
            )
            .await
            .unwrap();

        let vault = fixture
            .vault_program_client()
            .get_vault(&vault_pubkey)
            .await
            .unwrap();
        assert_eq!(vault.deposit_fee_bps(), new_deposit_fee_bps);
        assert_eq!(vault.withdrawal_fee_bps(), new_withdrawal_fee_bps);
        assert_eq!(vault.reward_fee_bps(), new_reward_fee_bps);
    }

    #[tokio::test]
    async fn test_set_fees_larger_than_cap() {
        let mut fixture = TestBuilder::new().await;

        let deposit_fee_bps = 100;
        let withdrawal_fee_bps = 200;
        let reward_fee_bps = 300;

        let (config_pubkey, vault_pubkey, vault_admin) = setup_test_vault(
            &mut fixture,
            deposit_fee_bps,
            withdrawal_fee_bps,
            reward_fee_bps,
        )
        .await
        .unwrap();

        let config = fixture
            .vault_program_client()
            .get_config(&config_pubkey)
            .await
            .unwrap();
        fixture
            .warp_slot_incremental(config.epoch_length() * 2)
            .await
            .unwrap();

        let new_deposit_fee_bps = config.deposit_withdrawal_fee_cap_bps() + 1;
        let new_withdrawal_fee_bps = config.deposit_withdrawal_fee_cap_bps() + 1;
        let new_reward_fee_bps = MAX_FEE_BPS + 1;

        let result = fixture
            .vault_program_client()
            .set_fees(
                &config_pubkey,
                &vault_pubkey,
                &vault_admin,
                Some(new_deposit_fee_bps),
                Some(new_withdrawal_fee_bps),
                Some(new_reward_fee_bps),
            )
            .await;

        assert_vault_error(result, VaultError::VaultFeeCapExceeded);
    }

    #[tokio::test]
    async fn test_set_fees_with_ok_change() {
        let mut fixture = TestBuilder::new().await;

        let deposit_fee_bps = 100;
        let withdrawal_fee_bps = 200;
        let reward_fee_bps = 300;

        let (config_pubkey, vault_pubkey, vault_admin) = setup_test_vault(
            &mut fixture,
            deposit_fee_bps,
            withdrawal_fee_bps,
            reward_fee_bps,
        )
        .await
        .unwrap();

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

        let new_deposit_fee_bps = vault.deposit_fee_bps()
            + (config.fee_rate_of_change_bps() as u64 * vault.deposit_fee_bps() as u64 / 10_000)
                as u16;
        let new_withdrawal_fee_bps = vault.withdrawal_fee_bps()
            + (config.fee_rate_of_change_bps() as u64 * vault.withdrawal_fee_bps() as u64 / 10_000)
                as u16;

        fixture
            .warp_slot_incremental(config.epoch_length() * 2)
            .await
            .unwrap();

        fixture
            .vault_program_client()
            .set_fees(
                &config_pubkey,
                &vault_pubkey,
                &vault_admin,
                Some(new_deposit_fee_bps),
                Some(new_withdrawal_fee_bps),
                None,
            )
            .await
            .unwrap();

        let vault = fixture
            .vault_program_client()
            .get_vault(&vault_pubkey)
            .await
            .unwrap();

        assert_eq!(vault.deposit_fee_bps(), new_deposit_fee_bps);
        assert_eq!(vault.withdrawal_fee_bps(), new_withdrawal_fee_bps);
    }

    #[tokio::test]
    async fn test_set_fees_with_too_large_change() {
        let mut fixture = TestBuilder::new().await;

        let deposit_fee_bps = 100;
        let withdrawal_fee_bps = 200;
        let reward_fee_bps = 300;

        let (config_pubkey, vault_pubkey, vault_admin) = setup_test_vault(
            &mut fixture,
            deposit_fee_bps,
            withdrawal_fee_bps,
            reward_fee_bps,
        )
        .await
        .unwrap();

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

        let new_deposit_fee_bps = vault.deposit_fee_bps()
            + (config.fee_rate_of_change_bps() as u64 * vault.deposit_fee_bps() as u64 / 10_000)
                as u16
            + 1;
        let new_withdrawal_fee_bps = vault.withdrawal_fee_bps()
            + (config.fee_rate_of_change_bps() as u64 * vault.withdrawal_fee_bps() as u64 / 10_000)
                as u16
            + 1;

        fixture
            .warp_slot_incremental(config.epoch_length() * 2)
            .await
            .unwrap();

        let result = fixture
            .vault_program_client()
            .set_fees(
                &config_pubkey,
                &vault_pubkey,
                &vault_admin,
                Some(new_deposit_fee_bps),
                Some(new_withdrawal_fee_bps),
                None,
            )
            .await;

        assert_vault_error(result, VaultError::VaultFeeBumpTooLarge);
    }

    #[tokio::test]
    async fn test_set_fees_to_max_bump() {
        let mut fixture = TestBuilder::new().await;

        let deposit_fee_bps = 100;
        let withdrawal_fee_bps = 200;
        let reward_fee_bps = 300;

        let (config_pubkey, vault_pubkey, vault_admin) = setup_test_vault(
            &mut fixture,
            deposit_fee_bps,
            withdrawal_fee_bps,
            reward_fee_bps,
        )
        .await
        .unwrap();

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
            .warp_slot_incremental(config.epoch_length() * 2)
            .await
            .unwrap();

        let new_deposit_fee_bps = vault.deposit_fee_bps() + config.fee_bump_bps();
        let new_withdrawal_fee_bps = vault.withdrawal_fee_bps() + config.fee_bump_bps();

        fixture
            .vault_program_client()
            .set_fees(
                &config_pubkey,
                &vault_pubkey,
                &vault_admin,
                Some(new_deposit_fee_bps),
                Some(new_withdrawal_fee_bps),
                None,
            )
            .await
            .unwrap();

        let updated_vault = fixture
            .vault_program_client()
            .get_vault(&vault_pubkey)
            .await
            .unwrap();
        assert_eq!(updated_vault.deposit_fee_bps(), new_deposit_fee_bps);
        assert_eq!(updated_vault.withdrawal_fee_bps(), new_withdrawal_fee_bps);
    }

    #[tokio::test]
    async fn test_set_fees_to_zero() {
        let mut fixture = TestBuilder::new().await;

        let deposit_fee_bps = 100;
        let withdrawal_fee_bps = 200;
        let reward_fee_bps = 300;

        let (config_pubkey, vault_pubkey, vault_admin) = setup_test_vault(
            &mut fixture,
            deposit_fee_bps,
            withdrawal_fee_bps,
            reward_fee_bps,
        )
        .await
        .unwrap();

        let config = fixture
            .vault_program_client()
            .get_config(&config_pubkey)
            .await
            .unwrap();
        fixture
            .warp_slot_incremental(config.epoch_length() * 2)
            .await
            .unwrap();

        let new_deposit_fee_bps = 0;
        let new_withdrawal_fee_bps = 0;
        let reward_fee_bps = 0;

        fixture
            .vault_program_client()
            .set_fees(
                &config_pubkey,
                &vault_pubkey,
                &vault_admin,
                Some(new_deposit_fee_bps),
                Some(new_withdrawal_fee_bps),
                Some(reward_fee_bps),
            )
            .await
            .unwrap();

        let updated_vault = fixture
            .vault_program_client()
            .get_vault(&vault_pubkey)
            .await
            .unwrap();
        assert_eq!(updated_vault.deposit_fee_bps(), new_deposit_fee_bps);
        assert_eq!(updated_vault.withdrawal_fee_bps(), new_withdrawal_fee_bps);
        assert_eq!(updated_vault.reward_fee_bps(), reward_fee_bps);
    }

    #[tokio::test]
    async fn test_set_each() {
        let mut fixture = TestBuilder::new().await;

        let deposit_fee_bps = 100;
        let withdrawal_fee_bps = 200;
        let reward_fee_bps = 300;

        let (config_pubkey, vault_pubkey, vault_admin) = setup_test_vault(
            &mut fixture,
            deposit_fee_bps,
            withdrawal_fee_bps,
            reward_fee_bps,
        )
        .await
        .unwrap();

        let config = fixture
            .vault_program_client()
            .get_config(&config_pubkey)
            .await
            .unwrap();

        let new_deposit_fee_bps = deposit_fee_bps + 1;

        fixture
            .warp_slot_incremental(config.epoch_length() * 2)
            .await
            .unwrap();

        fixture
            .vault_program_client()
            .set_fees(
                &config_pubkey,
                &vault_pubkey,
                &vault_admin,
                Some(new_deposit_fee_bps),
                None,
                None,
            )
            .await
            .unwrap();

        let updated_vault = fixture
            .vault_program_client()
            .get_vault(&vault_pubkey)
            .await
            .unwrap();
        assert_eq!(updated_vault.deposit_fee_bps(), new_deposit_fee_bps);
        assert_eq!(updated_vault.withdrawal_fee_bps(), withdrawal_fee_bps);
        assert_eq!(updated_vault.reward_fee_bps(), reward_fee_bps);

        let new_withdrawal_fee_bps = withdrawal_fee_bps + 1;

        fixture
            .warp_slot_incremental(config.epoch_length() * 2)
            .await
            .unwrap();

        fixture
            .vault_program_client()
            .set_fees(
                &config_pubkey,
                &vault_pubkey,
                &vault_admin,
                None,
                Some(new_withdrawal_fee_bps),
                None,
            )
            .await
            .unwrap();

        let updated_vault = fixture
            .vault_program_client()
            .get_vault(&vault_pubkey)
            .await
            .unwrap();
        assert_eq!(updated_vault.deposit_fee_bps(), new_deposit_fee_bps);
        assert_eq!(updated_vault.withdrawal_fee_bps(), new_withdrawal_fee_bps);
        assert_eq!(updated_vault.reward_fee_bps(), reward_fee_bps);

        let new_reward_fee_bps = reward_fee_bps + 1;

        fixture
            .warp_slot_incremental(config.epoch_length() * 2)
            .await
            .unwrap();

        fixture
            .vault_program_client()
            .set_fees(
                &config_pubkey,
                &vault_pubkey,
                &vault_admin,
                None,
                None,
                Some(new_reward_fee_bps),
            )
            .await
            .unwrap();

        let updated_vault = fixture
            .vault_program_client()
            .get_vault(&vault_pubkey)
            .await
            .unwrap();
        assert_eq!(updated_vault.deposit_fee_bps(), new_deposit_fee_bps);
        assert_eq!(updated_vault.withdrawal_fee_bps(), new_withdrawal_fee_bps);
        assert_eq!(updated_vault.reward_fee_bps(), new_reward_fee_bps);
    }

    #[tokio::test]
    async fn test_cap_for_each() {
        let mut fixture = TestBuilder::new().await;

        let deposit_fee_bps = 100;
        let withdrawal_fee_bps = 200;
        let reward_fee_bps = 300;

        let (config_pubkey, vault_pubkey, vault_admin) = setup_test_vault(
            &mut fixture,
            deposit_fee_bps,
            withdrawal_fee_bps,
            reward_fee_bps,
        )
        .await
        .unwrap();

        let config = fixture
            .vault_program_client()
            .get_config(&config_pubkey)
            .await
            .unwrap();
        fixture
            .warp_slot_incremental(config.epoch_length() * 2)
            .await
            .unwrap();

        let new_deposit_fee_bps = MAX_FEE_BPS + 1;
        let new_withdrawal_fee_bps = MAX_FEE_BPS + 1;
        let new_reward_fee_bps = MAX_FEE_BPS + 1;

        let result = fixture
            .vault_program_client()
            .set_fees(
                &config_pubkey,
                &vault_pubkey,
                &vault_admin,
                Some(new_deposit_fee_bps),
                None,
                None,
            )
            .await;

        assert_vault_error(result, VaultError::VaultFeeCapExceeded);

        let result = fixture
            .vault_program_client()
            .set_fees(
                &config_pubkey,
                &vault_pubkey,
                &vault_admin,
                None,
                Some(new_withdrawal_fee_bps),
                None,
            )
            .await;

        assert_vault_error(result, VaultError::VaultFeeCapExceeded);

        let result = fixture
            .vault_program_client()
            .set_fees(
                &config_pubkey,
                &vault_pubkey,
                &vault_admin,
                None,
                None,
                Some(new_reward_fee_bps),
            )
            .await;

        assert_vault_error(result, VaultError::VaultFeeCapExceeded);
    }

    #[tokio::test]
    async fn test_set_program_fee() {
        let fixture = TestBuilder::new().await;
        let mut vault_program_client = fixture.vault_program_client();

        // Initialize config and vault
        let (config_admin, _) = vault_program_client
            .setup_config_and_vault(0, 0, 0)
            .await
            .unwrap();

        // Set a new program fee
        let new_fee_bps = 10;
        vault_program_client
            .set_program_fee(&config_admin, new_fee_bps)
            .await
            .unwrap();

        // Check if the program fee was updated
        let updated_config = vault_program_client
            .get_config(&Config::find_program_address(&jito_vault_program::id()).0)
            .await
            .unwrap();
        assert_eq!(updated_config.program_fee_bps(), new_fee_bps);

        // Try to set fee with non-admin account
        let non_admin = Keypair::new();
        vault_program_client
            .airdrop(&non_admin.pubkey(), 1.0)
            .await
            .unwrap();
        let result = vault_program_client.set_program_fee(&non_admin, 200).await;
        assert_vault_error(result, VaultError::ConfigAdminInvalid);

        // Try to set fee above MAX_FEE_BPS
        let result = vault_program_client
            .set_program_fee(&config_admin, MAX_FEE_BPS + 1)
            .await;
        assert!(result.is_err());

        // Set fee to the same value (should succeed)
        vault_program_client
            .set_program_fee(&config_admin, new_fee_bps)
            .await
            .unwrap();
    }
}
