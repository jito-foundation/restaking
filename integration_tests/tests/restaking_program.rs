use jito_restaking_core::config::DEFAULT_RESTAKING_EPOCH_DURATION;
use solana_sdk::signature::Signer;

use crate::fixtures::{
    fixture::TestBuilder, restaking_test_config::RestakingTestConfig,
    vault_test_config::VaultTestConfig,
};

pub mod fixtures;

#[tokio::test]
async fn test_initialize_config_ok() {
    let mut fixture = TestBuilder::new().await;

    let mut restaking_program_client = fixture.restaking_program_client();

    let restaking_test_config = RestakingTestConfig::new_random();

    fixture
        .transfer(&restaking_test_config.config_admin.pubkey(), 10.0)
        .await
        .unwrap();

    restaking_program_client
        .initialize_config(&restaking_test_config)
        .await
        .unwrap();

    let config = restaking_program_client
        .get_config(&restaking_test_config.config)
        .await
        .unwrap();
    assert_eq!(config.admin(), restaking_test_config.config_admin.pubkey());
    assert_eq!(config.vault_program(), jito_vault_program::id());
    assert_eq!(config.avs_count(), 0);
    assert_eq!(config.operators_count(), 0);
    assert_eq!(config.epoch_duration(), DEFAULT_RESTAKING_EPOCH_DURATION);
}

#[tokio::test]
async fn test_initialize_avs_ok() {
    let mut fixture = TestBuilder::new().await;

    let mut restaking_program_client = fixture.restaking_program_client();

    let restaking_test_config = RestakingTestConfig::new_random();

    fixture
        .transfer(&restaking_test_config.config_admin.pubkey(), 10.0)
        .await
        .unwrap();
    fixture
        .transfer(&restaking_test_config.avs_admin.pubkey(), 10.0)
        .await
        .unwrap();

    restaking_program_client
        .initialize_config(&restaking_test_config)
        .await
        .unwrap();

    restaking_program_client
        .initialize_avs(&restaking_test_config)
        .await
        .unwrap();

    let config = restaking_program_client
        .get_config(&restaking_test_config.config)
        .await
        .unwrap();

    let avs = restaking_program_client
        .get_avs(&restaking_test_config.avs)
        .await
        .unwrap();

    let avs_operator_list = restaking_program_client
        .get_avs_operator_list(&restaking_test_config.avs_operator_list)
        .await
        .unwrap();

    let avs_vault_list = restaking_program_client
        .get_avs_vault_list(&restaking_test_config.avs_vault_list)
        .await
        .unwrap();

    assert_eq!(config.avs_count(), 1);

    assert_eq!(avs.admin(), restaking_test_config.avs_admin.pubkey());
    assert_eq!(avs.base(), restaking_test_config.avs_base.pubkey());
    assert_eq!(avs.avs_index(), 0);

    assert_eq!(avs_operator_list.avs(), restaking_test_config.avs);
    assert_eq!(avs_operator_list.operators().len(), 0);

    assert_eq!(avs_vault_list.avs(), restaking_test_config.avs);
    assert_eq!(avs_vault_list.vault_list().len(), 0);
}

#[tokio::test]
async fn test_initialize_operator_ok() {
    let mut fixture = TestBuilder::new().await;

    let mut restaking_program_client = fixture.restaking_program_client();

    let restaking_test_config = RestakingTestConfig::new_random();

    fixture
        .transfer(&restaking_test_config.config_admin.pubkey(), 10.0)
        .await
        .unwrap();
    fixture
        .transfer(&restaking_test_config.operator_admin.pubkey(), 10.0)
        .await
        .unwrap();

    restaking_program_client
        .initialize_config(&restaking_test_config)
        .await
        .unwrap();

    restaking_program_client
        .initialize_operator(&restaking_test_config)
        .await
        .unwrap();

    let config = restaking_program_client
        .get_config(&restaking_test_config.config)
        .await
        .unwrap();

    let operator = restaking_program_client
        .get_operator(&restaking_test_config.operator)
        .await
        .unwrap();
    let operator_vault_list = restaking_program_client
        .get_operator_vault_list(&restaking_test_config.operator_vault_list)
        .await
        .unwrap();
    let operator_avs_list = restaking_program_client
        .get_operator_avs_list(&restaking_test_config.operator_avs_list)
        .await
        .unwrap();

    assert_eq!(config.operators_count(), 1);

    assert_eq!(
        operator.admin(),
        restaking_test_config.operator_admin.pubkey()
    );
    assert_eq!(
        operator.base(),
        restaking_test_config.operator_base.pubkey()
    );
    assert_eq!(operator.index(), 0);

    assert_eq!(
        operator_vault_list.operator(),
        restaking_test_config.operator
    );
    assert_eq!(operator_vault_list.vault_list().len(), 0);

    assert_eq!(operator_avs_list.operator(), restaking_test_config.operator);
    assert_eq!(operator_avs_list.avs_list().len(), 0);
}

#[tokio::test]
async fn test_operator_add_vault_ok() {
    let mut fixture = TestBuilder::new().await;

    let mut restaking_program_client = fixture.restaking_program_client();
    let restaking_test_config = RestakingTestConfig::new_random();

    let mut vault_program_client = fixture.vault_program_client();
    let vault_test_config = VaultTestConfig::new_random();

    fixture
        .create_token_mint(&vault_test_config.token_mint)
        .await
        .unwrap();

    fixture
        .transfer(&restaking_test_config.config_admin.pubkey(), 10.0)
        .await
        .unwrap();
    fixture
        .transfer(&restaking_test_config.operator_admin.pubkey(), 10.0)
        .await
        .unwrap();

    fixture
        .transfer(&vault_test_config.config_admin.pubkey(), 10.0)
        .await
        .unwrap();
    fixture
        .transfer(&vault_test_config.vault_admin.pubkey(), 10.0)
        .await
        .unwrap();

    restaking_program_client
        .initialize_config(&restaking_test_config)
        .await
        .unwrap();

    restaking_program_client
        .initialize_operator(&restaking_test_config)
        .await
        .unwrap();

    vault_program_client
        .initialize_config(&vault_test_config)
        .await
        .unwrap();

    vault_program_client
        .initialize_vault(&vault_test_config)
        .await
        .unwrap();

    restaking_program_client
        .operator_add_vault(&restaking_test_config, &vault_test_config)
        .await
        .unwrap();
}
