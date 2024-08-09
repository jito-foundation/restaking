use jito_restaking_core::{
    config::SanitizedConfig, ncn::SanitizedNcn, ncn_operator_ticket::SanitizedNcnOperatorTicket,
    operator::SanitizedOperator,
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

/// The NCN admin can remove a node operator from the NCN.
/// This method is permissioned to the NCN admin.
/// [`crate::RestakingInstruction::NcnRemoveOperator`]
pub fn process_ncn_cooldown_operator(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let SanitizedAccounts {
        ncn,
        mut ncn_operator_ticket,
        admin,
    } = SanitizedAccounts::sanitize(program_id, accounts)?;

    ncn.ncn().check_operator_admin(admin.account().key)?;

    let slot = Clock::get()?.slot;

    ncn_operator_ticket
        .ncn_operator_ticket_mut()
        .cooldown(slot)?;

    ncn_operator_ticket.save()?;

    Ok(())
}

struct SanitizedAccounts<'a, 'info> {
    ncn: SanitizedNcn<'a, 'info>,
    ncn_operator_ticket: SanitizedNcnOperatorTicket<'a, 'info>,
    admin: SanitizedSignerAccount<'a, 'info>,
}

impl<'a, 'info> SanitizedAccounts<'a, 'info> {
    /// Sanitizes the accounts for the instruction: [`crate::RestakingInstruction::NcnCooldownOperator`]
    fn sanitize(
        program_id: &Pubkey,
        accounts: &'a [AccountInfo<'info>],
    ) -> Result<SanitizedAccounts<'a, 'info>, ProgramError> {
        let accounts_iter = &mut accounts.iter();

        let _config =
            SanitizedConfig::sanitize(program_id, next_account_info(accounts_iter)?, false)?;
        let ncn = SanitizedNcn::sanitize(program_id, next_account_info(accounts_iter)?, false)?;
        let operator =
            SanitizedOperator::sanitize(program_id, next_account_info(accounts_iter)?, false)?;
        let ncn_operator_ticket = SanitizedNcnOperatorTicket::sanitize(
            program_id,
            next_account_info(accounts_iter)?,
            true,
            ncn.account().key,
            operator.account().key,
        )?;
        let admin = SanitizedSignerAccount::sanitize(next_account_info(accounts_iter)?, false)?;

        Ok(SanitizedAccounts {
            ncn,
            ncn_operator_ticket,
            admin,
        })
    }
}
