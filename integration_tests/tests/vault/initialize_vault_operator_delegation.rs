#[cfg(test)]
mod tests {
    use jito_vault_core::config::Config;

    use crate::fixtures::fixture::TestBuilder;

    #[tokio::test]
    async fn test_add_operator_ok() {
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

        let operator_root = restaking_program_client
            .do_initialize_operator()
            .await
            .unwrap();

        restaking_program_client
            .do_initialize_operator_vault_ticket(&operator_root, &vault_root.vault_pubkey)
            .await
            .unwrap();
        vault_program_client
            .do_initialize_vault_operator_delegation(&vault_root, &operator_root.operator_pubkey)
            .await
            .unwrap();

        let vault_operator_delegation = vault_program_client
            .get_vault_operator_delegation(&vault_root.vault_pubkey, &operator_root.operator_pubkey)
            .await
            .unwrap();
        assert_eq!(vault_operator_delegation.vault, vault_root.vault_pubkey);
        assert_eq!(
            vault_operator_delegation.operator,
            operator_root.operator_pubkey
        );
        assert_eq!(vault_operator_delegation.index, 0);
        let config = vault_program_client
            .get_config(&Config::find_program_address(&jito_vault_program::id()).0)
            .await
            .unwrap();
        let slot = fixture.get_current_slot().await.unwrap();
    }
}
