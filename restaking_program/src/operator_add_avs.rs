use jito_restaking_core::{
    avs::SanitizedAvs,
    config::SanitizedConfig,
    operator::{SanitizedOperator, SanitizedOperatorAvsList},
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

/// The node operator admin can add support for running an AVS.
/// This method is permissioned to the node operator admin.
///
/// [`crate::RestakingInstruction::OperatorAddAvs`]
pub fn process_operator_add_avs(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let SanitizedAccounts {
        operator,
        mut operator_avs_list,
        avs,
        admin,
        payer,
    } = SanitizedAccounts::sanitize(program_id, accounts)?;

    operator.operator().check_admin(admin.account().key)?;

    let slot = Clock::get()?.slot;
    operator_avs_list
        .operator_avs_list_mut()
        .add_avs(*avs.account().key, slot)?;

    let rent = Rent::get()?;
    operator_avs_list.save_with_realloc(&rent, payer.account())?;

    Ok(())
}

struct SanitizedAccounts<'a, 'info> {
    operator: SanitizedOperator<'a, 'info>,
    operator_avs_list: SanitizedOperatorAvsList<'a, 'info>,
    avs: SanitizedAvs<'a, 'info>,
    admin: SanitizedSignerAccount<'a, 'info>,
    payer: SanitizedSignerAccount<'a, 'info>,
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
        let payer = SanitizedSignerAccount::sanitize(next_account_info(&mut accounts_iter)?, true)?;
        let _system_program =
            SanitizedSystemProgram::sanitize(next_account_info(&mut accounts_iter)?)?;

        Ok(SanitizedAccounts {
            operator,
            operator_avs_list,
            avs,
            admin,
            payer,
        })
    }
}
