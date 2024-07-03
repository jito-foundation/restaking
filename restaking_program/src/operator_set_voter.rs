use jito_restaking_core::operator::SanitizedOperator;
use jito_restaking_sanitization::signer::SanitizedSignerAccount;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program_error::ProgramError,
    pubkey::Pubkey,
};

/// The node operator admin can set a new voter for the node operator.
/// This method is permissioned to the node operator admin.
///
/// [`crate::RestakingInstruction::OperatorSetVoter`]
pub fn process_set_node_operator_voter(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let SanitizedAccounts {
        mut operator,
        admin,
        voter,
    } = SanitizedAccounts::sanitize(program_id, accounts)?;

    operator.operator().check_admin(admin.account().key)?;
    operator.operator_mut().set_voter(*voter.key);
    operator.save()?;

    Ok(())
}

struct SanitizedAccounts<'a, 'info> {
    operator: SanitizedOperator<'a, 'info>,
    admin: SanitizedSignerAccount<'a, 'info>,
    voter: &'a AccountInfo<'info>,
}

impl<'a, 'info> SanitizedAccounts<'a, 'info> {
    fn sanitize(
        program_id: &Pubkey,
        accounts: &'a [AccountInfo<'info>],
    ) -> Result<SanitizedAccounts<'a, 'info>, ProgramError> {
        let accounts_iter = &mut accounts.iter();

        let operator =
            SanitizedOperator::sanitize(program_id, next_account_info(accounts_iter)?, true)?;
        let admin = SanitizedSignerAccount::sanitize(next_account_info(accounts_iter)?, true)?;
        let voter = next_account_info(accounts_iter)?;

        Ok(SanitizedAccounts {
            operator,
            admin,
            voter,
        })
    }
}
