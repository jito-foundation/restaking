use solana_sdk::signature::{Keypair, Signer};

use crate::fixtures::fixture::TestBuilder;

#[tokio::test]
async fn test_add_slasher_ok() {
    let fixture = TestBuilder::new().await;

    let mut restaking_program_client = fixture.restaking_program_client();
    let mut vault_program_client = fixture.vault_program_client();

    let (_config_admin, vault_root) = vault_program_client.setup_vault(99, 100).await.unwrap();

    let _restaking_config_admin = restaking_program_client.setup_config().await.unwrap();

    let avs_root = restaking_program_client.setup_avs().await.unwrap();

    restaking_program_client
        .avs_vault_opt_in(&avs_root, &vault_root.vault_pubkey)
        .await
        .unwrap();

    vault_program_client
        .vault_avs_opt_in(&vault_root, &avs_root.avs_pubkey)
        .await
        .unwrap();

    let slasher = Keypair::new();
    restaking_program_client
        .avs_vault_slasher_opt_in(&avs_root, &vault_root.vault_pubkey, &slasher.pubkey(), 100)
        .await
        .unwrap();

    vault_program_client
        .vault_avs_vault_slasher_opt_in(&vault_root, &avs_root.avs_pubkey, &slasher.pubkey())
        .await
        .unwrap();

    let vault_avs_slasher = vault_program_client
        .get_vault_avs_slasher_ticket(
            &vault_root.vault_pubkey,
            &avs_root.avs_pubkey,
            &slasher.pubkey(),
        )
        .await
        .unwrap();
    assert_eq!(vault_avs_slasher.vault(), vault_root.vault_pubkey);
    assert_eq!(vault_avs_slasher.avs(), avs_root.avs_pubkey);
    assert_eq!(vault_avs_slasher.slasher(), slasher.pubkey());
    assert_eq!(vault_avs_slasher.index(), 0);
    assert_eq!(vault_avs_slasher.max_slashable_per_epoch(), 100);
    assert_eq!(vault_avs_slasher.state().slot_added(), 1);
}
