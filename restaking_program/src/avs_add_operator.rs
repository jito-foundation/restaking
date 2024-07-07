use jito_restaking_core::{
    avs::SanitizedAvs, avs_operator_list::SanitizedAvsOperatorList, config::SanitizedConfig,
    operator::SanitizedOperator, operator_avs_list::SanitizedOperatorAvsList,
};
use jito_restaking_sanitization::{
    signer::SanitizedSignerAccount, system_program::SanitizedSystemProgram,
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

/// After an operator opts-in to an AVS, the AVS operator admin can add the operator to the AVS.
/// The operator must have opted-in to the AVS before the AVS opts-in to the operator.
///
/// [`crate::RestakingInstruction::AvsAddOperator`]
pub fn process_avs_add_operator(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let SanitizedAccounts {
        avs,
        mut avs_operator_list,
        operator,
        operator_avs_list,
        admin,
        payer,
    } = SanitizedAccounts::sanitize(program_id, accounts)?;

    avs.avs().check_operator_admin(admin.account().key)?;

    let clock = Clock::get()?;

    operator_avs_list
        .operator_avs_list()
        .check_avs_active(avs.account().key, clock.slot)?;

    avs_operator_list
        .avs_operator_list_mut()
        .add_operator(*operator.account().key, clock.slot)?;

    let rent = Rent::get()?;
    avs_operator_list.save_with_realloc(&rent, payer.account())?;

    Ok(())
}

struct SanitizedAccounts<'a, 'info> {
    avs: SanitizedAvs<'a, 'info>,
    avs_operator_list: SanitizedAvsOperatorList<'a, 'info>,
    operator: SanitizedOperator<'a, 'info>,
    operator_avs_list: SanitizedOperatorAvsList<'a, 'info>,
    admin: SanitizedSignerAccount<'a, 'info>,
    payer: SanitizedSignerAccount<'a, 'info>,
}

impl<'a, 'info> SanitizedAccounts<'a, 'info> {
    /// [`crate::RestakingInstruction::AvsAddOperator`]
    fn sanitize(
        program_id: &Pubkey,
        accounts: &'a [AccountInfo<'info>],
    ) -> Result<SanitizedAccounts<'a, 'info>, ProgramError> {
        let accounts_iter = &mut accounts.iter();

        let _config =
            SanitizedConfig::sanitize(program_id, next_account_info(accounts_iter)?, false)?;
        let avs = SanitizedAvs::sanitize(program_id, next_account_info(accounts_iter)?, false)?;
        let avs_operator_list = SanitizedAvsOperatorList::sanitize(
            program_id,
            next_account_info(accounts_iter)?,
            true,
            avs.account().key,
        )?;
        let operator =
            SanitizedOperator::sanitize(program_id, next_account_info(accounts_iter)?, false)?;
        let operator_avs_list = SanitizedOperatorAvsList::sanitize(
            program_id,
            next_account_info(accounts_iter)?,
            false,
            operator.account().key,
        )?;
        let admin = SanitizedSignerAccount::sanitize(next_account_info(accounts_iter)?, false)?;
        let payer = SanitizedSignerAccount::sanitize(next_account_info(accounts_iter)?, true)?;
        let _system_program = SanitizedSystemProgram::sanitize(next_account_info(accounts_iter)?)?;

        Ok(SanitizedAccounts {
            avs,
            avs_operator_list,
            operator,
            operator_avs_list,
            admin,
            payer,
        })
    }
}
