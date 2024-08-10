use solana_program::program_error::ProgramError;
use solana_program_test::BanksClientError;
use thiserror::Error;

pub mod fixture;
pub mod restaking_client;
pub mod vault_client;

#[derive(Error, Debug)]
pub enum TestError {
    #[error(transparent)]
    BanksClientError(#[from] BanksClientError),
    #[error(transparent)]
    ProgramError(#[from] ProgramError),
}
