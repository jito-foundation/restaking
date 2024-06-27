use jito_restaking_sanitization::{assert_with_msg, signer::SanitizedSignerAccount};
use jito_vault_core::vault::SanitizedVault;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program_error::ProgramError,
    pubkey::Pubkey,
};

/// Processes the set delegation admin instruction: [`crate::VaultInstruction::SetDelegationAdmin`]
pub fn process_set_delegation_admin(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let mut accounts_iter = accounts.iter();

    let mut vault =
        SanitizedVault::sanitize(program_id, next_account_info(&mut accounts_iter)?, true)?;
    let admin = SanitizedSignerAccount::sanitize(next_account_info(&mut accounts_iter)?, false)?;
    let new_admin =
        SanitizedSignerAccount::sanitize(next_account_info(&mut accounts_iter)?, false)?;

    assert_with_msg(
        *admin.account().key == vault.vault().delegation_admin()
            || *admin.account().key == vault.vault().admin(),
        ProgramError::InvalidAccountData,
        "Admin account does not match vault delegation admin or admin",
    )?;

    vault
        .vault_mut()
        .set_delegation_admin(*new_admin.account().key);

    vault.save()?;

    Ok(())
}
