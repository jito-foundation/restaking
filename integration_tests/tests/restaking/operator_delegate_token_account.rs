#[cfg(test)]
mod tests {
    use solana_program::{program_option::COption, pubkey::Pubkey};
    use solana_sdk::{signature::Keypair, signer::Signer};
    use spl_associated_token_account::get_associated_token_address;
    use test_case::test_case;

    use crate::fixtures::{fixture::TestBuilder, restaking_client::OperatorRoot};

    async fn setup(token_program_id: &Pubkey) -> (TestBuilder, OperatorRoot, Keypair, Keypair) {
        let mut fixture = TestBuilder::new().await;

        let mut restaking_program_client = fixture.restaking_program_client();

        let _config_admin = restaking_program_client
            .do_initialize_config()
            .await
            .unwrap();
        let _ncn_root = restaking_program_client.do_initialize_ncn().await.unwrap();
        let operator_root = restaking_program_client
            .do_initialize_operator()
            .await
            .unwrap();

        let random_mint = Keypair::new();
        fixture
            .vault_program_client()
            ._create_token_mint(&random_mint, &token_program_id)
            .await
            .unwrap();

        let operator_token_account = Keypair::new();
        if token_program_id.eq(&spl_token::id()) {
            fixture
                .mint_spl_to(
                    &random_mint.pubkey(),
                    &operator_root.operator_pubkey,
                    100_000,
                    &token_program_id,
                )
                .await
                .unwrap();
        } else {
            fixture
                .create_token_account(
                    &token_program_id,
                    &operator_token_account,
                    &random_mint.pubkey(),
                    &operator_root.operator_pubkey,
                    &[],
                )
                .await
                .unwrap();
            fixture
                .mint_spl_to(
                    &random_mint.pubkey(),
                    &operator_token_account.pubkey(),
                    100_000,
                    &token_program_id,
                )
                .await
                .unwrap();
        }

        (fixture, operator_root, random_mint, operator_token_account)
    }

    #[test_case(spl_token::id(); "token")]
    #[test_case(spl_token_2022::id(); "token-2022")]
    #[tokio::test]
    async fn test_operator_token_account_ok(token_program_id: Pubkey) {
        let (mut fixture, operator_root, random_mint, operator_token_account) =
            setup(&token_program_id).await;
        let mut restaking_program_client = fixture.restaking_program_client();

        if token_program_id.eq(&spl_token::id()) {
            // Delegate
            let bob = Pubkey::new_unique();
            restaking_program_client
                .operator_delegate_token_account(
                    &operator_root.operator_pubkey,
                    &operator_root.operator_admin,
                    &random_mint.pubkey(),
                    &get_associated_token_address(
                        &operator_root.operator_pubkey,
                        &random_mint.pubkey(),
                    ),
                    &bob,
                    &token_program_id,
                    50_000,
                )
                .await
                .unwrap();
            let ata =
                get_associated_token_address(&operator_root.operator_pubkey, &random_mint.pubkey());
            let token_account_acc = fixture.get_token_account(&ata).await.unwrap();

            assert_eq!(token_account_acc.delegate, COption::Some(bob));
            assert_eq!(token_account_acc.delegated_amount, 50_000);
        } else {
            let bob = Pubkey::new_unique();
            restaking_program_client
                .operator_delegate_token_account(
                    &operator_root.operator_pubkey,
                    &operator_root.operator_admin,
                    &random_mint.pubkey(),
                    &operator_token_account.pubkey(),
                    &bob,
                    &token_program_id,
                    50_000,
                )
                .await
                .unwrap();

            let vault_token_acc = fixture
                .get_token_account(&operator_token_account.pubkey())
                .await
                .unwrap();

            assert_eq!(vault_token_acc.delegate, COption::Some(bob));
            assert_eq!(vault_token_acc.delegated_amount, 50_000);
        }
    }
}
