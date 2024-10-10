#[cfg(test)]
mod tests {
    use jito_vault_core::{config::Config, MAX_FEE_BPS};
    use solana_program::{clock::DEFAULT_SLOTS_PER_EPOCH, instruction::InstructionError};
    use solana_sdk::signature::{Keypair, Signer};

    use crate::fixtures::{
        assert_ix_error, fixture::TestBuilder,
    };

    #[tokio::test]
    async fn test_initialize_config_ok() {
        let fixture = TestBuilder::new().await;
        let mut vault_program_client = fixture.vault_program_client();

        let config_admin = vault_program_client.do_initialize_config().await.unwrap();

        let config = vault_program_client
            .get_config(&Config::find_program_address(&jito_vault_program::id()).0)
            .await
            .unwrap();

        assert_eq!(config.admin, config_admin.pubkey());
        assert_eq!(config.restaking_program, jito_restaking_program::id());
        assert_eq!(config.epoch_length(), DEFAULT_SLOTS_PER_EPOCH);
        assert_eq!(config.num_vaults(), 0);
    }

    #[tokio::test]
    async fn test_initialize_config_fee_above_max() {
        let fixture = TestBuilder::new().await;
        let mut vault_program_client = fixture.vault_program_client();

        // Airdrop to the config admin
        let config_admin = Keypair::new();
        vault_program_client
            .airdrop(&config_admin.pubkey(), 1.0)
            .await
            .unwrap();

        // Attempt to initialize config with fee above MAX_FEE_BPS
        let result = vault_program_client
            .initialize_config(
                &Config::find_program_address(&jito_vault_program::id()).0,
                &config_admin,
                &config_admin.pubkey(),
                MAX_FEE_BPS + 1,
            )
            .await;

        assert!(result.is_err());
        assert_ix_error(result, InstructionError::InvalidArgument);
    }
}
