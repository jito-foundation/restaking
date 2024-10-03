#[cfg(test)]
mod tests {
    use jito_jsm_core::slot_toggle::SlotToggleState;
    use jito_restaking_core::config::Config;

    use crate::fixtures::fixture::TestBuilder;

    #[tokio::test]
    async fn test_operator_add_vault_ok() {
        let mut fixture = TestBuilder::new().await;
        let mut restaking_program_client = fixture.restaking_program_client();

        let mut vault_program_client = fixture.vault_program_client();

        let token_program = spl_token::id();

        let (_vault_config_admin, vault_root) = vault_program_client
            .setup_config_and_vault(&token_program, 0, 0, 0)
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

        // Verify operator state
        let operator = restaking_program_client
            .get_operator(&operator_root.operator_pubkey)
            .await
            .unwrap();
        assert_eq!(operator.vault_count(), 1);

        // Verify operator vault ticket
        let ticket = restaking_program_client
            .get_operator_vault_ticket(&operator_root.operator_pubkey, &vault_root.vault_pubkey)
            .await
            .unwrap();
        assert_eq!(ticket.operator, operator_root.operator_pubkey);
        assert_eq!(ticket.vault, vault_root.vault_pubkey);
        assert_eq!(ticket.index(), 0);

        let config = restaking_program_client
            .get_config(&Config::find_program_address(&jito_restaking_program::id()).0)
            .await
            .unwrap();
        let slot = fixture.get_current_slot().await.unwrap();
        assert_eq!(
            ticket.state.state(slot, config.epoch_length()),
            SlotToggleState::Inactive
        );
    }
}
