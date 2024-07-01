use jito_restaking_core::{
    avs::SanitizedAvs,
    config::SanitizedConfig,
    operator::{SanitizedNodeOperatorAvsList, SanitizedOperator},
};
use jito_restaking_sanitization::{assert_with_msg, signer::SanitizedSignerAccount};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::Clock,
    entrypoint::ProgramResult,
    program_error::ProgramError,
    pubkey::Pubkey,
    rent::Rent,
    sysvar::Sysvar,
};

/// The node operator admin can remove support for running an AVS.
/// This method is permissioned to the node operator admin.
///
/// [`crate::RestakingInstruction::OperatorRemoveAvs`]
pub fn process_operator_remove_avs(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    let _config = SanitizedConfig::sanitize(program_id, next_account_info(accounts_iter)?, false)?;
    let node_operator =
        SanitizedOperator::sanitize(program_id, next_account_info(accounts_iter)?, true)?;
    let mut node_operator_avs_list = SanitizedNodeOperatorAvsList::sanitize(
        program_id,
        next_account_info(accounts_iter)?,
        true,
        node_operator.account().key,
    )?;

    let avs = SanitizedAvs::sanitize(program_id, next_account_info(accounts_iter)?, false)?;

    let admin = SanitizedSignerAccount::sanitize(next_account_info(accounts_iter)?, true)?;
    assert_with_msg(
        node_operator.operator().admin() == *admin.account().key,
        ProgramError::InvalidAccountData,
        "Admin is not the node operator admin",
    )?;

    let clock = Clock::get()?;

    assert_with_msg(
        node_operator_avs_list
            .operator_avs_list_mut()
            .remove_avs(*avs.account().key, clock.slot),
        ProgramError::InvalidAccountData,
        "AVS already exists in node operator AVS list",
    )?;

    node_operator_avs_list.save_with_realloc(&Rent::get()?, admin.account())?;

    Ok(())
}
