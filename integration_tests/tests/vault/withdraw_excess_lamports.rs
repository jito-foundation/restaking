use jito_vault_sdk::error::VaultError;
use solana_program::{
    instruction::InstructionError, native_token::LAMPORTS_PER_SOL, pubkey::Pubkey,
};
use solana_sdk::{
    account::ReadableAccount, signature::Keypair, signer::Signer, transaction::TransactionError,
};

use crate::fixtures::{
    fixture::TestBuilder,
    vault_client::{assert_vault_error, VaultRoot},
};

async fn setup() -> (TestBuilder, Pubkey, Keypair) {
    let fixture = TestBuilder::new().await;

    let mut vault_program_client = fixture.vault_program_client();

    let deposit_fee_bps = 99;
    let withdrawal_fee_bps = 100;
    let reward_fee_bps = 101;

    let (
        _config_admin,
        VaultRoot {
            vault_pubkey,
            vault_admin,
        },
    ) = vault_program_client
        .setup_config_and_vault(deposit_fee_bps, withdrawal_fee_bps, reward_fee_bps)
        .await
        .unwrap();

    (fixture, vault_pubkey, vault_admin)
}

#[tokio::test]
async fn test_success_withdraw_excess_lamports() {
    let (mut fixture, vault_pubkey, vault_admin) = setup().await;
    let mut vault_program_client = fixture.vault_program_client();

    let alice = Keypair::new();
    let expected_amount = 100.0;

    let mut before_vault_lamports = 0;
    if let Some(before_vault) = fixture.get_account(&vault_pubkey).await.unwrap() {
        before_vault_lamports = before_vault.lamports();
    }
    fixture
        .transfer(&vault_pubkey, expected_amount)
        .await
        .unwrap();

    if let Some(after_vault) = fixture.get_account(&vault_pubkey).await.unwrap() {
        assert_eq!(
            after_vault.lamports(),
            before_vault_lamports + expected_amount as u64 * LAMPORTS_PER_SOL
        );
    }

    vault_program_client
        .withdraw_excess_lamports(&vault_pubkey, &alice.pubkey(), &vault_admin)
        .await
        .unwrap();

    if let Some(after_vault) = fixture.get_account(&vault_pubkey).await.unwrap() {
        assert_eq!(before_vault_lamports, after_vault.lamports());
    }

    if let Some(alice_account) = fixture.get_account(&alice.pubkey()).await.unwrap() {
        assert_eq!(alice_account.lamports(), 100 * LAMPORTS_PER_SOL);
    }
}

#[tokio::test]
async fn test_wrong_admin_signed_fail() {
    let (mut fixture, vault_pubkey, _) = setup().await;
    let mut vault_program_client = fixture.vault_program_client();

    let alice = Keypair::new();
    let expected_amount = 100.0;

    let mut before_vault_lamports = 0;
    if let Some(before_vault) = fixture.get_account(&vault_pubkey).await.unwrap() {
        before_vault_lamports = before_vault.lamports();
    }
    fixture
        .transfer(&vault_pubkey, expected_amount)
        .await
        .unwrap();

    if let Some(after_vault) = fixture.get_account(&vault_pubkey).await.unwrap() {
        assert_eq!(
            after_vault.lamports(),
            before_vault_lamports + expected_amount as u64 * LAMPORTS_PER_SOL
        );
    }

    let wrong_admin = Keypair::new();
    fixture
        .transfer(&wrong_admin.pubkey(), 100.0)
        .await
        .unwrap();
    let response = vault_program_client
        .withdraw_excess_lamports(&vault_pubkey, &alice.pubkey(), &wrong_admin)
        .await;

    assert_vault_error(response, VaultError::VaultAdminInvalid);
}

#[tokio::test]
async fn test_wrong_account_fail() {
    let (mut fixture, vault_pubkey, vault_admin) = setup().await;
    let mut vault_program_client = fixture.vault_program_client();

    let alice = Keypair::new();
    let expected_amount = 100.0;

    let mut before_vault_lamports = 0;
    if let Some(before_vault) = fixture.get_account(&vault_pubkey).await.unwrap() {
        before_vault_lamports = before_vault.lamports();
    }
    fixture
        .transfer(&vault_pubkey, expected_amount)
        .await
        .unwrap();

    if let Some(after_vault) = fixture.get_account(&vault_pubkey).await.unwrap() {
        assert_eq!(
            after_vault.lamports(),
            before_vault_lamports + expected_amount as u64 * LAMPORTS_PER_SOL
        );
    }

    let wrong_account = Pubkey::new_unique();
    let transaction_error = vault_program_client
        .withdraw_excess_lamports(&wrong_account, &alice.pubkey(), &vault_admin)
        .await
        .unwrap_err()
        .to_transaction_error()
        .unwrap();

    assert_eq!(
        transaction_error,
        TransactionError::InstructionError(0, InstructionError::InvalidAccountData)
    );
}
