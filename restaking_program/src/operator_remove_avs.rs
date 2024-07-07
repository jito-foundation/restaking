use jito_restaking_core::{
    avs::SanitizedAvs, config::SanitizedConfig, operator::SanitizedOperator,
    operator_avs_list::SanitizedOperatorAvsList,
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

/// [`crate::RestakingInstruction::OperatorRemoveAvs`]
pub fn process_operator_remove_avs(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let SanitizedAccounts {
        operator,
        mut operator_avs_list,
        avs,
        admin,
    } = SanitizedAccounts::sanitize(program_id, accounts)?;

    operator.operator().check_admin(admin.account().key)?;

    // TODO (LB): should it get removed from the AVS?

    let slot = Clock::get()?.slot;
    operator_avs_list
        .operator_avs_list_mut()
        .remove_avs(*avs.account().key, slot)?;

    operator_avs_list.save()?;

    Ok(())
}

struct SanitizedAccounts<'a, 'info> {
    operator: SanitizedOperator<'a, 'info>,
    operator_avs_list: SanitizedOperatorAvsList<'a, 'info>,
    avs: SanitizedAvs<'a, 'info>,
    admin: SanitizedSignerAccount<'a, 'info>,
}

impl<'a, 'info> SanitizedAccounts<'a, 'info> {
    fn sanitize(
        program_id: &Pubkey,
        accounts: &'a [AccountInfo<'info>],
    ) -> Result<SanitizedAccounts<'a, 'info>, ProgramError> {
        let mut accounts_iter = accounts.iter();

        let _config =
            SanitizedConfig::sanitize(program_id, next_account_info(&mut accounts_iter)?, false)?;
        let operator =
            SanitizedOperator::sanitize(program_id, next_account_info(&mut accounts_iter)?, false)?;
        let operator_avs_list = SanitizedOperatorAvsList::sanitize(
            program_id,
            next_account_info(&mut accounts_iter)?,
            true,
            operator.account().key,
        )?;
        let avs =
            SanitizedAvs::sanitize(program_id, next_account_info(&mut accounts_iter)?, false)?;
        let admin =
            SanitizedSignerAccount::sanitize(next_account_info(&mut accounts_iter)?, false)?;

        Ok(SanitizedAccounts {
            operator,
            operator_avs_list,
            avs,
            admin,
        })
    }
}
