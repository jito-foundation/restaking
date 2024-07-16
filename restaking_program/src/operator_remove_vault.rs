use jito_restaking_core::{
    config::SanitizedConfig, operator::SanitizedOperator,
    operator_vault_ticket::SanitizedOperatorVaultTicket,
};
use jito_restaking_sanitization::signer::SanitizedSignerAccount;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::Clock,
    entrypoint::ProgramResult,
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvar::Sysvar,
};

/// [`crate::RestakingInstruction::OperatorRemoveVault`]
pub fn process_operator_remove_vault(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let SanitizedAccounts {
        operator,
        mut operator_vault_ticket,
        admin,
    } = SanitizedAccounts::sanitize(program_id, accounts)?;

    operator.operator().check_vault_admin(admin.account().key)?;

    let slot = Clock::get()?.slot;
    operator_vault_ticket
        .operator_vault_ticket_mut()
        .deactivate(slot)?;

    operator_vault_ticket.save()?;

    Ok(())
}

struct SanitizedAccounts<'a, 'info> {
    operator: SanitizedOperator<'a, 'info>,
    operator_vault_ticket: SanitizedOperatorVaultTicket<'a, 'info>,
    admin: SanitizedSignerAccount<'a, 'info>,
}

impl<'a, 'info> SanitizedAccounts<'a, 'info> {
    /// Sanitizes the accounts for the instruction: [`crate::RestakingInstruction::OperatorRemoveVault`]
    pub fn sanitize(
        program_id: &Pubkey,
        accounts: &'a [AccountInfo<'info>],
    ) -> Result<SanitizedAccounts<'a, 'info>, ProgramError> {
        let accounts_iter = &mut accounts.iter();

        let _config =
            SanitizedConfig::sanitize(program_id, next_account_info(accounts_iter)?, false)?;
        let operator =
            SanitizedOperator::sanitize(program_id, next_account_info(accounts_iter)?, false)?;
        let vault = next_account_info(accounts_iter)?;
        let operator_vault_ticket = SanitizedOperatorVaultTicket::sanitize(
            program_id,
            next_account_info(accounts_iter)?,
            true,
            operator.account().key,
            vault.key,
        )?;
        let admin = SanitizedSignerAccount::sanitize(next_account_info(accounts_iter)?, false)?;

        Ok(SanitizedAccounts {
            operator,
            operator_vault_ticket,
            admin,
        })
    }
}
