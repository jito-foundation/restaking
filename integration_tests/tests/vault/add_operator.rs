#[cfg(test)]
mod tests {
    use crate::fixtures::fixture::TestBuilder;

    #[tokio::test]
    async fn test_add_operator_ok() {
        let fixture = TestBuilder::new().await;

        let mut restaking_program_client = fixture.restaking_program_client();
        let mut vault_program_client = fixture.vault_program_client();

        let (_config_admin, vault_root) = vault_program_client.setup_vault(99, 100).await.unwrap();

        let _restaking_config_admin = restaking_program_client.setup_config().await.unwrap();

        let operator_root = restaking_program_client.setup_operator().await.unwrap();

        restaking_program_client
            .operator_vault_opt_in(&operator_root, &vault_root.vault_pubkey)
            .await
            .unwrap();

        vault_program_client
            .vault_operator_opt_in(&vault_root, &operator_root.operator_pubkey)
            .await
            .unwrap();

        let vault_operator_ticket = vault_program_client
            .get_vault_operator_ticket(&vault_root.vault_pubkey, &operator_root.operator_pubkey)
            .await
            .unwrap();
        assert_eq!(vault_operator_ticket.vault(), vault_root.vault_pubkey);
        assert_eq!(
            vault_operator_ticket.operator(),
            operator_root.operator_pubkey
        );
        assert_eq!(vault_operator_ticket.index(), 0);
        assert_eq!(vault_operator_ticket.state().slot_added(), 1);
    }
}
