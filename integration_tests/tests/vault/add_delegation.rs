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
        const MINT_AMOUNT: u64 = 100_000;
        let mut fixture = TestBuilder::new().await;
        let ConfiguredVault {
            mut vault_program_client,
            vault_root,
            operator_roots,
            ..
        } = fixture
            .setup_vault_with_ncn_and_operators(0, 0, 1, &[])
            .await
            .unwrap();

        // setup depositor, mint, deposit and delegate
        let depositor = Keypair::new();
        vault_program_client
            .configure_depositor(&vault_root, &depositor.pubkey(), 100_000)
            .await
            .unwrap();
        vault_program_client
            .do_mint_to(&vault_root, &depositor, MINT_AMOUNT)
            .await
            .unwrap();

        vault_program_client
            .do_add_delegation(&vault_root, &operator_roots[0].operator_pubkey, MINT_AMOUNT)
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
            MINT_AMOUNT
        );

        let vault = vault_program_client
            .get_vault(&vault_root.vault_pubkey)
            .await
            .unwrap();
        assert_eq!(
            vault.delegation_state,
            vault_operator_delegation.delegation_state
        );
        assert_eq!(vault.tokens_deposited, MINT_AMOUNT);
        assert_eq!(vault.vrt_supply, MINT_AMOUNT);
    }

    #[tokio::test]
    async fn test_add_delegation_over_delegate_fails() {
        const MINT_AMOUNT: u64 = 100_000;
        let mut fixture = TestBuilder::new().await;
        let ConfiguredVault {
            mut vault_program_client,
            vault_root,
            operator_roots,
            ..
        } = fixture
            .setup_vault_with_ncn_and_operators(0, 0, 1, &[])
            .await
            .unwrap();

        // setup depositor, mint, deposit and delegate
        let depositor = Keypair::new();
        vault_program_client
            .configure_depositor(&vault_root, &depositor.pubkey(), 100_000)
            .await
            .unwrap();
        vault_program_client
            .do_mint_to(&vault_root, &depositor, MINT_AMOUNT)
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
