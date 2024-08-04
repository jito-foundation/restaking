use solana_sdk::signature::{Keypair, Signer};
use spl_associated_token_account::get_associated_token_address;

use crate::fixtures::{fixture::TestBuilder, vault_client::VaultStakerWithdrawTicketRoot};

#[tokio::test]
async fn test_enqueue_withdraw_more_than_staked_fails() {
    let mut fixture = TestBuilder::new().await;

    let mut vault_program_client = fixture.vault_program_client();

    let (_vault_config_admin, vault_root) =
        vault_program_client.setup_vault(100, 100).await.unwrap();

    let vault = vault_program_client
        .get_vault(&vault_root.vault_pubkey)
        .await
        .unwrap();

    let depositor = Keypair::new();
    fixture.transfer(&depositor.pubkey(), 100.0).await.unwrap();
    fixture
        .mint_to(&vault.supported_mint(), &depositor.pubkey(), 100_000)
        .await
        .unwrap();

    fixture
        .create_ata(&vault.lrt_mint(), &depositor.pubkey())
        .await
        .unwrap();

    let depositor_lrt_token_account =
        get_associated_token_address(&depositor.pubkey(), &vault.lrt_mint());
    vault_program_client
        .mint_to(
            &vault_root.vault_pubkey,
            &vault.lrt_mint(),
            &depositor,
            &get_associated_token_address(&depositor.pubkey(), &vault.supported_mint()),
            &get_associated_token_address(&vault_root.vault_pubkey, &vault.supported_mint()),
            &get_associated_token_address(&depositor.pubkey(), &vault.lrt_mint()),
            &get_associated_token_address(&vault.fee_owner(), &vault.lrt_mint()),
            None,
            100_000,
        )
        .await
        .unwrap();

    let depositor_ata = fixture
        .get_token_account(&depositor_lrt_token_account)
        .await
        .unwrap();
    assert_eq!(depositor_ata.amount, 99_000);

    vault_program_client
        .do_enqueue_withdraw(&vault_root, &depositor, 49_500)
        .await
        .unwrap_err();
}

#[tokio::test]
async fn test_enqueue_withdraw_with_fee_success() {
    let mut fixture = TestBuilder::new().await;

    let mut vault_program_client = fixture.vault_program_client();
    let mut restaking_pool_client = fixture.restaking_program_client();

    let (_vault_config_admin, vault_root) =
        vault_program_client.setup_vault(100, 100).await.unwrap();

    let _restaking_config_admin = restaking_pool_client.setup_config().await.unwrap();

    let operator_root = restaking_pool_client.setup_operator().await.unwrap();
    let avs_root = restaking_pool_client.setup_avs().await.unwrap();

    restaking_pool_client
        .operator_avs_opt_in(&operator_root, &avs_root.avs_pubkey)
        .await
        .unwrap();
    restaking_pool_client
        .avs_operator_opt_in(&avs_root, &operator_root.operator_pubkey)
        .await
        .unwrap();

    restaking_pool_client
        .avs_vault_opt_in(&avs_root, &vault_root.vault_pubkey)
        .await
        .unwrap();
    restaking_pool_client
        .operator_vault_opt_in(&operator_root, &vault_root.vault_pubkey)
        .await
        .unwrap();

    vault_program_client
        .vault_avs_opt_in(&vault_root, &avs_root.avs_pubkey)
        .await
        .unwrap();
    vault_program_client
        .vault_operator_opt_in(&vault_root, &operator_root.operator_pubkey)
        .await
        .unwrap();

    let vault = vault_program_client
        .get_vault(&vault_root.vault_pubkey)
        .await
        .unwrap();

    let depositor = Keypair::new();
    fixture.transfer(&depositor.pubkey(), 100.0).await.unwrap();
    fixture
        .mint_to(&vault.supported_mint(), &depositor.pubkey(), 100_000)
        .await
        .unwrap();

    fixture
        .create_ata(&vault.lrt_mint(), &depositor.pubkey())
        .await
        .unwrap();

    vault_program_client
        .mint_to(
            &vault_root.vault_pubkey,
            &vault.lrt_mint(),
            &depositor,
            &get_associated_token_address(&depositor.pubkey(), &vault.supported_mint()),
            &get_associated_token_address(&vault_root.vault_pubkey, &vault.supported_mint()),
            &get_associated_token_address(&depositor.pubkey(), &vault.lrt_mint()),
            &get_associated_token_address(&vault.fee_owner(), &vault.lrt_mint()),
            None,
            100_000,
        )
        .await
        .unwrap();

    let vault_lrt_account = fixture
        .get_token_account(&get_associated_token_address(
            &depositor.pubkey(),
            &vault.lrt_mint(),
        ))
        .await
        .unwrap();
    assert_eq!(vault_lrt_account.amount, 99_000);

    let vault_fee_account = fixture
        .get_token_account(&get_associated_token_address(
            &vault.fee_owner(),
            &vault.lrt_mint(),
        ))
        .await
        .unwrap();
    assert_eq!(vault_fee_account.amount, 1_000);

    vault_program_client
        .delegate(&vault_root, &operator_root.operator_pubkey, 100_000)
        .await
        .unwrap();

    let vault_delegation_list = vault_program_client
        .get_vault_delegation_list(&vault_root.vault_pubkey)
        .await
        .unwrap();

    let delegation = vault_delegation_list.delegations().get(0).unwrap();
    assert_eq!(delegation.staked_amount(), 100_000);
    assert_eq!(delegation.total_security().unwrap(), 100_000);

    // the user is withdrawing 99,000 LRT tokens, there is a 1% fee on withdraws, so
    // 98010 tokens will be undeleged for withdraw
    let VaultStakerWithdrawTicketRoot { base } = vault_program_client
        .do_enqueue_withdraw(&vault_root, &depositor, 99_000)
        .await
        .unwrap();

    let vault_delegation_list = vault_program_client
        .get_vault_delegation_list(&vault_root.vault_pubkey)
        .await
        .unwrap();

    let delegation = vault_delegation_list.delegations().get(0).unwrap();
    // this is 1,000 because 1% of the fee went to the vault fee account, the assets still staked
    // are for the LRT in the fee account to unstake later
    assert_eq!(delegation.staked_amount(), 1_990);
    assert_eq!(delegation.enqueued_for_withdraw_amount(), 98_010);
    assert_eq!(delegation.total_security().unwrap(), 100_000);

    let vault_staker_withdraw_ticket = vault_program_client
        .get_vault_staker_withdraw_ticket(&vault_root.vault_pubkey, &depositor.pubkey(), &base)
        .await
        .unwrap();
    assert_eq!(vault_staker_withdraw_ticket.lrt_amount(), 98_010);
    assert_eq!(
        vault_staker_withdraw_ticket.withdraw_allocation_amount(),
        98_010
    );
}

#[tokio::test]
async fn test_enqueue_withdraw_with_reward_ok() {
    let mut fixture = TestBuilder::new().await;
    let mut vault_program_client = fixture.vault_program_client();
    let mut restaking_program_client = fixture.restaking_program_client();

    // Setup vault with initial deposit
    let (_vault_config_admin, vault_root) = vault_program_client.setup_vault(0, 0).await.unwrap();
    let _restaking_config_admin = restaking_program_client.setup_config().await.unwrap();

    // Setup operator and AVS
    let operator_root = restaking_program_client.setup_operator().await.unwrap();
    let avs_root = restaking_program_client.setup_avs().await.unwrap();

    // Setup necessary relationships
    restaking_program_client
        .operator_avs_opt_in(&operator_root, &avs_root.avs_pubkey)
        .await
        .unwrap();
    restaking_program_client
        .avs_operator_opt_in(&avs_root, &operator_root.operator_pubkey)
        .await
        .unwrap();
    restaking_program_client
        .avs_vault_opt_in(&avs_root, &vault_root.vault_pubkey)
        .await
        .unwrap();
    restaking_program_client
        .operator_vault_opt_in(&operator_root, &vault_root.vault_pubkey)
        .await
        .unwrap();
    vault_program_client
        .vault_avs_opt_in(&vault_root, &avs_root.avs_pubkey)
        .await
        .unwrap();
    vault_program_client
        .vault_operator_opt_in(&vault_root, &operator_root.operator_pubkey)
        .await
        .unwrap();

    let vault = vault_program_client
        .get_vault(&vault_root.vault_pubkey)
        .await
        .unwrap();

    // Initial deposit
    let depositor = Keypair::new();
    fixture.transfer(&depositor.pubkey(), 100.0).await.unwrap();
    fixture
        .mint_to(&vault.supported_mint(), &depositor.pubkey(), 100_000)
        .await
        .unwrap();
    fixture
        .create_ata(&vault.lrt_mint(), &depositor.pubkey())
        .await
        .unwrap();

    // Mint LRT tokens to depositor
    vault_program_client
        .mint_to(
            &vault_root.vault_pubkey,
            &vault.lrt_mint(),
            &depositor,
            &get_associated_token_address(&depositor.pubkey(), &vault.supported_mint()),
            &get_associated_token_address(&vault_root.vault_pubkey, &vault.supported_mint()),
            &get_associated_token_address(&depositor.pubkey(), &vault.lrt_mint()),
            &get_associated_token_address(&vault.fee_owner(), &vault.lrt_mint()),
            None,
            100_000,
        )
        .await
        .unwrap();

    // Delegate all funds to the operator
    vault_program_client
        .delegate(&vault_root, &operator_root.operator_pubkey, 100_000)
        .await
        .unwrap();

    // Simulate rewards by adding more tokens to the vault
    fixture
        .mint_to(&vault.supported_mint(), &vault_root.vault_pubkey, 10_000)
        .await
        .unwrap();
    vault_program_client
        .do_update_vault(&vault_root.vault_pubkey)
        .await
        .unwrap();

    // Enqueue withdrawal for half of the original deposit
    let withdraw_amount = 50_000;
    let VaultStakerWithdrawTicketRoot { base } = vault_program_client
        .do_enqueue_withdraw(&vault_root, &depositor, withdraw_amount)
        .await
        .unwrap();

    // Verify the withdraw ticket
    let withdraw_ticket = vault_program_client
        .get_vault_staker_withdraw_ticket(&vault_root.vault_pubkey, &depositor.pubkey(), &base)
        .await
        .unwrap();

    assert_eq!(withdraw_ticket.lrt_amount(), withdraw_amount);

    // The actual assets to be withdrawn should be more than the LRT amount due to rewards
    assert_eq!(withdraw_ticket.withdraw_allocation_amount(), 55_000);

    // Verify the vault delegation list
    let vault_delegation_list = vault_program_client
        .get_vault_delegation_list(&vault_root.vault_pubkey)
        .await
        .unwrap();

    let delegation = vault_delegation_list.delegations().get(0).unwrap();
    assert_eq!(delegation.staked_amount(), 45_000);
    assert_eq!(delegation.enqueued_for_withdraw_amount(), 55_000);
    assert_eq!(delegation.total_security().unwrap(), 100_000);
}

#[tokio::test]
async fn test_enqueue_withdraw_with_slash_ok() {}

#[tokio::test]
async fn test_enqueue_withdraw_with_multiple_operators_pro_rata_ok() {}

#[tokio::test]
async fn test_enqueue_withdraw_at_epoch_boundary() {}

#[tokio::test]
async fn test_enqueue_withdraw_with_existing_cooldowns() {}

#[tokio::test]
async fn test_enqueue_withdraw_with_zero_amount() {}

#[tokio::test]
async fn test_enqueue_withdraw_insufficient_balance() {}

#[tokio::test]
async fn test_enqueue_withdraw_concurrent_requests() {}

#[tokio::test]
async fn test_enqueue_multiple_same_ticket() {}

#[tokio::test]
async fn test_enqueue_delegation_list_update_needed() {}

#[tokio::test]
async fn test_enqueue_withdraw_with_all_assets_cooling_down() {}

#[tokio::test]
async fn test_enqueue_withdraw_with_partially_cooling_down_assets() {}
