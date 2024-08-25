use jito_restaking_sdk::error::RestakingError;
use solana_program::{
    instruction::InstructionError, native_token::LAMPORTS_PER_SOL, pubkey::Pubkey,
};
use solana_sdk::{
    account::ReadableAccount, signature::Keypair, signer::Signer, transaction::TransactionError,
};

use crate::fixtures::{
    fixture::TestBuilder,
    restaking_client::{assert_restaking_error, NcnRoot, OperatorRoot},
};

async fn setup() -> (TestBuilder, NcnRoot, OperatorRoot) {
    let fixture = TestBuilder::new().await;
    let mut restaking_program_client = fixture.restaking_program_client();

    let _config_admin = restaking_program_client
        .do_initialize_config()
        .await
        .unwrap();
    let ncn_root = restaking_program_client.do_initialize_ncn().await.unwrap();
    let operator_root = restaking_program_client
        .do_initialize_operator()
        .await
        .unwrap();

    (fixture, ncn_root, operator_root)
}

#[tokio::test]
async fn test_success_withdraw_excess_lamports() {
    let (mut fixture, ncn_root, operator_root) = setup().await;
    let mut restaking_program_client = fixture.restaking_program_client();

    // NCN
    {
        let alice = Keypair::new();
        let expected_amount = 100;

        let mut before_ncn_lamports = 0;
        if let Some(before_ncn) = fixture.get_account(&ncn_root.ncn_pubkey).await.unwrap() {
            before_ncn_lamports = before_ncn.lamports();
        }
        fixture
            .transfer(&ncn_root.ncn_pubkey, expected_amount as f64)
            .await
            .unwrap();

        if let Some(after_ncn) = fixture.get_account(&ncn_root.ncn_pubkey).await.unwrap() {
            assert_eq!(
                after_ncn.lamports(),
                before_ncn_lamports + expected_amount as u64 * LAMPORTS_PER_SOL
            );
        }

        restaking_program_client
            .withdraw_excess_lamports(&ncn_root.ncn_pubkey, &alice.pubkey(), &ncn_root.ncn_admin)
            .await
            .unwrap();

        let after_ncn = fixture
            .get_account(&ncn_root.ncn_pubkey)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(before_ncn_lamports, after_ncn.lamports());

        if let Some(alice_account) = fixture.get_account(&alice.pubkey()).await.unwrap() {
            assert_eq!(alice_account.lamports(), expected_amount * LAMPORTS_PER_SOL);
        }
    }

    // Operator
    {
        let bob = Keypair::new();
        let expected_amount = 100;

        let mut before_operator_lamports = 0;
        if let Some(before_operator) = fixture
            .get_account(&operator_root.operator_pubkey)
            .await
            .unwrap()
        {
            before_operator_lamports = before_operator.lamports();
        }
        fixture
            .transfer(&operator_root.operator_pubkey, expected_amount as f64)
            .await
            .unwrap();

        if let Some(after_operator) = fixture
            .get_account(&operator_root.operator_pubkey)
            .await
            .unwrap()
        {
            assert_eq!(
                after_operator.lamports(),
                before_operator_lamports + expected_amount as u64 * LAMPORTS_PER_SOL
            );
        }

        restaking_program_client
            .withdraw_excess_lamports(
                &operator_root.operator_pubkey,
                &bob.pubkey(),
                &operator_root.operator_admin,
            )
            .await
            .unwrap();

        let after_operator = fixture
            .get_account(&operator_root.operator_pubkey)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(before_operator_lamports, after_operator.lamports());

        if let Some(bob_account) = fixture.get_account(&bob.pubkey()).await.unwrap() {
            assert_eq!(bob_account.lamports(), expected_amount * LAMPORTS_PER_SOL);
        }
    }
}

#[tokio::test]
async fn test_wrong_admin_signed_fail() {
    let (mut fixture, ncn_root, operator_root) = setup().await;
    let mut restaking_program_client = fixture.restaking_program_client();

    // NCN
    {
        let alice = Keypair::new();
        let expected_amount = 100.0;

        let mut before_ncn_lamports = 0;
        if let Some(before_ncn) = fixture.get_account(&ncn_root.ncn_pubkey).await.unwrap() {
            before_ncn_lamports = before_ncn.lamports();
        }
        fixture
            .transfer(&ncn_root.ncn_pubkey, expected_amount)
            .await
            .unwrap();

        if let Some(after_ncn) = fixture.get_account(&ncn_root.ncn_pubkey).await.unwrap() {
            assert_eq!(
                after_ncn.lamports(),
                before_ncn_lamports + expected_amount as u64 * LAMPORTS_PER_SOL
            );
        }

        let wrong_admin = Keypair::new();
        fixture
            .transfer(&wrong_admin.pubkey(), 100.0)
            .await
            .unwrap();
        let response = restaking_program_client
            .withdraw_excess_lamports(&ncn_root.ncn_pubkey, &alice.pubkey(), &wrong_admin)
            .await;

        assert_restaking_error(response, RestakingError::NcnAdminInvalid);
    }

    // Operator
    {
        let bob = Keypair::new();
        let expected_amount = 100.0;

        let mut before_operator_lamports = 0;
        if let Some(before_operator) = fixture
            .get_account(&operator_root.operator_pubkey)
            .await
            .unwrap()
        {
            before_operator_lamports = before_operator.lamports();
        }
        fixture
            .transfer(&operator_root.operator_pubkey, expected_amount)
            .await
            .unwrap();

        if let Some(after_operator) = fixture
            .get_account(&operator_root.operator_pubkey)
            .await
            .unwrap()
        {
            assert_eq!(
                after_operator.lamports(),
                before_operator_lamports + expected_amount as u64 * LAMPORTS_PER_SOL
            );
        }

        let wrong_admin = Keypair::new();
        fixture
            .transfer(&wrong_admin.pubkey(), 100.0)
            .await
            .unwrap();
        let response = restaking_program_client
            .withdraw_excess_lamports(&operator_root.operator_pubkey, &bob.pubkey(), &wrong_admin)
            .await;

        assert_restaking_error(response, RestakingError::OperatorAdminInvalid);
    }
}

#[tokio::test]
async fn test_wrong_account_fail() {
    let (mut fixture, ncn_root, operator_root) = setup().await;
    let mut restaking_program_client = fixture.restaking_program_client();

    // NCN
    {
        let alice = Keypair::new();
        let expected_amount = 100.0;

        let mut before_ncn_lamports = 0;
        if let Some(before_ncn) = fixture.get_account(&ncn_root.ncn_pubkey).await.unwrap() {
            before_ncn_lamports = before_ncn.lamports();
        }
        fixture
            .transfer(&ncn_root.ncn_pubkey, expected_amount)
            .await
            .unwrap();

        if let Some(after_ncn) = fixture.get_account(&ncn_root.ncn_pubkey).await.unwrap() {
            assert_eq!(
                after_ncn.lamports(),
                before_ncn_lamports + expected_amount as u64 * LAMPORTS_PER_SOL
            );
        }

        let wrong_account = Pubkey::new_unique();
        let transaction_error = restaking_program_client
            .withdraw_excess_lamports(&wrong_account, &alice.pubkey(), &ncn_root.ncn_admin)
            .await
            .unwrap_err()
            .to_transaction_error()
            .unwrap();

        assert_eq!(
            transaction_error,
            TransactionError::InstructionError(0, InstructionError::InvalidAccountData)
        );
    }

    // Operator
    {
        let bob = Keypair::new();
        let expected_amount = 100.0;

        let mut before_operator_lamports = 0;
        if let Some(before_operator) = fixture
            .get_account(&operator_root.operator_pubkey)
            .await
            .unwrap()
        {
            before_operator_lamports = before_operator.lamports();
        }
        fixture
            .transfer(&operator_root.operator_pubkey, expected_amount)
            .await
            .unwrap();

        if let Some(after_operator) = fixture
            .get_account(&operator_root.operator_pubkey)
            .await
            .unwrap()
        {
            assert_eq!(
                after_operator.lamports(),
                before_operator_lamports + expected_amount as u64 * LAMPORTS_PER_SOL
            );
        }

        let wrong_account = Pubkey::new_unique();
        let transaction_error = restaking_program_client
            .withdraw_excess_lamports(&wrong_account, &bob.pubkey(), &operator_root.operator_admin)
            .await
            .unwrap_err()
            .to_transaction_error()
            .unwrap();

        assert_eq!(
            transaction_error,
            TransactionError::InstructionError(0, InstructionError::InvalidAccountData)
        );
    }
}
