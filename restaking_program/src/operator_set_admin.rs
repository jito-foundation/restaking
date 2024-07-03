use jito_restaking_core::operator::SanitizedOperator;
use jito_restaking_sanitization::signer::SanitizedSignerAccount;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program_error::ProgramError,
    pubkey::Pubkey,
};

/// The node operator admin can set a new admin for the node operator.
/// This method is permissioned to the node operator admin and both the old and new admins must sign.
///
/// [`crate::RestakingInstruction::OperatorSetAdmin`]
pub fn process_set_node_operator_admin(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let SanitizedAccounts {
        mut operator,
        old_admin,
        new_admin,
    } = SanitizedAccounts::sanitize(program_id, accounts)?;

    operator.operator().check_admin(old_admin.account().key)?;
    operator.operator_mut().set_admin(*new_admin.account().key);

    operator.save()?;

    Ok(())
}

struct SanitizedAccounts<'a, 'info> {
    operator: SanitizedOperator<'a, 'info>,
    old_admin: SanitizedSignerAccount<'a, 'info>,
    new_admin: SanitizedSignerAccount<'a, 'info>,
}

impl<'a, 'info> SanitizedAccounts<'a, 'info> {
    fn sanitize(
        program_id: &Pubkey,
        accounts: &'a [AccountInfo<'info>],
    ) -> Result<SanitizedAccounts<'a, 'info>, ProgramError> {
        let accounts_iter = &mut accounts.iter();

        let operator =
            SanitizedOperator::sanitize(program_id, next_account_info(accounts_iter)?, true)?;
        let old_admin = SanitizedSignerAccount::sanitize(next_account_info(accounts_iter)?, false)?;
        let new_admin = SanitizedSignerAccount::sanitize(next_account_info(accounts_iter)?, false)?;

        Ok(SanitizedAccounts {
            operator,
            old_admin,
            new_admin,
        })
    }
}
