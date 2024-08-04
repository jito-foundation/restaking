use solana_sdk::signature::{Keypair, Signer};
use spl_associated_token_account::get_associated_token_address;

use crate::fixtures::fixture::TestBuilder;

#[tokio::test]
async fn test_slash_ok() {
    let mut fixture = TestBuilder::new().await;

    let mut restaking_program_client = fixture.restaking_program_client();
    let mut vault_program_client = fixture.vault_program_client();

    let (_config_admin, vault_root) = vault_program_client.setup_vault(100, 100).await.unwrap();

    let _restaking_config_admin = restaking_program_client.setup_config().await.unwrap();

    let avs_root = restaking_program_client.setup_avs().await.unwrap();
    let operator_root = restaking_program_client.setup_operator().await.unwrap();

    // AVS <-> Vault
    restaking_program_client
        .avs_vault_opt_in(&avs_root, &vault_root.vault_pubkey)
        .await
        .unwrap();
    vault_program_client
        .vault_avs_opt_in(&vault_root, &avs_root.avs_pubkey)
        .await
        .unwrap();

    // AVS <-> Operator
    // operator needs to opt-in first
    restaking_program_client
        .operator_avs_opt_in(&operator_root, &avs_root.avs_pubkey)
        .await
        .unwrap();
    restaking_program_client
        .avs_operator_opt_in(&avs_root, &operator_root.operator_pubkey)
        .await
        .unwrap();

    // Vault <-> Operator
    // operator needs to opt-in first
    restaking_program_client
        .operator_vault_opt_in(&operator_root, &vault_root.vault_pubkey)
        .await
        .unwrap();
    vault_program_client
        .vault_operator_opt_in(&vault_root, &operator_root.operator_pubkey)
        .await
        .unwrap();

    // AVS + vault configures slasher
    let slasher = Keypair::new();
    fixture.transfer(&slasher.pubkey(), 1.0).await.unwrap();

    restaking_program_client
        .avs_vault_slasher_opt_in(&avs_root, &vault_root.vault_pubkey, &slasher.pubkey(), 100)
        .await
        .unwrap();
    vault_program_client
        .vault_avs_vault_slasher_opt_in(&vault_root, &avs_root.avs_pubkey, &slasher.pubkey())
        .await
        .unwrap();

    let vault = vault_program_client
        .get_vault(&vault_root.vault_pubkey)
        .await
        .unwrap();

    let depositor = Keypair::new();
    fixture.transfer(&depositor.pubkey(), 1.0).await.unwrap();
    fixture
        .mint_to(&vault.supported_mint(), &depositor.pubkey(), 100_000)
        .await
        .unwrap();

    let vault = vault_program_client
        .get_vault(&vault_root.vault_pubkey)
        .await
        .unwrap();
    // depositor ATA for LRT
    fixture
        .create_ata(&vault.lrt_mint(), &depositor.pubkey())
        .await
        .unwrap();

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

    // user has 99_000 because 100 bips deposit fee

    vault_program_client
        .delegate(&vault_root, &operator_root.operator_pubkey, 10_000)
        .await
        .unwrap();

    let vault_delegation_list = vault_program_client
        .get_vault_delegation_list(&vault_root.vault_pubkey)
        .await
        .unwrap();

    let delegations = vault_delegation_list.delegations();
    assert_eq!(delegations.len(), 1);
    assert_eq!(delegations[0].operator(), operator_root.operator_pubkey);
    assert_eq!(delegations[0].staked_amount(), 10_000);

    fixture
        .create_ata(&vault.supported_mint(), &slasher.pubkey())
        .await
        .unwrap();

    vault_program_client
        .setup_vault_avs_slasher_operator_ticket(
            &vault_root,
            &avs_root.avs_pubkey,
            &slasher.pubkey(),
            &operator_root.operator_pubkey,
        )
        .await
        .unwrap();

    vault_program_client
        .do_slash(
            &vault_root,
            &avs_root.avs_pubkey,
            &slasher,
            &operator_root.operator_pubkey,
            100,
        )
        .await
        .unwrap();

    let vault = vault_program_client
        .get_vault(&vault_root.vault_pubkey)
        .await
        .unwrap();
    assert_eq!(vault.tokens_deposited(), 99_900);

    let delegation_list = vault_program_client
        .get_vault_delegation_list(&vault_root.vault_pubkey)
        .await
        .unwrap();
    let delegations = delegation_list.delegations();
    assert_eq!(delegations.len(), 1);
    assert_eq!(delegations[0].operator(), operator_root.operator_pubkey);
    assert_eq!(delegations[0].staked_amount(), 9_900);

    let vault_avs_slasher_operator_ticket = vault_program_client
        .get_vault_avs_slasher_operator_ticket(
            &vault_root.vault_pubkey,
            &avs_root.avs_pubkey,
            &slasher.pubkey(),
            &operator_root.operator_pubkey,
            0,
        )
        .await
        .unwrap();
    assert_eq!(vault_avs_slasher_operator_ticket.slashed(), 100);
    assert_eq!(vault_avs_slasher_operator_ticket.epoch(), 0);
    assert_eq!(
        vault_avs_slasher_operator_ticket.vault(),
        vault_root.vault_pubkey
    );
    assert_eq!(vault_avs_slasher_operator_ticket.avs(), avs_root.avs_pubkey);
    assert_eq!(
        vault_avs_slasher_operator_ticket.slasher(),
        slasher.pubkey()
    );
    assert_eq!(
        vault_avs_slasher_operator_ticket.operator(),
        operator_root.operator_pubkey
    );
}
