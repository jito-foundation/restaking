use crate::fixtures::{fixture::TestBuilder, restaking_client::AvsRoot};
use jito_restaking_core::{avs::Avs, config::Config};
use log::info;
use solana_program::clock::Clock;
use solana_program::sysvar::SysvarId;
use solana_sdk::signature::{Keypair, Signer};
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
async fn test_add_avs_ok() {
    let fixture = TestBuilder::new().await;

    let mut restaking_program_client = fixture.restaking_program_client();
    let mut fixture = TestBuilder::new().await;

    let mut vault_program_client = fixture.vault_program_client();

    let (_config_admin, vault_root) = vault_program_client.setup_vault(99, 100).await.unwrap();

    let _restaking_config_admin = restaking_program_client.setup_config().await.unwrap();

    // let avs_root = restaking_program_client.setup_avs().await.unwrap();

    // create AVS + add AVS vault
    let avs_admin = Keypair::new();
    let avs_base = Keypair::new();
    fixture.transfer(&avs_admin.pubkey(), 1.0).await.unwrap();
    sleep(Duration::from_secs(1)).await;
    let avs_pubkey = Avs::find_program_address(&jito_restaking_program::id(), &avs_base.pubkey()).0;

    restaking_program_client
        .initialize_avs(
            &Config::find_program_address(&jito_restaking_program::id()).0,
            &avs_pubkey,
            &avs_admin,
            &avs_base,
        )
        .await
        .unwrap();

    // let avs_root = AvsRoot {
    //     avs_pubkey,
    //     avs_admin,
    // };
    //
    // restaking_program_client
    //     .avs_vault_opt_in(&avs_root, &vault_root.vault_pubkey)
    //     .await
    //     .unwrap();
    //
    // vault_program_client
    //     .vault_avs_opt_in(&vault_root, &avs_root.avs_pubkey)
    //     .await
    //     .unwrap();
    //
    // let vault_avs_ticket_account = vault_program_client
    //     .get_vault_avs_ticket(&vault_root.vault_pubkey, &avs_root.avs_pubkey)
    //     .await
    //     .unwrap();
    // assert_eq!(vault_avs_ticket_account.vault(), vault_root.vault_pubkey);
    // assert_eq!(vault_avs_ticket_account.avs(), avs_root.avs_pubkey);
    // assert_eq!(vault_avs_ticket_account.index(), 0);
    // assert_eq!(vault_avs_ticket_account.state().slot_added(), 1);
}
