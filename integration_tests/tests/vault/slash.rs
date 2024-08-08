use jito_restaking_core::{
    avs::Avs, avs_operator_ticket::AvsOperatorTicket,
    avs_vault_slasher_ticket::AvsVaultSlasherTicket, avs_vault_ticket::AvsVaultTicket,
    config::Config as RestakingConfig, operator::Operator, operator_avs_ticket::OperatorAvsTicket,
    operator_vault_ticket::OperatorVaultTicket,
};
use jito_vault_core::{
    config::Config as VaultConfig, vault::Vault,
    vault_avs_slasher_operator_ticket::VaultAvsSlasherOperatorTicket,
    vault_avs_slasher_ticket::VaultAvsSlasherTicket, vault_avs_ticket::VaultAvsTicket,
    vault_delegation_list::VaultDelegationList, vault_operator_ticket::VaultOperatorTicket,
};
use solana_sdk::signature::{Keypair, Signer};
use spl_associated_token_account::get_associated_token_address;

use crate::fixtures::fixture::TestBuilder;

#[tokio::test]
async fn test_slash_ok() {
    let mut fixture = TestBuilder::new().await;
    let mut restaking_program_client = fixture.restaking_program_client();
    let mut vault_program_client = fixture.vault_program_client();

    // Initialize restaking config
    let config_admin = Keypair::new();
    let restaking_config = RestakingConfig::find_program_address(&jito_restaking_program::id()).0;
    fixture
        .transfer(&config_admin.pubkey(), 10.0)
        .await
        .unwrap();
    restaking_program_client
        .initialize_config(&restaking_config, &config_admin)
        .await
        .unwrap();

    let restaking_config_account = restaking_program_client
        .get_config(&restaking_config)
        .await
        .unwrap();

    // Initialize AVS
    let avs_admin = Keypair::new();
    let avs_base = Keypair::new();
    fixture.transfer(&avs_admin.pubkey(), 10.0).await.unwrap();
    let avs_pubkey = Avs::find_program_address(&jito_restaking_program::id(), &avs_base.pubkey()).0;
    restaking_program_client
        .initialize_avs(&restaking_config, &avs_pubkey, &avs_admin, &avs_base)
        .await
        .unwrap();

    // Initialize operator
    let operator_admin = Keypair::new();
    let operator_base = Keypair::new();
    fixture
        .transfer(&operator_admin.pubkey(), 1.0)
        .await
        .unwrap();
    let operator_pubkey =
        Operator::find_program_address(&jito_restaking_program::id(), &operator_base.pubkey()).0;
    restaking_program_client
        .initialize_operator(
            &restaking_config,
            &operator_pubkey,
            &operator_admin,
            &operator_base,
        )
        .await
        .unwrap();

    // Initialize vault config
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

    let vault_config_account = vault_program_client
        .get_config(&vault_config_pubkey)
        .await
        .unwrap();

    let max_epoch_length = restaking_config_account
        .epoch_length()
        .max(vault_config_account.epoch_length());

    // Initialize Vault
    let vault_base = Keypair::new();
    let vault_pubkey =
        Vault::find_program_address(&jito_vault_program::id(), &vault_base.pubkey()).0;
    let vault_delegate_list_pubkey =
        VaultDelegationList::find_program_address(&jito_vault_program::id(), &vault_pubkey).0;
    let lrt_mint = Keypair::new();
    let token_mint = Keypair::new();
    let vault_admin = Keypair::new();

    fixture.create_token_mint(&token_mint).await.unwrap();
    fixture.transfer(&vault_admin.pubkey(), 1.0).await.unwrap();

    vault_program_client
        .initialize_vault(
            &vault_config_pubkey,
            &vault_pubkey,
            &vault_delegate_list_pubkey,
            &lrt_mint,
            &token_mint,
            &vault_admin,
            &vault_base,
            100,
            100,
        )
        .await
        .unwrap();

    let avs_vault_ticket_pubkey = AvsVaultTicket::find_program_address(
        &jito_restaking_program::id(),
        &avs_pubkey,
        &vault_pubkey,
    )
    .0;

    // AVS adds vault
    restaking_program_client
        .avs_add_vault(
            &restaking_config,
            &avs_pubkey,
            &vault_pubkey,
            &avs_vault_ticket_pubkey,
            &avs_admin,
            &avs_admin,
        )
        .await
        .unwrap();

    // Operator adds vault
    let operator_vault_ticket_pubkey = OperatorVaultTicket::find_program_address(
        &jito_restaking_program::id(),
        &operator_pubkey,
        &vault_pubkey,
    )
    .0;
    restaking_program_client
        .operator_add_vault(
            &restaking_config,
            &operator_pubkey,
            &vault_pubkey,
            &operator_vault_ticket_pubkey,
            &operator_admin,
            &operator_admin,
        )
        .await
        .unwrap();

    // operator adds avs
    let operator_avs_ticket_pubkey = OperatorAvsTicket::find_program_address(
        &jito_restaking_program::id(),
        &operator_pubkey,
        &avs_pubkey,
    )
    .0;
    restaking_program_client
        .operator_add_avs(
            &restaking_config,
            &operator_pubkey,
            &avs_pubkey,
            &operator_avs_ticket_pubkey,
            &operator_admin,
            &operator_admin,
        )
        .await
        .unwrap();

    fixture
        .warp_slot_incremental(max_epoch_length * 2)
        .await
        .unwrap();

    // avs adds operator
    let avs_operator_ticket_pubkey = AvsOperatorTicket::find_program_address(
        &jito_restaking_program::id(),
        &avs_pubkey,
        &operator_pubkey,
    )
    .0;
    restaking_program_client
        .avs_add_operator(
            &restaking_config,
            &avs_pubkey,
            &operator_pubkey,
            &avs_operator_ticket_pubkey,
            &operator_avs_ticket_pubkey,
            &avs_admin,
            &avs_admin,
        )
        .await
        .unwrap();

    // vault adds avs
    let vault_avs_ticket_pubkey =
        VaultAvsTicket::find_program_address(&jito_vault_program::id(), &vault_pubkey, &avs_pubkey)
            .0;
    vault_program_client
        .add_avs(
            &vault_config_pubkey,
            &vault_pubkey,
            &avs_pubkey,
            &avs_vault_ticket_pubkey,
            &vault_avs_ticket_pubkey,
            &vault_admin,
            &vault_admin,
        )
        .await
        .unwrap();

    // vault adds operator
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
            &operator_vault_ticket_pubkey,
            &vault_operator_ticket,
            &vault_admin,
            &vault_admin,
        )
        .await
        .unwrap();

    // AVS adds slasher
    let slasher = Keypair::new();
    fixture.transfer(&slasher.pubkey(), 1.0).await.unwrap();
    let avs_slasher_ticket_pubkey = AvsVaultSlasherTicket::find_program_address(
        &jito_restaking_program::id(),
        &avs_pubkey,
        &vault_pubkey,
        &slasher.pubkey(),
    )
    .0;

    restaking_program_client
        .avs_add_vault_slasher(
            &restaking_config,
            &avs_pubkey,
            &vault_pubkey,
            &slasher.pubkey(),
            &avs_vault_ticket_pubkey,
            &avs_slasher_ticket_pubkey,
            &avs_admin,
            &avs_admin,
            100,
        )
        .await
        .unwrap();

    fixture
        .warp_slot_incremental(max_epoch_length * 2)
        .await
        .unwrap();

    // vault adds slasher
    let vault_slasher_ticket_pubkey = VaultAvsSlasherTicket::find_program_address(
        &jito_vault_program::id(),
        &vault_pubkey,
        &avs_pubkey,
        &slasher.pubkey(),
    )
    .0;

    vault_program_client
        .add_slasher(
            &vault_config_pubkey,
            &vault_pubkey,
            &avs_pubkey,
            &slasher.pubkey(),
            &avs_slasher_ticket_pubkey,
            &vault_slasher_ticket_pubkey,
            &vault_admin,
            &vault_admin,
        )
        .await
        .unwrap();

    let depositor = Keypair::new();
    fixture.transfer(&depositor.pubkey(), 1.0).await.unwrap();
    fixture
        .mint_to(&token_mint.pubkey(), &depositor.pubkey(), 100_000)
        .await
        .unwrap();

    let depositor_token_account =
        get_associated_token_address(&depositor.pubkey(), &token_mint.pubkey());
    let depositor_lrt_token_account =
        get_associated_token_address(&depositor.pubkey(), &lrt_mint.pubkey());
    let vault_fee_token_account =
        get_associated_token_address(&vault_admin.pubkey(), &lrt_mint.pubkey());
    let vault_token_account = get_associated_token_address(&vault_pubkey, &token_mint.pubkey());

    // deposit lrt receiver
    fixture
        .create_ata(&lrt_mint.pubkey(), &depositor.pubkey())
        .await
        .unwrap();

    // vault fee account
    fixture
        .create_ata(&lrt_mint.pubkey(), &vault_admin.pubkey())
        .await
        .unwrap();
    // vault holdings
    fixture
        .create_ata(&token_mint.pubkey(), &vault_pubkey)
        .await
        .unwrap();

    vault_program_client
        .mint_to(
            &vault_pubkey,
            &lrt_mint.pubkey(),
            &depositor,
            &depositor_token_account,
            &vault_token_account,
            &depositor_lrt_token_account,
            &vault_fee_token_account,
            None,
            100_000,
        )
        .await
        .unwrap();

    let vault = vault_program_client.get_vault(&vault_pubkey).await.unwrap();
    assert_eq!(vault.lrt_supply(), 100_000);

    let fee_account = fixture
        .get_token_account(&vault_fee_token_account)
        .await
        .unwrap();
    assert_eq!(fee_account.amount, 1_000);

    let user_account = fixture
        .get_token_account(&depositor_lrt_token_account)
        .await
        .unwrap();
    assert_eq!(user_account.amount, 99_000);

    vault_program_client
        .add_delegation(
            &vault_config_pubkey,
            &vault_pubkey,
            &operator_pubkey,
            &vault_operator_ticket,
            &vault_delegate_list_pubkey,
            &vault_admin,
            &vault_admin,
            10_000,
        )
        .await
        .unwrap();

    fixture
        .warp_slot_incremental(max_epoch_length * 2)
        .await
        .unwrap();

    let vault_delegation_list = vault_program_client
        .get_vault_delegation_list(&vault_delegate_list_pubkey)
        .await
        .unwrap();
    let delegations = vault_delegation_list.delegations();
    assert_eq!(delegations.len(), 1);
    assert_eq!(delegations[0].operator(), operator_pubkey);
    assert_eq!(delegations[0].active_amount(), 10_000);

    let slasher_token_account =
        get_associated_token_address(&slasher.pubkey(), &token_mint.pubkey());
    fixture
        .create_ata(&token_mint.pubkey(), &slasher.pubkey())
        .await
        .unwrap();

    let vault_avs_slasher_ticket = VaultAvsSlasherTicket::find_program_address(
        &jito_vault_program::id(),
        &vault_pubkey,
        &avs_pubkey,
        &slasher.pubkey(),
    )
    .0;

    let current_epoch = fixture.get_current_epoch(max_epoch_length).await.unwrap();

    let vault_avs_slasher_operator_ticket = VaultAvsSlasherOperatorTicket::find_program_address(
        &jito_vault_program::id(),
        &vault_pubkey,
        &avs_pubkey,
        &slasher.pubkey(),
        &operator_pubkey,
        current_epoch,
    )
    .0;

    vault_program_client
        .initialize_vault_avs_slasher_operator_ticket(
            &vault_config_pubkey,
            &vault_pubkey,
            &avs_pubkey,
            &slasher.pubkey(),
            &operator_pubkey,
            &vault_avs_slasher_ticket,
            &vault_avs_slasher_operator_ticket,
            &vault_admin,
        )
        .await
        .unwrap();

    vault_program_client
        .slash(
            &vault_config_pubkey,
            &vault_pubkey,
            &avs_pubkey,
            &operator_pubkey,
            &slasher,
            &avs_operator_ticket_pubkey,
            &operator_avs_ticket_pubkey,
            &avs_vault_ticket_pubkey,
            &operator_vault_ticket_pubkey,
            &vault_avs_ticket_pubkey,
            &vault_operator_ticket,
            &avs_slasher_ticket_pubkey,
            &vault_slasher_ticket_pubkey,
            &vault_delegate_list_pubkey,
            &vault_avs_slasher_operator_ticket,
            &vault_token_account,
            &slasher_token_account,
            100,
        )
        .await
        .unwrap();

    let vault = vault_program_client.get_vault(&vault_pubkey).await.unwrap();
    assert_eq!(vault.tokens_deposited(), 99_900);

    let delegation_list = vault_program_client
        .get_vault_delegation_list(&vault_delegate_list_pubkey)
        .await
        .unwrap();
    let delegations = delegation_list.delegations();
    assert_eq!(delegations.len(), 1);
    assert_eq!(delegations[0].operator(), operator_pubkey);
    assert_eq!(delegations[0].active_amount(), 9_900);

    let vault_avs_slasher_operator_ticket = vault_program_client
        .get_vault_avs_slasher_operator_ticket(
            &vault_pubkey,
            &avs_pubkey,
            &slasher.pubkey(),
            &operator_pubkey,
            current_epoch,
        )
        .await
        .unwrap();
    assert_eq!(vault_avs_slasher_operator_ticket.slashed(), 100);
    assert_eq!(vault_avs_slasher_operator_ticket.epoch(), current_epoch);
    assert_eq!(vault_avs_slasher_operator_ticket.vault(), vault_pubkey);
    assert_eq!(vault_avs_slasher_operator_ticket.avs(), avs_pubkey);
    assert_eq!(
        vault_avs_slasher_operator_ticket.slasher(),
        slasher.pubkey()
    );
    assert_eq!(
        vault_avs_slasher_operator_ticket.operator(),
        operator_pubkey
    );
}
