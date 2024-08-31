#[cfg(test)]
mod tests {
    use jito_vault_core::{
        config::Config, vault_operator_delegation::VaultOperatorDelegation,
        vault_update_state_tracker::VaultUpdateStateTracker,
    };
    use solana_sdk::{signature::Keypair, signer::Signer};

    use crate::fixtures::fixture::{ConfiguredVault, TestBuilder};

    #[tokio::test]
    async fn test_update_vault_balance_ok() {
        const MINT_AMOUNT: u64 = 100_000;

        let deposit_fee_bps = 0;
        let withdraw_fee_bps = 0;
        let reward_fee_bps = 1000; // 10%
        let num_operators = 1;
        let slasher_amounts = vec![];

        let mut fixture = TestBuilder::new().await;
        let ConfiguredVault {
            mut vault_program_client,
            restaking_program_client: _,
            vault_config_admin: _,
            vault_root,
            restaking_config_admin: _,
            ncn_root: _,
            operator_roots,
            slashers_amounts: _,
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

        // Reward vault instead of staking
        vault_program_client
            .create_and_fund_reward_vault(&vault_root.vault_pubkey, &depositor, MINT_AMOUNT)
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

        assert_eq!(vault.tokens_deposited(), MINT_AMOUNT);
        assert_eq!(reward_fee_account.amount, MINT_AMOUNT / 10);
        assert_eq!(vault.vrt_supply(), MINT_AMOUNT / 10);
    }

    #[tokio::test]
    async fn test_update_vault_balance_no_state_change_on_error() {
        const MINT_AMOUNT: u64 = 100_000;

        let deposit_fee_bps = 0;
        let withdraw_fee_bps = 0;
        let reward_fee_bps = 1000; // 10%
        let num_operators = 1;
        let slasher_amounts = vec![];

        let mut fixture = TestBuilder::new().await;
        let ConfiguredVault {
            mut vault_program_client,
            restaking_program_client: _,
            vault_config_admin: _,
            vault_root,
            restaking_config_admin: _,
            ncn_root: _,
            operator_roots,
            slashers_amounts: _,
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

        let alice = Keypair::new();
        vault_program_client
            .configure_depositor(&vault_root, &alice.pubkey(), MINT_AMOUNT)
            .await
            .unwrap();

        // Reward vault instead of staking
        vault_program_client
            .create_and_fund_reward_vault(&vault_root.vault_pubkey, &alice, MINT_AMOUNT)
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

        let wrong_vault_root = vault_program_client
            .do_initialize_vault(deposit_fee_bps, withdraw_fee_bps, reward_fee_bps, 9)
            .await
            .unwrap();

        let response = vault_program_client
            .update_vault_balance(&wrong_vault_root.vault_pubkey)
            .await;

        assert!(response.is_err());

        let vault = vault_program_client
            .get_vault(&vault_root.vault_pubkey)
            .await
            .unwrap();

        let reward_fee_account = vault_program_client
            .get_reward_fee_token_account(&vault_root.vault_pubkey)
            .await
            .unwrap();

        assert_eq!(vault.tokens_deposited(), 0);
        assert_eq!(reward_fee_account.amount, 0);
        assert_eq!(vault.vrt_supply(), 0);
    }
}
