use jito_restaking_sanitization::signer::SanitizedSignerAccount;
use jito_vault_core::{
    config::SanitizedConfig, vault::SanitizedVault,
    vault_delegation_list::SanitizedVaultDelegationList,
};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::Clock,
    entrypoint::ProgramResult,
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvar::Sysvar,
};

pub fn process_remove_delegation(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64,
) -> ProgramResult {
    let SanitizedAccounts {
        config,
        vault,
        mut vault_delegation_list,
        operator,
        delegation_admin,
    } = SanitizedAccounts::sanitize(program_id, accounts)?;

    vault
        .vault()
        .check_delegation_admin(delegation_admin.account().key)?;

    let slot = Clock::get()?.slot;
    let epoch_length = config.config().epoch_length();
    vault_delegation_list
        .vault_delegation_list_mut()
        .check_update_needed(slot, epoch_length)?;
    vault_delegation_list
        .vault_delegation_list_mut()
        .undelegate(*operator.key, amount)?;

    vault_delegation_list.save()?;

    Ok(())
}

struct SanitizedAccounts<'a, 'info> {
    config: SanitizedConfig<'a, 'info>,
    vault: SanitizedVault<'a, 'info>,
    vault_delegation_list: SanitizedVaultDelegationList<'a, 'info>,
    operator: &'a AccountInfo<'info>,
    delegation_admin: SanitizedSignerAccount<'a, 'info>,
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
        let vault_delegation_list = SanitizedVaultDelegationList::sanitize(
            program_id,
            next_account_info(&mut accounts_iter)?,
            true,
            vault.account().key,
        )?;
        let operator = next_account_info(&mut accounts_iter)?;
        let delegation_admin =
            SanitizedSignerAccount::sanitize(next_account_info(&mut accounts_iter)?, false)?;

        Ok(SanitizedAccounts {
            config,
            vault,
            vault_delegation_list,
            operator,
            delegation_admin,
        })
    }
}
