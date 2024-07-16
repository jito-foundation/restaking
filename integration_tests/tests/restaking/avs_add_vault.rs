use jito_restaking_core::{avs::Avs, avs_vault_ticket::AvsVaultTicket, config::Config};
use jito_vault_core::vault::Vault;
use solana_program::pubkey::Pubkey;
use solana_sdk::signature::{Keypair, Signer};

use crate::fixtures::fixture::TestBuilder;

#[tokio::test]
async fn test_avs_add_vault_happy_path() {
    let mut fixture = TestBuilder::new().await;
    let mut restaking_program_client = fixture.restaking_program_client();

    // Initialize config
    let config_admin = Keypair::new();
    let config = Config::find_program_address(&jito_restaking_program::id()).0;
    fixture
        .transfer(&config_admin.pubkey(), 10.0)
        .await
        .unwrap();
    restaking_program_client
        .initialize_config(&config, &config_admin)
        .await
        .unwrap();

    // Initialize AVS
    let avs_admin = Keypair::new();
    let avs_base = Keypair::new();
    fixture.transfer(&avs_admin.pubkey(), 10.0).await.unwrap();
    let avs_pubkey = Avs::find_program_address(&jito_restaking_program::id(), &avs_base.pubkey()).0;
    restaking_program_client
        .initialize_avs(&config, &avs_pubkey, &avs_admin, &avs_base)
        .await
        .unwrap();

    let vault_pubkey =
        Vault::find_program_address(&jito_restaking_program::id(), &Pubkey::new_unique()).0;

    // AVS adds vault
    let avs_vault_ticket = AvsVaultTicket::find_program_address(
        &jito_restaking_program::id(),
        &avs_pubkey,
        &vault_pubkey,
    )
    .0;
    restaking_program_client
        .avs_add_vault(
            &config,
            &avs_pubkey,
            &vault_pubkey,
            &avs_vault_ticket,
            &avs_admin,
            &avs_admin,
        )
        .await
        .unwrap();

    // Verify AVS state
    let avs = restaking_program_client.get_avs(&avs_pubkey).await.unwrap();
    assert_eq!(avs.vault_count(), 1);

    // Verify AVS vault ticket
    let ticket = restaking_program_client
        .get_avs_vault_ticket(&avs_pubkey, &vault_pubkey)
        .await
        .unwrap();
    assert_eq!(ticket.avs(), avs_pubkey);
    assert_eq!(ticket.vault(), vault_pubkey);
    assert_eq!(ticket.index(), 0);
    assert_eq!(ticket.state().slot_added(), 1);
}
