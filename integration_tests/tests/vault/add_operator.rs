use jito_restaking_core::{
    config::Config as RestakingConfig, operator::Operator,
    operator_vault_ticket::OperatorVaultTicket,
};
use jito_vault_core::{
    config::Config as VaultConfig, vault::Vault, vault_delegation_list::VaultDelegationList,
    vault_operator_ticket::VaultOperatorTicket,
};
use solana_sdk::signature::{Keypair, Signer};

use crate::fixtures::fixture::TestBuilder;

#[tokio::test]
async fn test_add_operator_ok() {
    let mut fixture = TestBuilder::new().await;

    let mut restaking_program_client = fixture.restaking_program_client();
    let mut vault_program_client = fixture.vault_program_client();

    let backing_token_mint = Keypair::new();
    fixture
        .create_token_mint(&backing_token_mint)
        .await
        .unwrap();

    // create vault config
    let vault_config_pubkey = VaultConfig::find_program_address(&jito_vault_program::id()).0;
    let vault_config_admin = Keypair::new();

    fixture
        .transfer(&vault_config_admin.pubkey(), 1.0)
        .await
        .unwrap();

    vault_program_client
        .initialize_config(&vault_config_pubkey, &vault_config_admin)
        .await
        .unwrap();

    let config_account = vault_program_client
        .get_config(&vault_config_pubkey)
        .await
        .unwrap();

    // create vault
    let vault_base = Keypair::new();
    let vault_pubkey =
        Vault::find_program_address(&jito_vault_program::id(), &vault_base.pubkey()).0;
    let vault_delegation_list =
        VaultDelegationList::find_program_address(&jito_vault_program::id(), &vault_pubkey).0;
    let lrt_mint = Keypair::new();
    let vault_admin = Keypair::new();

    fixture.transfer(&vault_admin.pubkey(), 1.0).await.unwrap();

    vault_program_client
        .initialize_vault(
            &vault_config_pubkey,
            &vault_pubkey,
            &vault_delegation_list,
            &lrt_mint,
            &backing_token_mint,
            &vault_admin,
            &vault_base,
            100,
            100,
        )
        .await
        .unwrap();

    let restaking_config_pubkey =
        RestakingConfig::find_program_address(&jito_restaking_program::id()).0;
    let restaking_config_admin = Keypair::new();

    fixture
        .transfer(&restaking_config_admin.pubkey(), 1.0)
        .await
        .unwrap();
    restaking_program_client
        .initialize_config(&restaking_config_pubkey, &restaking_config_admin)
        .await
        .unwrap();

    // create operator + add operator vault
    let operator_base = Keypair::new();
    let operator_pubkey =
        Operator::find_program_address(&jito_restaking_program::id(), &operator_base.pubkey()).0;
    let operator_admin = Keypair::new();
    fixture
        .transfer(&operator_admin.pubkey(), 1.0)
        .await
        .unwrap();
    restaking_program_client
        .initialize_operator(
            &restaking_config_pubkey,
            &operator_pubkey,
            &operator_admin,
            &operator_base,
        )
        .await
        .unwrap();

    let operator_vault_ticket = OperatorVaultTicket::find_program_address(
        &jito_restaking_program::id(),
        &operator_pubkey,
        &vault_pubkey,
    )
    .0;
    restaking_program_client
        .operator_add_vault(
            &restaking_config_pubkey,
            &operator_pubkey,
            &vault_pubkey,
            &operator_vault_ticket,
            &operator_admin,
            &operator_admin,
        )
        .await
        .unwrap();

    fixture
        .warp_slot_incremental(config_account.epoch_length() * 2)
        .await
        .unwrap();

    let vault_operator_ticket = VaultOperatorTicket::find_program_address(
        &jito_vault_program::id(),
        &vault_pubkey,
        &operator_pubkey,
    )
    .0;
    vault_program_client
        .add_operator(
            &vault_config_pubkey,
            &vault_pubkey,
            &operator_pubkey,
            &operator_vault_ticket,
            &vault_operator_ticket,
            &vault_admin,
            &vault_admin,
        )
        .await
        .unwrap();

    let vault_operator_ticket = vault_program_client
        .get_vault_operator_ticket(&vault_pubkey, &operator_pubkey)
        .await
        .unwrap();
    assert_eq!(vault_operator_ticket.vault(), vault_pubkey);
    assert_eq!(vault_operator_ticket.operator(), operator_pubkey);
    assert_eq!(vault_operator_ticket.index(), 0);
    assert_eq!(
        vault_operator_ticket.state().slot_added(),
        fixture.get_current_slot().await.unwrap()
    );
}
