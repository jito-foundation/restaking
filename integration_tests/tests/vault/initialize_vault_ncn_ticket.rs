#[cfg(test)]
mod tests {
    use jito_restaking_core::config::Config;

    use crate::fixtures::fixture::TestBuilder;

    #[tokio::test]
    async fn test_add_ncn_ok() {
        let mut fixture = TestBuilder::new().await;

        let mut restaking_program_client = fixture.restaking_program_client();
        let mut vault_program_client = fixture.vault_program_client();

        let (_config_admin, vault_root) = vault_program_client
            .setup_config_and_vault(99, 100)
            .await
            .unwrap();

        let _restaking_config_admin = restaking_program_client
            .do_initialize_config()
            .await
            .unwrap();

        let ncn_root = restaking_program_client.do_initialize_ncn().await.unwrap();

        restaking_program_client
            .do_initialize_ncn_vault_ticket(&ncn_root, &vault_root.vault_pubkey)
            .await
            .unwrap();

        let config_account = restaking_program_client
            .get_config(&Config::find_program_address(&jito_restaking_program::id()).0)
            .await
            .unwrap();
        fixture
            .warp_slot_incremental(2 * config_account.epoch_length)
            .await
            .unwrap();

        vault_program_client
            .do_initialize_vault_ncn_ticket(&vault_root, &ncn_root.ncn_pubkey)
            .await
            .unwrap();

        let vault_ncn_ticket_account = vault_program_client
            .get_vault_ncn_ticket(&vault_root.vault_pubkey, &ncn_root.ncn_pubkey)
            .await
            .unwrap();
        assert_eq!(vault_ncn_ticket_account.vault, vault_root.vault_pubkey);
        assert_eq!(vault_ncn_ticket_account.ncn, ncn_root.ncn_pubkey);
        assert_eq!(vault_ncn_ticket_account.index, 0);
        assert_eq!(
            vault_ncn_ticket_account.state.slot_added(),
            fixture.get_current_slot().await.unwrap()
        );
    }
}
