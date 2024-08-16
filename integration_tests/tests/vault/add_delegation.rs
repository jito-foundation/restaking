#[cfg(test)]
mod tests {
    use crate::fixtures::fixture::{ConfiguredVault, TestBuilder};
    use solana_sdk::signature::{Keypair, Signer};
    use spl_associated_token_account::get_associated_token_address;

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

        let vault = vault_program_client
            .get_vault(&vault_root.vault_pubkey)
            .await
            .unwrap();

        // setup depositor, mint, deposit and delegate
        let depositor = Keypair::new();
        fixture.transfer(&depositor.pubkey(), 100.0).await.unwrap();
        fixture
            .mint_to(&vault.supported_mint, &depositor.pubkey(), MINT_AMOUNT)
            .await
            .unwrap();
        fixture
            .create_ata(&vault.vrt_mint, &depositor.pubkey())
            .await
            .unwrap();
        vault_program_client
            .mint_to(
                &vault_root.vault_pubkey,
                &vault.vrt_mint,
                &depositor,
                &get_associated_token_address(&depositor.pubkey(), &vault.supported_mint),
                &get_associated_token_address(&vault_root.vault_pubkey, &vault.supported_mint),
                &get_associated_token_address(&depositor.pubkey(), &vault.vrt_mint),
                &get_associated_token_address(&vault.fee_wallet, &vault.vrt_mint),
                None,
                MINT_AMOUNT,
            )
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
            vault_operator_delegation.delegation_state.staked_amount,
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
    }
}
