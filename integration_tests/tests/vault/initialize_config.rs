use jito_vault_core::config::Config;
use solana_sdk::signature::{Keypair, Signer};

use crate::fixtures::fixture::TestBuilder;

#[tokio::test]
async fn test_initialize_config_ok() {
    let mut fixture = TestBuilder::new().await;
    let mut vault_program_client = fixture.vault_program_client();

    let config_pubkey = Config::find_program_address(&jito_vault_program::id()).0;
    let config_admin = Keypair::new();

    fixture.transfer(&config_admin.pubkey(), 1.0).await.unwrap();

    vault_program_client
        .initialize_config(&config_pubkey, &config_admin)
        .await
        .unwrap();

    let config = vault_program_client
        .get_config(&config_pubkey)
        .await
        .unwrap();

    assert_eq!(config.admin(), config_admin.pubkey());
    assert_eq!(config.restaking_program(), jito_restaking_program::id());
    assert_eq!(config.epoch_length(), 864_000);
    assert_eq!(config.vaults_count(), 0);
}
