use solana_program::pubkey::Pubkey;
use solana_sdk::signature::{Keypair, Signer};

use crate::fixtures::{fixture::TestBuilder, lrt_test_config::LrtTestConfig};

pub mod fixtures;

#[tokio::test]
async fn test_initialize_config_ok() {
    let mut fixture = TestBuilder::new().await;
    let mut collateral_program_client = fixture.lrt_program_client();

    let restaking_program_signer = Keypair::new();

    let test_config = LrtTestConfig::new_random(restaking_program_signer.pubkey());

    fixture
        .transfer(&test_config.config_admin.pubkey(), 10.0)
        .await
        .unwrap();

    collateral_program_client
        .initialize_config(&test_config)
        .await
        .unwrap();

    let config = collateral_program_client
        .get_config(&test_config.config)
        .await
        .unwrap();
    assert_eq!(config.admin(), test_config.config_admin.pubkey());
    assert_eq!(
        config.restaking_program_signer(),
        test_config.restaking_program_signer
    );
    assert_eq!(config.vaults_count(), 0);
}

#[tokio::test]
async fn test_initialize_config_bad_address_fails() {
    let mut fixture = TestBuilder::new().await;
    let mut collateral_program_client = fixture.lrt_program_client();

    let restaking_program_signer = Keypair::new();

    let mut test_config = LrtTestConfig::new_random(restaking_program_signer.pubkey());
    test_config.config = Pubkey::new_unique();

    fixture
        .transfer(&test_config.config_admin.pubkey(), 10.0)
        .await
        .unwrap();

    collateral_program_client
        .initialize_config(&test_config)
        .await
        .unwrap_err();
}

#[tokio::test]
async fn test_initialize_vault_ok() {
    let mut fixture = TestBuilder::new().await;
    let mut collateral_program_client = fixture.lrt_program_client();

    let restaking_program_signer = Keypair::new();

    let test_config = LrtTestConfig::new_random(restaking_program_signer.pubkey());

    fixture
        .transfer(&test_config.config_admin.pubkey(), 10.0)
        .await
        .unwrap();
    fixture
        .transfer(&test_config.vault_admin.pubkey(), 10.0)
        .await
        .unwrap();

    fixture
        .create_token_mint(&test_config.token_mint)
        .await
        .unwrap();

    collateral_program_client
        .initialize_config(&test_config)
        .await
        .unwrap();

    collateral_program_client
        .initialize_vault(&test_config)
        .await
        .unwrap();

    let config = collateral_program_client
        .get_config(&test_config.config)
        .await
        .unwrap();
    let vault = collateral_program_client
        .get_vault(&test_config.vault)
        .await
        .unwrap();
    let vault_avs_list = collateral_program_client
        .get_vault_avs_list(&test_config.vault_avs_list)
        .await
        .unwrap();
    let vault_operator_list = collateral_program_client
        .get_vault_operator_list(&test_config.vault_operator_list)
        .await
        .unwrap();

    assert_eq!(config.vaults_count(), 1);

    assert_eq!(vault.admin(), test_config.vault_admin.pubkey());
    assert_eq!(vault.base(), test_config.vault_base.pubkey());
    assert_eq!(vault.lrt_mint(), test_config.lrt_mint.pubkey());
    assert_eq!(vault.supported_mint(), test_config.token_mint.pubkey());
    assert_eq!(vault.capacity(), u64::MAX);
    assert_eq!(vault.lrt_index(), 0);

    assert_eq!(vault_avs_list.vault(), test_config.vault);
    assert_eq!(vault_avs_list.supported_avs().len(), 0);

    assert_eq!(vault_operator_list.vault(), test_config.vault);
    assert_eq!(vault_operator_list.operator_list().len(), 0);
}

#[tokio::test]
async fn test_initialize_vault_bad_addresses_fails() {
    let mut fixture = TestBuilder::new().await;
    let mut collateral_program_client = fixture.lrt_program_client();

    let restaking_program_signer = Keypair::new();

    let test_config = LrtTestConfig::new_random(restaking_program_signer.pubkey());

    fixture
        .transfer(&test_config.config_admin.pubkey(), 10.0)
        .await
        .unwrap();
    fixture
        .transfer(&test_config.vault_admin.pubkey(), 10.0)
        .await
        .unwrap();

    fixture
        .create_token_mint(&test_config.token_mint)
        .await
        .unwrap();

    collateral_program_client
        .initialize_config(&test_config)
        .await
        .unwrap();

    let mut bad_vault_base = test_config.clone();
    bad_vault_base.vault_base = Keypair::new();

    collateral_program_client
        .initialize_vault(&bad_vault_base)
        .await
        .unwrap_err();

    let mut bad_vault = test_config.clone();
    bad_vault.vault = Pubkey::new_unique();
    collateral_program_client
        .initialize_vault(&bad_vault)
        .await
        .unwrap_err();

    let mut bad_vault_avs_list = test_config.clone();
    bad_vault_avs_list.vault = Pubkey::new_unique();
    collateral_program_client
        .initialize_vault(&bad_vault_avs_list)
        .await
        .unwrap_err();

    let mut bad_vault_operator_list = test_config.clone();
    bad_vault_operator_list.vault_operator_list = Pubkey::new_unique();
    collateral_program_client
        .initialize_vault(&bad_vault_operator_list)
        .await
        .unwrap_err();
}
