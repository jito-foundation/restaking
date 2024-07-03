use jito_restaking_core::operator::SanitizedOperator;
use jito_restaking_sanitization::signer::SanitizedSignerAccount;
use jito_vault_core::{
    config::SanitizedConfig, vault::SanitizedVault, vault_operator_list::SanitizedVaultOperatorList,
};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::Clock,
    entrypoint::ProgramResult,
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvar::Sysvar,
};

/// Processes the vault remove operator instruction: [`crate::VaultInstruction::RemoveOperator`]
pub fn process_vault_remove_operator(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let SanitizedAccounts {
        vault,
        mut vault_operator_list,
        operator,
        admin,
    } = SanitizedAccounts::sanitize(program_id, accounts)?;

    vault.vault().check_admin(admin.account().key)?;

    let slot = Clock::get()?.slot;
    vault_operator_list
        .vault_operator_list_mut()
        .remove_operator(*operator.account().key, slot)?;

    // TODO (LB): should one deactivate the stake here as well?

    vault_operator_list.save()?;

    Ok(())
}

struct SanitizedAccounts<'a, 'info> {
    // config: SanitizedConfig<'a, 'info>,
    vault: SanitizedVault<'a, 'info>,
    vault_operator_list: SanitizedVaultOperatorList<'a, 'info>,
    operator: SanitizedOperator<'a, 'info>,
    admin: SanitizedSignerAccount<'a, 'info>,
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
        let operator = SanitizedOperator::sanitize(
            &config.config().restaking_program(),
            next_account_info(&mut accounts_iter)?,
            false,
        )?;
        let admin =
            SanitizedSignerAccount::sanitize(next_account_info(&mut accounts_iter)?, false)?;

        Ok(SanitizedAccounts {
            // config,
            vault,
            vault_operator_list,
            operator,
            admin,
        })
    }
}
