use solana_sdk::signature::{Keypair, Signer};
use spl_associated_token_account::get_associated_token_address;

use crate::fixtures::fixture::TestBuilder;

#[tokio::test]
async fn test_enqueue_withdrawal_success() {
    let mut fixture = TestBuilder::new().await;

    let mut vault_program_client = fixture.vault_program_client();

    let (_vault_config_admin, vault_root) =
        vault_program_client.setup_vault(100, 100).await.unwrap();

    let vault = vault_program_client
        .get_vault(&vault_root.vault_pubkey)
        .await
        .unwrap();

    let depositor = Keypair::new();
    fixture.transfer(&depositor.pubkey(), 100.0).await.unwrap();
    fixture
        .mint_to(&vault.supported_mint(), &depositor.pubkey(), 100_000)
        .await
        .unwrap();

    fixture
        .create_ata(&vault.lrt_mint(), &depositor.pubkey())
        .await
        .unwrap();

    let depositor_lrt_token_account =
        get_associated_token_address(&depositor.pubkey(), &vault.lrt_mint());
    vault_program_client
        .mint_to(
            &vault_root.vault_pubkey,
            &vault.lrt_mint(),
            &depositor,
            &get_associated_token_address(&depositor.pubkey(), &vault.supported_mint()),
            &get_associated_token_address(&vault_root.vault_pubkey, &vault.supported_mint()),
            &get_associated_token_address(&depositor.pubkey(), &vault.lrt_mint()),
            &get_associated_token_address(&vault.fee_owner(), &vault.lrt_mint()),
            None,
            100_000,
        )
        .await
        .unwrap();

    let depositor_ata = fixture
        .get_token_account(&depositor_lrt_token_account)
        .await
        .unwrap();
    assert_eq!(depositor_ata.amount, 99_000);

    vault_program_client
        .do_enqueue_withdraw(&vault_root, &depositor, 49_500)
        .await
        .unwrap();

    // let withdrawal = vault_program_client
    //     .enqueue_withdraw(&vault_root, &depositor, &depositor_lrt_token_account, 1000)
    //     .await
    //     .unwrap();
}
