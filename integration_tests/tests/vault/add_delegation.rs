#[cfg(test)]
mod tests {
    use jito_vault_sdk::error::VaultError;
    use solana_sdk::signature::{Keypair, Signer};

    use crate::fixtures::{
        fixture::{ConfiguredVault, TestBuilder},
        vault_client::assert_vault_error,
    };

    #[tokio::test]
    async fn test_add_delegation_ok() {
        const AMOUNT_IN: u64 = 100_000;
        const MIN_AMOUNT_OUT: u64 = 100_000;
        let mut fixture = TestBuilder::new().await;

        let deposit_fee_bps = 0;
        let withdraw_fee_bps = 0;
        let reward_fee_bps = 0;
        let epoch_withdraw_cap_bps = 2500; // 25%
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
                epoch_withdraw_cap_bps,
                num_operators,
                &slasher_amounts,
            )
            .await
            .unwrap();

        // setup depositor, mint, deposit and delegate
        let depositor = Keypair::new();
        vault_program_client
            .configure_depositor(&vault_root, &depositor.pubkey(), 100_000)
            .await
            .unwrap();
        vault_program_client
            .do_mint_to(&vault_root, &depositor, AMOUNT_IN, MIN_AMOUNT_OUT)
            .await
            .unwrap();

        vault_program_client
            .do_add_delegation(&vault_root, &operator_roots[0].operator_pubkey, AMOUNT_IN)
            .await
            .unwrap();

        let vault_operator_delegation = vault_program_client
            .get_vault_operator_delegation(
                &vault_root.vault_pubkey,
                &operator_roots[0].operator_pubkey,
            )
            .await
            .unwrap();
        assert_eq!(vault_operator_delegation.vault, vault_root.vault_pubkey);
        assert_eq!(
            vault_operator_delegation.operator,
            operator_roots[0].operator_pubkey
        );
        assert_eq!(
            vault_operator_delegation
                .delegation_state
                .total_security()
                .unwrap(),
            AMOUNT_IN
        );

        let vault = vault_program_client
            .get_vault(&vault_root.vault_pubkey)
            .await
            .unwrap();
        assert_eq!(
            vault.delegation_state,
            vault_operator_delegation.delegation_state
        );
        assert_eq!(vault.tokens_deposited, AMOUNT_IN);
        assert_eq!(vault.vrt_supply, AMOUNT_IN);
    }

    #[tokio::test]
    async fn test_add_delegation_over_delegate_fails() {
        const MINT_AMOUNT: u64 = 100_000;
        const MIN_AMOUNT_OUT: u64 = 100_000;
        let mut fixture = TestBuilder::new().await;

        let deposit_fee_bps = 0;
        let withdraw_fee_bps = 0;
        let reward_fee_bps = 0;
        let epoch_withdraw_cap_bps = 2500; // 25%
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
                epoch_withdraw_cap_bps,
                num_operators,
                &slasher_amounts,
            )
            .await
            .unwrap();

        // setup depositor, mint, deposit and delegate
        let depositor = Keypair::new();
        vault_program_client
            .configure_depositor(&vault_root, &depositor.pubkey(), 100_000)
            .await
            .unwrap();
        vault_program_client
            .do_mint_to(&vault_root, &depositor, MINT_AMOUNT, MIN_AMOUNT_OUT)
            .await
            .unwrap();

        vault_program_client
            .do_add_delegation(&vault_root, &operator_roots[0].operator_pubkey, 50_000)
            .await
            .unwrap();

        let result = vault_program_client
            .do_add_delegation(&vault_root, &operator_roots[0].operator_pubkey, 50_001)
            .await;
        assert_vault_error(result, VaultError::VaultInsufficientFunds);
    }
}
