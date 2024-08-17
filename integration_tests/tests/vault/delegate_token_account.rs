#[cfg(test)]
mod tests {
    use solana_program::{program_option::COption, pubkey::Pubkey};
    use solana_sdk::signature::{Keypair, Signer};
    use spl_associated_token_account::get_associated_token_address;
    use test_case::test_case;

    use crate::fixtures::{fixture::TestBuilder, vault_client::VaultRoot};

    #[test_case(spl_token::id(); "token")]
    #[tokio::test]
    async fn test_delegate_token_account_ok(token_program_id: Pubkey) {
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

        // Initial deposit
        let random_mint = Keypair::new();
        vault_program_client
            ._create_token_mint(&random_mint, &token_program_id)
            .await
            .unwrap();
        fixture
            .create_ata(&random_mint.pubkey(), &vault_pubkey, &token_program_id)
            .await
            .unwrap();
        fixture
            .mint_spl_to(
                &random_mint.pubkey(),
                &vault_pubkey,
                100_000,
                &token_program_id,
            )
            .await
            .unwrap();

        let token_account = vault_program_client
            .get_token_account(&&vault_pubkey, &random_mint.pubkey())
            .await
            .unwrap();

        assert_eq!(token_account.amount, 100_000);
        assert_eq!(token_account.delegate, COption::None);

        // Delegate
        let bob = Pubkey::new_unique();
        vault_program_client
            .delegate_token_account(
                &vault_pubkey,
                &vault_admin,
                &random_mint.pubkey(),
                &get_associated_token_address(&&vault_pubkey, &random_mint.pubkey()),
                &bob,
                &token_program_id,
                50_000,
            )
            .await
            .unwrap();

        let token_account = vault_program_client
            .get_token_account(&vault_pubkey, &random_mint.pubkey())
            .await
            .unwrap();

        assert_eq!(token_account.delegate, COption::Some(bob));
        assert_eq!(token_account.delegated_amount, 50_000);
    }
}
