use jito_restaking_core::{
    config::SanitizedConfig, ncn::SanitizedNcn, operator::SanitizedOperator,
    operator_ncn_ticket::SanitizedOperatorNcnTicket,
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

/// [`crate::RestakingInstruction::OperatorRemoveNcn`]
pub fn process_operator_remove_ncn(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let SanitizedAccounts {
        config,
        operator,
        mut operator_ncn_ticket,
        admin,
    } = SanitizedAccounts::sanitize(program_id, accounts)?;

    operator.operator.check_ncn_admin(admin.account().key)?;

    // TODO (LB): should it get removed from the NCN?

    let slot = Clock::get()?.slot;
    operator_ncn_ticket
        .operator_ncn_ticket_mut()
        .deactivate(slot, config.config().epoch_length())?;

    operator_ncn_ticket.save()?;

    Ok(())
}

struct SanitizedAccounts<'a, 'info> {
    config: SanitizedConfig<'a, 'info>,
    operator: SanitizedOperator<'a, 'info>,
    operator_ncn_ticket: SanitizedOperatorNcnTicket<'a, 'info>,
    admin: SanitizedSignerAccount<'a, 'info>,
}

impl<'a, 'info> SanitizedAccounts<'a, 'info> {
    /// Sanitizes the accounts for the instruction: [`crate::RestakingInstruction::OperatorRemoveNcn`]
    fn sanitize(
        program_id: &Pubkey,
        accounts: &'a [AccountInfo<'info>],
    ) -> Result<SanitizedAccounts<'a, 'info>, ProgramError> {
        let mut accounts_iter = accounts.iter();

        let config =
            SanitizedConfig::sanitize(program_id, next_account_info(&mut accounts_iter)?, false)?;
        let operator =
            SanitizedOperator::sanitize(program_id, next_account_info(&mut accounts_iter)?, false)?;
        let ncn =
            SanitizedNcn::sanitize(program_id, next_account_info(&mut accounts_iter)?, false)?;
        let operator_ncn_ticket = SanitizedOperatorNcnTicket::sanitize(
            program_id,
            next_account_info(&mut accounts_iter)?,
            true,
            operator.account().key,
            ncn.account().key,
        )?;
        let admin =
            SanitizedSignerAccount::sanitize(next_account_info(&mut accounts_iter)?, false)?;

        Ok(SanitizedAccounts {
            operator,
            operator_ncn_ticket,
            admin,
            config,
        })
    }
}
