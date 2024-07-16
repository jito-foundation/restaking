use jito_restaking_core::{
    avs::SanitizedAvs, avs_operator_ticket::SanitizedAvsOperatorTicket, config::SanitizedConfig,
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

/// The AVS admin can remove a node operator from the AVS.
/// This method is permissioned to the AVS admin.
/// [`crate::RestakingInstruction::AvsRemoveOperator`]
pub fn process_avs_remove_operator(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let SanitizedAccounts {
        avs,
        mut avs_operator_ticket,
        admin,
    } = SanitizedAccounts::sanitize(program_id, accounts)?;

    avs.avs().check_operator_admin(admin.account().key)?;

    let slot = Clock::get()?.slot;

    avs_operator_ticket
        .avs_operator_ticket_mut()
        .deactivate(slot)?;

    avs_operator_ticket.save()?;

    Ok(())
}

struct SanitizedAccounts<'a, 'info> {
    avs: SanitizedAvs<'a, 'info>,
    avs_operator_ticket: SanitizedAvsOperatorTicket<'a, 'info>,
    admin: SanitizedSignerAccount<'a, 'info>,
}

impl<'a, 'info> SanitizedAccounts<'a, 'info> {
    /// Sanitizes the accounts for the instruction: [`crate::RestakingInstruction::AvsRemoveOperator`]
    fn sanitize(
        program_id: &Pubkey,
        accounts: &'a [AccountInfo<'info>],
    ) -> Result<SanitizedAccounts<'a, 'info>, ProgramError> {
        let accounts_iter = &mut accounts.iter();

        let _config =
            SanitizedConfig::sanitize(program_id, next_account_info(accounts_iter)?, false)?;
        let avs = SanitizedAvs::sanitize(program_id, next_account_info(accounts_iter)?, false)?;
        let operator =
            SanitizedOperator::sanitize(program_id, next_account_info(accounts_iter)?, false)?;
        let avs_operator_ticket = SanitizedAvsOperatorTicket::sanitize(
            program_id,
            next_account_info(accounts_iter)?,
            true,
            avs.account().key,
            operator.account().key,
        )?;
        let admin = SanitizedSignerAccount::sanitize(next_account_info(accounts_iter)?, false)?;

        Ok(SanitizedAccounts {
            avs,
            avs_operator_ticket,
            admin,
        })
    }
}
