#[cfg(test)]
mod tests {
    use jito_vault_core::config::Config;
    use jito_vault_sdk::error::VaultError;
    use solana_sdk::signature::{Keypair, Signer};

    use crate::fixtures::{
        fixture::TestBuilder,
        vault_client::{assert_vault_error, VaultRoot},
    };

    #[tokio::test]
    async fn test_set_admin_with_bad_admin() {
        let fixture = TestBuilder::new().await;

        let mut vault_program_client = fixture.vault_program_client();

        let deposit_fee_bps = 99;
        let withdrawal_fee_bps = 100;

        let (
            _config_admin,
            VaultRoot {
                vault_pubkey,
                vault_admin,
            },
        ) = vault_program_client
            .setup_config_and_vault(deposit_fee_bps, withdrawal_fee_bps)
            .await
            .unwrap();

        let config_pubkey = Config::find_program_address(&jito_vault_program::id()).0;
        let vault = vault_program_client.get_vault(&vault_pubkey).await.unwrap();

        assert_eq!(vault.admin, vault_admin.pubkey());

        let bad_admin = Keypair::new();
        vault_program_client
            ._airdrop(&bad_admin.pubkey(), 10.0)
            .await
            .unwrap();

        let new_admin = Keypair::new();
        let response = vault_program_client
            .set_admin(&config_pubkey, &vault_pubkey, &bad_admin, &new_admin)
            .await;

        assert_vault_error(response, VaultError::VaultAdminInvalid);
    }

    #[tokio::test]
    async fn test_set_admin() {
        let fixture = TestBuilder::new().await;

        let mut vault_program_client = fixture.vault_program_client();

        let deposit_fee_bps = 99;
        let withdrawal_fee_bps = 100;

        let (
            _config_admin,
            VaultRoot {
                vault_pubkey,
                vault_admin,
            },
        ) = vault_program_client
            .setup_config_and_vault(deposit_fee_bps, withdrawal_fee_bps)
            .await
            .unwrap();

        let config_pubkey = Config::find_program_address(&jito_vault_program::id()).0;
        let vault = vault_program_client.get_vault(&vault_pubkey).await.unwrap();

        assert_eq!(vault.admin, vault_admin.pubkey());

        let new_admin = Keypair::new();
        vault_program_client
            .set_admin(&config_pubkey, &vault_pubkey, &vault_admin, &new_admin)
            .await
            .unwrap();

        let vault = vault_program_client.get_vault(&vault_pubkey).await.unwrap();

        assert_eq!(vault.admin, new_admin.pubkey());
    }
}
