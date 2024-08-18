#[cfg(test)]
mod tests {
    use solana_program::{program_option::COption, pubkey::Pubkey};
    use solana_sdk::signature::{Keypair, Signer};
    use spl_associated_token_account::get_associated_token_address;
    use test_case::test_case;

    use crate::fixtures::{fixture::TestBuilder, vault_client::VaultRoot};

    async fn setup(token_program_id: &Pubkey) -> (TestBuilder, Pubkey, Keypair, Keypair, Keypair) {
        let mut fixture = TestBuilder::new().await;

        let mut vault_program_client = fixture.vault_program_client();

        let (
            _config_admin,
            VaultRoot {
                vault_pubkey,
                vault_admin,
            },
        ) = vault_program_client
            .setup_config_and_vault(99, 100)
            .await
            .unwrap();

        let random_mint = Keypair::new();
        vault_program_client
            ._create_token_mint(&random_mint, &token_program_id)
            .await
            .unwrap();

        let vault_token_account = Keypair::new();
        if token_program_id.eq(&spl_token::id()) {
            fixture
                .mint_spl_to(
                    &random_mint.pubkey(),
                    &vault_pubkey,
                    100_000,
                    &token_program_id,
                )
                .await
                .unwrap();
        } else {
            fixture
                .create_token_account(
                    &token_program_id,
                    &vault_token_account,
                    &random_mint.pubkey(),
                    &vault_pubkey,
                    &[],
                )
                .await
                .unwrap();
            fixture
                .mint_spl_to(
                    &random_mint.pubkey(),
                    &vault_token_account.pubkey(),
                    100_000,
                    &token_program_id,
                )
                .await
                .unwrap();
        }

        (
            fixture,
            vault_pubkey,
            vault_admin,
            random_mint,
            vault_token_account,
        )
    }

    #[test_case(spl_token::id(); "token")]
    #[test_case(spl_token_2022::id(); "token-2022")]
    #[tokio::test]
    async fn test_delegate_token_account_ok(token_program_id: Pubkey) {
        let (mut fixture, vault_pubkey, vault_admin, random_mint, vault_token_account) =
            setup(&token_program_id).await;
        let mut vault_program_client = fixture.vault_program_client();

        if token_program_id.eq(&spl_token::id()) {
            // Delegate
            let bob = Pubkey::new_unique();
            vault_program_client
                .delegate_token_account(
                    &vault_pubkey,
                    &vault_admin,
                    &random_mint.pubkey(),
                    &get_associated_token_address(&vault_pubkey, &random_mint.pubkey()),
                    &bob,
                    &token_program_id,
                    50_000,
                )
                .await
                .unwrap();
            let ata = get_associated_token_address(&vault_pubkey, &random_mint.pubkey());
            let token_account_acc = fixture.get_token_account(&ata).await.unwrap();

            assert_eq!(token_account_acc.delegate, COption::Some(bob));
            assert_eq!(token_account_acc.delegated_amount, 50_000);
        } else {
            let bob = Pubkey::new_unique();
            vault_program_client
                .delegate_token_account(
                    &vault_pubkey,
                    &vault_admin,
                    &random_mint.pubkey(),
                    &vault_token_account.pubkey(),
                    &bob,
                    &token_program_id,
                    50_000,
                )
                .await
                .unwrap();

            let vault_token_acc = fixture
                .get_token_account(&vault_token_account.pubkey())
                .await
                .unwrap();

            assert_eq!(vault_token_acc.delegate, COption::Some(bob));
            assert_eq!(vault_token_acc.delegated_amount, 50_000);
        }
    }

    #[test_case(spl_token::id(); "token")]
    #[test_case(spl_token_2022::id(); "token-2022")]
    #[tokio::test]
    async fn test_delegate_vault_supprted_token_account_err(token_program_id: Pubkey) {
        let (fixture, vault_pubkey, vault_admin, random_mint, vault_token_account) =
            setup(&token_program_id).await;
        let mut vault_program_client = fixture.vault_program_client();

        let vault = vault_program_client.get_vault(&vault_pubkey).await.unwrap();

        if token_program_id.eq(&spl_token::id()) {
            // Delegate
            let bob = Pubkey::new_unique();
            let response = vault_program_client
                .delegate_token_account(
                    &vault_pubkey,
                    &vault_admin,
                    &&vault.supported_mint,
                    &get_associated_token_address(&&vault_pubkey, &random_mint.pubkey()),
                    &bob,
                    &token_program_id,
                    50_000,
                )
                .await;

            assert!(response.is_err());
        } else {
            let bob = Pubkey::new_unique();
            let response = vault_program_client
                .delegate_token_account(
                    &vault_pubkey,
                    &vault_admin,
                    &&vault.supported_mint,
                    &vault_token_account.pubkey(),
                    &bob,
                    &token_program_id,
                    50_000,
                )
                .await;

            assert!(response.is_err());
        }
    }
}
