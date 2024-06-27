use jito_restaking_sanitization::{assert_with_msg, signer::SanitizedSignerAccount};
use jito_vault_core::vault::SanitizedVault;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program_error::ProgramError,
    pubkey::Pubkey,
};

pub fn process_set_capacity(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    capacity: u64,
) -> ProgramResult {
    let mut accounts_iter = accounts.iter();

    let mut lrt_account =
        SanitizedVault::sanitize(program_id, next_account_info(&mut accounts_iter)?, true)?;

    let admin = SanitizedSignerAccount::sanitize(next_account_info(&mut accounts_iter)?, false)?;

    assert_with_msg(
        *admin.account().key == lrt_account.vault().admin(),
        ProgramError::InvalidAccountData,
        "Admin account does not match LRT admin",
    )?;

    lrt_account.vault_mut().set_capacity(capacity);
    lrt_account.save()?;

    Ok(())
}
