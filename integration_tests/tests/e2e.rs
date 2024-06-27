use solana_sdk::signature::Signer;

use crate::fixtures::{
    fixture::TestBuilder, restaking_test_config::RestakingTestConfig,
    vault_test_config::VaultTestConfig,
};

pub mod fixtures;

#[tokio::test]
async fn test_vault_add_and_remove_avs() {
    let mut fixture = TestBuilder::new().await;

    let mut restaking_program_client = fixture.restaking_program_client();
    let mut vault_program_client = fixture.vault_program_client();

    let restaking_test_config = RestakingTestConfig::new_random();
    let vault_test_config = VaultTestConfig::new_random(restaking_test_config.config);

    fixture
        .create_token_mint(&vault_test_config.token_mint)
        .await
        .unwrap();

    fixture
        .transfer(&restaking_test_config.config_admin.pubkey(), 10.0)
        .await
        .unwrap();
    fixture
        .transfer(&restaking_test_config.avs_admin.pubkey(), 10.0)
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

    vault_program_client
        .initialize_config(&vault_test_config)
        .await
        .unwrap();

    vault_program_client
        .initialize_vault(&vault_test_config)
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

    restaking_program_client
        .avs_add_vault(&restaking_test_config, &vault_test_config)
        .await
        .unwrap();

    let vault_avs_list = vault_program_client
        .get_vault_avs_list(&vault_test_config.vault_avs_list)
        .await
        .unwrap();
    assert_eq!(
        *vault_avs_list.supported_avs()[0].avs(),
        restaking_test_config.avs
    );

    fixture.warp_to_next_slot().await.unwrap();

    restaking_program_client
        .avs_remove_vault(&restaking_test_config, &vault_test_config)
        .await
        .unwrap();

    let vault_avs_list = vault_program_client
        .get_vault_avs_list(&vault_test_config.vault_avs_list)
        .await
        .unwrap();

    let avs = vault_avs_list.supported_avs().get(0).unwrap();
    assert_eq!(*avs.avs(), restaking_test_config.avs);
    assert!(avs.state().slot_removed() > avs.state().slot_added());
}

#[tokio::test]
async fn test_vault_add_and_remove_operator() {
    let mut fixture = TestBuilder::new().await;

    let mut restaking_program_client = fixture.restaking_program_client();
    let mut vault_program_client = fixture.vault_program_client();

    let restaking_test_config = RestakingTestConfig::new_random();
    let vault_test_config = VaultTestConfig::new_random(restaking_test_config.config);

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

    vault_program_client
        .initialize_config(&vault_test_config)
        .await
        .unwrap();

    vault_program_client
        .initialize_vault(&vault_test_config)
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

    restaking_program_client
        .operator_add_vault(&restaking_test_config, &vault_test_config)
        .await
        .unwrap();

    let vault_operator_list = vault_program_client
        .get_vault_operator_list(&vault_test_config.vault_operator_list)
        .await
        .unwrap();

    let operator_vault_list = restaking_program_client
        .get_operator_vault_list(&restaking_test_config.operator_vault_list)
        .await
        .unwrap();

    let operator = vault_operator_list.operator_list().get(0).unwrap();
    assert_eq!(operator.operator(), restaking_test_config.operator);
    assert_eq!(operator.state().slot_added(), 1);
    assert_eq!(operator.state().slot_removed(), 0);
    assert_eq!(operator.active_amount(), 0);
    assert_eq!(operator.cooling_down_amount(), 0);

    let vault = operator_vault_list.vault_list().get(0).unwrap();
    assert_eq!(vault.vault(), vault_test_config.vault);
    assert_eq!(vault.state().slot_added(), 1);
    assert_eq!(vault.state().slot_removed(), 0);
}
