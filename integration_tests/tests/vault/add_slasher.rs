use jito_restaking_core::{
    avs::Avs, avs_vault_slasher_ticket::AvsVaultSlasherTicket, avs_vault_ticket::AvsVaultTicket,
    config::Config as RestakingConfig,
};
use jito_vault_core::{
    config::Config as VaultConfig, vault::Vault, vault_avs_ticket::VaultAvsTicket,
    vault_delegation_list::VaultDelegationList, vault_slasher_ticket::VaultAvsSlasherTicket,
};
use solana_sdk::signature::{Keypair, Signer};

use crate::fixtures::fixture::TestBuilder;

#[tokio::test]
async fn test_add_slasher_ok() {
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

    // Initialize AVS
    let avs_admin = Keypair::new();
    let avs_base = Keypair::new();
    fixture.transfer(&avs_admin.pubkey(), 10.0).await.unwrap();
    let avs_pubkey = Avs::find_program_address(&jito_restaking_program::id(), &avs_base.pubkey()).0;
    restaking_program_client
        .initialize_avs(&restaking_config, &avs_pubkey, &avs_admin, &avs_base)
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

    // AVS adds slasher
    let slasher = Keypair::new();
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

    let vault_avs_slasher = vault_program_client
        .get_vault_avs_slasher_ticket(&vault_pubkey, &avs_pubkey, &slasher.pubkey())
        .await
        .unwrap();
    assert_eq!(vault_avs_slasher.vault(), vault_pubkey);
    assert_eq!(vault_avs_slasher.avs(), avs_pubkey);
    assert_eq!(vault_avs_slasher.slasher(), slasher.pubkey());
    assert_eq!(vault_avs_slasher.index(), 0);
    assert_eq!(vault_avs_slasher.max_slashable_per_epoch(), 100);
    assert_eq!(vault_avs_slasher.state().slot_added(), 1);
}
