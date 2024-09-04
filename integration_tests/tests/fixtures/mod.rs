use solana_program::{instruction::InstructionError, program_error::ProgramError};
use solana_program_test::BanksClientError;
use solana_sdk::transaction::TransactionError;
use thiserror::Error;

pub mod fixture;
pub mod restaking_client;
pub mod vault_client;

pub type TestResult<T> = Result<T, TestError>;

#[derive(Error, Debug)]
pub enum TestError {
    #[error(transparent)]
    BanksClientError(#[from] BanksClientError),
    #[error(transparent)]
    ProgramError(#[from] ProgramError),
}

impl TestError {
    pub fn to_transaction_error(&self) -> Option<TransactionError> {
        match self {
            TestError::BanksClientError(e) => match e {
                BanksClientError::TransactionError(e) => Some(e.clone()),
                BanksClientError::SimulationError { err, .. } => Some(err.clone()),
                _ => None,
            },
            TestError::ProgramError(_) => None,
        }
    }
}

#[inline(always)]
#[track_caller]
pub fn assert_ix_error<T>(test_error: Result<T, TestError>, ix_error: InstructionError) {
    assert!(test_error.is_err());
    assert_eq!(
        test_error.err().unwrap().to_transaction_error().unwrap(),
        TransactionError::InstructionError(0, ix_error)
    );
}
