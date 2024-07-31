use crate::fixtures::fixture::TestBuilder;

#[tokio::test]
async fn test_add_avs_ok() {
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

    let vault_avs_ticket_account = vault_program_client
        .get_vault_avs_ticket(&vault_root.vault_pubkey, &avs_root.avs_pubkey)
        .await
        .unwrap();
    assert_eq!(vault_avs_ticket_account.vault(), vault_root.vault_pubkey);
    assert_eq!(vault_avs_ticket_account.avs(), avs_root.avs_pubkey);
    assert_eq!(vault_avs_ticket_account.index(), 0);
    assert_eq!(vault_avs_ticket_account.state().slot_added(), 1);
}
