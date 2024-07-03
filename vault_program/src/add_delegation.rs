use jito_restaking_sanitization::{
    signer::SanitizedSignerAccount, system_program::SanitizedSystemProgram,
};
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

pub fn process_add_delegation(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64,
) -> ProgramResult {
    let SanitizedAccounts {
        config,
        vault,
        mut vault_operator_list,
        operator,
        delegation_admin,
        payer,
    } = SanitizedAccounts::sanitize(program_id, accounts)?;

    vault
        .vault()
        .check_delegation_admin(delegation_admin.account().key)?;

    vault_operator_list
        .vault_operator_list_mut()
        .update_delegations(Clock::get()?.slot, config.config().epoch_length());
    vault_operator_list.vault_operator_list_mut().delegate(
        *operator.key,
        amount,
        vault.vault().tokens_deposited(),
    )?;

    vault_operator_list.save_with_realloc(&Rent::get()?, payer.account())?;

    Ok(())
}

struct SanitizedAccounts<'a, 'info> {
    config: SanitizedConfig<'a, 'info>,
    vault: SanitizedVault<'a, 'info>,
    vault_operator_list: SanitizedVaultOperatorList<'a, 'info>,
    operator: &'a AccountInfo<'info>,
    delegation_admin: SanitizedSignerAccount<'a, 'info>,
    payer: SanitizedSignerAccount<'a, 'info>,
}

impl<'a, 'info> SanitizedAccounts<'a, 'info> {
    fn sanitize(
        program_id: &Pubkey,
        accounts: &'a [AccountInfo<'info>],
    ) -> Result<SanitizedAccounts<'a, 'info>, ProgramError> {
        let mut accounts_iter = accounts.iter();

        let config =
            SanitizedConfig::sanitize(program_id, next_account_info(&mut accounts_iter)?, false)?;
        let vault =
            SanitizedVault::sanitize(program_id, next_account_info(&mut accounts_iter)?, false)?;
        let vault_operator_list = SanitizedVaultOperatorList::sanitize(
            program_id,
            next_account_info(&mut accounts_iter)?,
            true,
            vault.account().key,
        )?;
        let operator = next_account_info(&mut accounts_iter)?;
        let delegation_admin =
            SanitizedSignerAccount::sanitize(next_account_info(&mut accounts_iter)?, false)?;
        let payer = SanitizedSignerAccount::sanitize(next_account_info(&mut accounts_iter)?, true)?;
        let _system_program =
            SanitizedSystemProgram::sanitize(next_account_info(&mut accounts_iter)?)?;

        Ok(SanitizedAccounts {
            config,
            vault,
            vault_operator_list,
            operator,
            delegation_admin,
            payer,
        })
    }
}
