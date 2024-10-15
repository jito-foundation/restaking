#[cfg(test)]
mod tests {
    use jito_vault_core::config::Config;
    use solana_program::clock::DEFAULT_SLOTS_PER_EPOCH;
    use solana_sdk::signature::Signer;

    use crate::fixtures::fixture::TestBuilder;

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

        assert_ne!(config.epoch_length(), 0);
    }
}
