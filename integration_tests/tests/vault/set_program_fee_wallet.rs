#[cfg(test)]
mod tests {
    use jito_vault_core::config::Config;
    use jito_vault_sdk::error::VaultError;
    use solana_sdk::{signature::Keypair, signer::Signer};

    use crate::fixtures::{fixture::TestBuilder, vault_client::assert_vault_error, TestError};

    #[tokio::test]
    async fn test_set_config_fee_wallet() -> Result<(), TestError> {
        let context = TestBuilder::new().await;
        let mut vault_program_client = context.vault_program_client();

        // Initialize config and vault
        let (config_admin, _) = vault_program_client
            .setup_config_and_vault(0, 0, 0, 0)
            .await?;

        // Set a new fee wallet
        let new_fee_wallet = Keypair::new().pubkey();
        vault_program_client
            .set_program_fee_wallet(&config_admin, &new_fee_wallet)
            .await?;

        // Check if the fee wallet was updated
        let updated_config = vault_program_client
            .get_config(&Config::find_program_address(&jito_vault_program::id()).0)
            .await?;
        assert_eq!(updated_config.program_fee_wallet, new_fee_wallet);

        // Try to set fee wallet with non-admin account
        let non_admin = Keypair::new();
        vault_program_client
            .airdrop(&non_admin.pubkey(), 1.0)
            .await?;
        let result = vault_program_client
            .set_program_fee_wallet(&non_admin, &Keypair::new().pubkey())
            .await;
        assert_vault_error(result, VaultError::VaultConfigFeeAdminInvalid);

        // Try to set fee wallet to the same address (should succeed)
        vault_program_client
            .set_program_fee_wallet(&config_admin, &new_fee_wallet)
            .await?;

        Ok(())
    }
}
