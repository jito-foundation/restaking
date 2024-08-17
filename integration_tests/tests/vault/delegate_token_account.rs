#[cfg(test)]
mod tests {
    use solana_program::{program_option::COption, pubkey::Pubkey};
    use solana_sdk::signature::{Keypair, Signer};
    use spl_associated_token_account::get_associated_token_address;

    use crate::fixtures::{fixture::TestBuilder, vault_client::VaultRoot};

    #[tokio::test]
    async fn test_delegate_token_account_ok() {
        let mut fixture = TestBuilder::new().await;

        let mut vault_program_client = fixture.vault_program_client();

        let (
            _config_admin,
            VaultRoot {
                vault_pubkey,
                vault_admin: _,
            },
        ) = vault_program_client
            .setup_config_and_vault(99, 100)
            .await
            .unwrap();

        let vault = vault_program_client.get_vault(&vault_pubkey).await.unwrap();

        // Initial deposit
        let depositor = Keypair::new();
        fixture.transfer(&depositor.pubkey(), 100.0).await.unwrap();
        fixture
            .mint_to(&vault.supported_mint, &depositor.pubkey(), 100_000)
            .await
            .unwrap();
        fixture
            .create_ata(&vault.vrt_mint, &depositor.pubkey())
            .await
            .unwrap();

        // Mint VRT tokens to depositor
        vault_program_client
            .mint_to(
                &vault_pubkey,
                &vault.vrt_mint,
                &depositor,
                &get_associated_token_address(&depositor.pubkey(), &vault.supported_mint),
                &get_associated_token_address(&vault_pubkey, &vault.supported_mint),
                &get_associated_token_address(&depositor.pubkey(), &vault.vrt_mint),
                &get_associated_token_address(&vault.fee_wallet, &vault.vrt_mint),
                None,
                100_000,
            )
            .await
            .unwrap();

        let token_account = vault_program_client
            .get_token_account(&depositor.pubkey(), &vault.vrt_mint)
            .await
            .unwrap();

        assert_eq!(token_account.delegate, COption::None);

        // Delegate
        let bob = Pubkey::new_unique();
        vault_program_client
            .delegate_token_account(
                &vault.vrt_mint,
                &get_associated_token_address(&depositor.pubkey(), &vault.vrt_mint),
                &depositor,
                &bob,
                50_000,
            )
            .await
            .unwrap();

        let token_account = vault_program_client
            .get_token_account(&depositor.pubkey(), &vault.vrt_mint)
            .await
            .unwrap();

        assert_eq!(token_account.delegate, COption::Some(bob));
        assert_eq!(token_account.delegated_amount, 50_000);
    }
}
