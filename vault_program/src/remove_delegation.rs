use jito_restaking_sanitization::{assert_with_msg, signer::SanitizedSignerAccount};
use jito_vault_core::{
    config::SanitizedConfig, vault::SanitizedVault, vault_operator_list::SanitizedVaultOperatorList,
};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::Clock,
    entrypoint::ProgramResult,
    program_error::ProgramError,
    pubkey::Pubkey,
    rent::Rent,
    sysvar::Sysvar,
};

pub fn process_remove_delegation(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64,
) -> ProgramResult {
    let mut accounts_iter = accounts.iter();

    let config =
        SanitizedConfig::sanitize(program_id, next_account_info(&mut accounts_iter)?, false)?;
    let vault =
        SanitizedVault::sanitize(program_id, next_account_info(&mut accounts_iter)?, false)?;
    let mut vault_operator_list = SanitizedVaultOperatorList::sanitize(
        program_id,
        next_account_info(&mut accounts_iter)?,
        true,
        vault.account().key,
    )?;
    let operator = next_account_info(&mut accounts_iter)?;
    let delegation_admin =
        SanitizedSignerAccount::sanitize(next_account_info(&mut accounts_iter)?, false)?;
    let payer = SanitizedSignerAccount::sanitize(next_account_info(&mut accounts_iter)?, true)?;

    assert_with_msg(
        vault.vault().delegation_admin() == *delegation_admin.account().key,
        ProgramError::InvalidAccountData,
        "Admin account does not match vault delegation admin",
    )?;

    vault_operator_list
        .vault_operator_list_mut()
        .update_delegations(Clock::get()?.slot, config.config().epoch_length());

    vault_operator_list
        .vault_operator_list_mut()
        .undelegate(*operator.key, amount)?;

    vault_operator_list.save_with_realloc(&Rent::get()?, payer.account())?;

    Ok(())
}
