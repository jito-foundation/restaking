#[cfg(test)]
mod tests {

    use jito_vault_sdk::error::VaultError;

    use crate::fixtures::{fixture::TestBuilder, vault_client::assert_vault_error};

    const DEPOSIT_FEE_BPS: u16 = 99;
    const WITHDRAW_FEE_BPS: u16 = 100;
    const ZERO_REWARD_FEE_BPS: u16 = 0;

    #[tokio::test]
    async fn test_add_operator_ok() {
        let fixture = TestBuilder::new().await;

        let mut restaking_program_client = fixture.restaking_program_client();
        let mut vault_program_client = fixture.vault_program_client();

        let (_config_admin, vault_root) = vault_program_client
            .setup_config_and_vault(DEPOSIT_FEE_BPS, WITHDRAW_FEE_BPS, ZERO_REWARD_FEE_BPS)
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
        assert_eq!(vault_operator_delegation.index(), 0);
    }

    #[tokio::test]
    async fn test_add_operator_vault_is_paused_ok() {
        let fixture = TestBuilder::new().await;

        let mut restaking_program_client = fixture.restaking_program_client();
        let mut vault_program_client = fixture.vault_program_client();

        let (_config_admin, vault_root) = vault_program_client
            .setup_config_and_vault(DEPOSIT_FEE_BPS, WITHDRAW_FEE_BPS, ZERO_REWARD_FEE_BPS)
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
            .set_is_paused(&vault_root.vault_pubkey, &vault_root.vault_admin, true)
            .await
            .unwrap();

        let test_error = vault_program_client
            .do_initialize_vault_operator_delegation(&vault_root, &operator_root.operator_pubkey)
            .await;

        assert_vault_error(test_error, VaultError::VaultIsPaused);
    }
}
