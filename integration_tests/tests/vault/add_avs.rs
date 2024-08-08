use jito_restaking_core::{
    avs::Avs, avs_vault_ticket::AvsVaultTicket, config::Config as RestakingConfig,
};
use jito_vault_core::{
    config::Config as VaultConfig, vault::Vault, vault_avs_ticket::VaultAvsTicket,
    vault_delegation_list::VaultDelegationList,
};
use solana_sdk::signature::{Keypair, Signer};

use crate::fixtures::fixture::TestBuilder;

#[tokio::test]
async fn test_add_avs_ok() {
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

    // create AVS + add AVS vault
    let avs_admin = Keypair::new();
    let avs_base = Keypair::new();
    fixture.transfer(&avs_admin.pubkey(), 1.0).await.unwrap();
    let avs_pubkey = Avs::find_program_address(&jito_restaking_program::id(), &avs_base.pubkey()).0;
    restaking_program_client
        .initialize_avs(&restaking_config_pubkey, &avs_pubkey, &avs_admin, &avs_base)
        .await
        .unwrap();
    let avs_vault_ticket = AvsVaultTicket::find_program_address(
        &jito_restaking_program::id(),
        &avs_pubkey,
        &vault_pubkey,
    )
    .0;
    restaking_program_client
        .avs_add_vault(
            &restaking_config_pubkey,
            &avs_pubkey,
            &vault_pubkey,
            &avs_vault_ticket,
            &avs_admin,
            &avs_admin,
        )
        .await
        .unwrap();

    fixture
        .warp_slot_incremental(config_account.epoch_length() * 2)
        .await
        .unwrap();

    let vault_avs_ticket =
        VaultAvsTicket::find_program_address(&jito_vault_program::id(), &vault_pubkey, &avs_pubkey)
            .0;
    vault_program_client
        .add_avs(
            &vault_config_pubkey,
            &vault_pubkey,
            &avs_pubkey,
            &avs_vault_ticket,
            &vault_avs_ticket,
            &vault_admin,
            &vault_admin,
        )
        .await
        .unwrap();

    let vault_avs_ticket_account = vault_program_client
        .get_vault_avs_ticket(&vault_pubkey, &avs_pubkey)
        .await
        .unwrap();
    assert_eq!(vault_avs_ticket_account.vault(), vault_pubkey);
    assert_eq!(vault_avs_ticket_account.avs(), avs_pubkey);
    assert_eq!(vault_avs_ticket_account.index(), 0);
    assert_eq!(
        vault_avs_ticket_account.state().slot_added(),
        fixture.get_current_slot().await.unwrap()
    );
}
