#[cfg(test)]
mod tests {
    use jito_vault_core::{config::Config, vault_operator_delegation::VaultOperatorDelegation};
    use jito_vault_sdk::error::VaultError;
    use solana_sdk::signature::{Keypair, Signer};

    use crate::fixtures::{
        fixture::{ConfiguredVault, TestBuilder},
        vault_client::assert_vault_error,
    };

    #[tokio::test]
    async fn test_cooldown_delegation_invalid_admin_fails() {
        let mut fixture = TestBuilder::new().await;

        let deposit_fee_bps = 0;
        let withdraw_fee_bps = 0;
        let reward_fee_bps = 0;
        let num_operators = 1;
        let slasher_amounts = vec![];

        let ConfiguredVault {
            mut vault_program_client,
            operator_roots,
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

        let result = vault_program_client
            .cooldown_delegation(
                &Config::find_program_address(&jito_vault_program::id()).0,
                &vault_root.vault_pubkey,
                &operator_roots[0].operator_pubkey,
                &VaultOperatorDelegation::find_program_address(
                    &jito_vault_program::id(),
                    &vault_root.vault_pubkey,
                    &operator_roots[0].operator_pubkey,
                )
                .0,
                &Keypair::new(),
                1,
            )
            .await;
        assert_vault_error(result, VaultError::VaultDelegationAdminInvalid);
    }

    #[tokio::test]
    async fn test_cooldown_delegation_too_much_fails() {
        let mut fixture = TestBuilder::new().await;

        let deposit_fee_bps = 0;
        let withdraw_fee_bps = 0;
        let reward_fee_bps = 0;
        let num_operators = 1;
        let slasher_amounts = vec![];

        let ConfiguredVault {
            mut vault_program_client,
            operator_roots,
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

        let result = vault_program_client
            .do_cooldown_delegation(&vault_root, &operator_roots[0].operator_pubkey, 50_001)
            .await;
        assert_vault_error(result, VaultError::VaultSecurityUnderflow);
    }

    #[tokio::test]
    async fn test_cooldown_delegation_vault_needs_updating_fails() {
        let mut fixture = TestBuilder::new().await;

        let deposit_fee_bps = 0;
        let withdraw_fee_bps = 0;
        let reward_fee_bps = 0;
        let num_operators = 1;
        let slasher_amounts = vec![];

        let ConfiguredVault {
            mut vault_program_client,
            operator_roots,
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

        let config = vault_program_client
            .get_config(&Config::find_program_address(&jito_vault_program::id()).0)
            .await
            .unwrap();

        fixture
            .warp_slot_incremental(config.epoch_length)
            .await
            .unwrap();

        let result = vault_program_client
            .do_cooldown_delegation(&vault_root, &operator_roots[0].operator_pubkey, 1)
            .await;
        assert_vault_error(result, VaultError::VaultUpdateNeeded);
    }

    #[tokio::test]
    async fn test_cooldown_delegation_vault_withdrawal_ok() {
        let mut fixture = TestBuilder::new().await;

        let deposit_fee_bps = 0;
        let withdraw_fee_bps = 0;
        let reward_fee_bps = 0;
        let num_operators = 1;
        let slasher_amounts = vec![];

        let ConfiguredVault {
            mut vault_program_client,
            operator_roots,
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
            .do_cooldown_delegation(&vault_root, &operator_roots[0].operator_pubkey, 50_000)
            .await
            .unwrap();

        let vault = vault_program_client
            .get_vault(&vault_root.vault_pubkey)
            .await
            .unwrap();
        assert_eq!(vault.delegation_state.total_security().unwrap(), 100_000);
        assert_eq!(vault.delegation_state.staked_amount, 50_000);
        assert_eq!(vault.delegation_state.enqueued_for_cooldown_amount, 50_000);

        let vault_operator_delegation = vault_program_client
            .get_vault_operator_delegation(
                &vault_root.vault_pubkey,
                &operator_roots[0].operator_pubkey,
            )
            .await
            .unwrap();
        assert_eq!(
            vault_operator_delegation
                .delegation_state
                .total_security()
                .unwrap(),
            100_000
        );
        assert_eq!(
            vault_operator_delegation.delegation_state.staked_amount,
            50_000
        );
        assert_eq!(
            vault_operator_delegation
                .delegation_state
                .enqueued_for_cooldown_amount,
            50_000
        );
    }

    #[tokio::test]
    async fn test_cooldown_delegation_vault_ok() {
        let mut fixture = TestBuilder::new().await;

        let deposit_fee_bps = 0;
        let withdraw_fee_bps = 0;
        let reward_fee_bps = 0;
        let num_operators = 1;
        let slasher_amounts = vec![];

        let ConfiguredVault {
            mut vault_program_client,
            operator_roots,
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
            .do_cooldown_delegation(&vault_root, &operator_roots[0].operator_pubkey, 50_000)
            .await
            .unwrap();

        let vault = vault_program_client
            .get_vault(&vault_root.vault_pubkey)
            .await
            .unwrap();
        assert_eq!(vault.delegation_state.total_security().unwrap(), 100_000);
        assert_eq!(vault.delegation_state.staked_amount, 50_000);
        assert_eq!(vault.delegation_state.enqueued_for_cooldown_amount, 50_000);

        let vault_operator_delegation = vault_program_client
            .get_vault_operator_delegation(
                &vault_root.vault_pubkey,
                &operator_roots[0].operator_pubkey,
            )
            .await
            .unwrap();
        assert_eq!(
            vault_operator_delegation
                .delegation_state
                .total_security()
                .unwrap(),
            100_000
        );
        assert_eq!(
            vault_operator_delegation.delegation_state.staked_amount,
            50_000
        );
        assert_eq!(
            vault_operator_delegation
                .delegation_state
                .enqueued_for_cooldown_amount,
            50_000
        );
    }
}
