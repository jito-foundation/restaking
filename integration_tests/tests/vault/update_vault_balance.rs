#[cfg(test)]
mod tests {
    use jito_vault_core::{
        config::Config, vault_operator_delegation::VaultOperatorDelegation,
        vault_update_state_tracker::VaultUpdateStateTracker,
    };
    use jito_vault_sdk::error::VaultError;
    use solana_sdk::{signature::Keypair, signer::Signer};

    use crate::fixtures::{
        fixture::{ConfiguredVault, TestBuilder},
        vault_client::{assert_vault_error, VaultRoot},
    };

    async fn setup_with_reward(
        reward_amount: u64,
        deposit_fee_bps: u16,
        withdrawal_fee_bps: u16,
        reward_fee_bps: u16,
    ) -> (TestBuilder, VaultRoot) {
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
                withdrawal_fee_bps,
                reward_fee_bps,
                num_operators,
                &slasher_amounts,
            )
            .await
            .unwrap();

        // Initial deposit + mint
        let depositor = Keypair::new();
        vault_program_client
            .configure_depositor(&vault_root, &depositor.pubkey(), reward_amount)
            .await
            .unwrap();

        // Reward vault instead of staking
        vault_program_client
            .create_and_fund_reward_vault(&vault_root.vault_pubkey, &depositor, reward_amount)
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
        let vault_update_state_tracker = VaultUpdateStateTracker::find_program_address(
            &jito_vault_program::id(),
            &vault_root.vault_pubkey,
            slot / config.epoch_length(),
        )
        .0;
        vault_program_client
            .initialize_vault_update_state_tracker(
                &vault_root.vault_pubkey,
                &vault_update_state_tracker,
            )
            .await
            .unwrap();

        for operator in operator_roots {
            vault_program_client
                .crank_vault_update_state_tracker(
                    &vault_root.vault_pubkey,
                    &operator.operator_pubkey,
                    &VaultOperatorDelegation::find_program_address(
                        &jito_vault_program::id(),
                        &vault_root.vault_pubkey,
                        &operator.operator_pubkey,
                    )
                    .0,
                    &vault_update_state_tracker,
                )
                .await
                .unwrap();
        }

        vault_program_client
            .close_vault_update_state_tracker(
                &vault_root.vault_pubkey,
                &vault_update_state_tracker,
                slot / config.epoch_length(),
            )
            .await
            .unwrap();

        (fixture, vault_root)
    }

    #[tokio::test]
    async fn test_update_vault_balance_ok() {
        const MINT_AMOUNT: u64 = 1000;
        // Match's unit test in vault.rs: test_calculate_reward_fee
        const EXPECTED_FEE: u64 = 52;

        let (fixture, vault_root) = setup_with_reward(
            MINT_AMOUNT,
            0,
            0,
            1000, //10%
        )
        .await;
        let mut vault_program_client = fixture.vault_program_client();

        let depositor = Keypair::new();
        vault_program_client
            .configure_depositor(&vault_root, &depositor.pubkey(), MINT_AMOUNT)
            .await
            .unwrap();

        vault_program_client
            .do_mint_to(&vault_root, &depositor, MINT_AMOUNT, MINT_AMOUNT)
            .await
            .unwrap();

        vault_program_client
            .update_vault_balance(&vault_root.vault_pubkey)
            .await
            .unwrap();

        let vault = vault_program_client
            .get_vault(&vault_root.vault_pubkey)
            .await
            .unwrap();

        let reward_fee_account = vault_program_client
            .get_reward_fee_token_account(&vault_root.vault_pubkey)
            .await
            .unwrap();

        assert_eq!(vault.tokens_deposited(), MINT_AMOUNT * 2);
        assert_eq!(reward_fee_account.amount, EXPECTED_FEE);
        assert_eq!(vault.vrt_supply(), MINT_AMOUNT + EXPECTED_FEE);
    }

    #[tokio::test]
    async fn test_update_vault_balance_no_initial_supply() {
        let (fixture, vault_root) = setup_with_reward(
            1000, 0, 0, 1000, //10%
        )
        .await;
        let mut vault_program_client = fixture.vault_program_client();

        let test_error = vault_program_client
            .update_vault_balance(&vault_root.vault_pubkey)
            .await;

        assert_vault_error(test_error, VaultError::VaultRewardFeeIsZero);
    }

    #[tokio::test]
    async fn test_update_vault_balance_vault_is_paused_fails() {
        let (fixture, vault_root) = setup_with_reward(
            1000, 0, 0, 1000, //10%
        )
        .await;
        let mut vault_program_client = fixture.vault_program_client();

        vault_program_client
            .set_is_paused(&vault_root.vault_pubkey, &vault_root.vault_admin, true)
            .await
            .unwrap();

        let test_error = vault_program_client
            .update_vault_balance(&vault_root.vault_pubkey)
            .await;

        assert_vault_error(test_error, VaultError::VaultIsPaused);
    }
}
