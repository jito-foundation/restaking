use jito_restaking_core::operator::{SanitizedOperator, SanitizedOperatorVaultList};
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

/// Adds an operator to the vault's operator list.
///
/// # Behavior
/// * The vault admin shall have the ability to add support for a new AVS
/// if the AVS is actively supporting the vault
///
/// Instruction: [`crate::VaultInstruction::AddAvs`]
pub fn process_vault_add_operator(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let SanitizedAccounts {
        vault,
        mut vault_operator_list,
        operator,
        operator_vault_list,
        admin,
        payer,
    } = SanitizedAccounts::sanitize(program_id, accounts)?;

    vault.vault().check_admin(admin.account().key)?;

    let slot = Clock::get()?.slot;
    // The operator shall support the vault for it to be added
    operator_vault_list
        .operator_vault_list()
        .check_active_vault(*vault.account().key, slot)?;
    vault_operator_list
        .vault_operator_list_mut()
        .add_operator(*operator.account().key, slot)?;

    vault_operator_list.save_with_realloc(&Rent::get()?, payer.account())?;

    Ok(())
}

struct SanitizedAccounts<'a, 'info> {
    // config: SanitizedConfig<'a, 'info>,
    vault: SanitizedVault<'a, 'info>,
    vault_operator_list: SanitizedVaultOperatorList<'a, 'info>,
    operator: SanitizedOperator<'a, 'info>,
    operator_vault_list: SanitizedOperatorVaultList<'a, 'info>,
    admin: SanitizedSignerAccount<'a, 'info>,
    payer: SanitizedSignerAccount<'a, 'info>,
    // system_program: SanitizedSystemProgram<'a, 'info>,
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
        let operator_vault_list = SanitizedOperatorVaultList::sanitize(
            &config.config().restaking_program(),
            next_account_info(&mut accounts_iter)?,
            true,
            operator.account().key,
        )?;
        let admin =
            SanitizedSignerAccount::sanitize(next_account_info(&mut accounts_iter)?, false)?;
        let payer = SanitizedSignerAccount::sanitize(next_account_info(&mut accounts_iter)?, true)?;
        let _system_program =
            SanitizedSystemProgram::sanitize(next_account_info(&mut accounts_iter)?)?;

        Ok(SanitizedAccounts {
            // config,
            vault,
            vault_operator_list,
            operator,
            operator_vault_list,
            admin,
            payer,
            // system_program,
        })
    }
}
