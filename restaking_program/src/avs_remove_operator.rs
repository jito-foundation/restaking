use jito_restaking_core::{
    avs::SanitizedAvs, avs_operator_list::SanitizedAvsOperatorList, config::SanitizedConfig,
    node_operator::SanitizedNodeOperator,
};
use jito_restaking_sanitization::{assert_with_msg, signer::SanitizedSignerAccount};
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
pub fn process_avs_remove_node_operator(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    let _config = SanitizedConfig::sanitize(program_id, next_account_info(accounts_iter)?, false)?;
    let avs = SanitizedAvs::sanitize(program_id, next_account_info(accounts_iter)?, false)?;
    let mut avs_operator_list = SanitizedAvsOperatorList::sanitize(
        program_id,
        next_account_info(accounts_iter)?,
        true,
        avs.account().key,
    )?;
    let node_operator =
        SanitizedNodeOperator::sanitize(program_id, next_account_info(accounts_iter)?, false)?;

    let admin = SanitizedSignerAccount::sanitize(next_account_info(accounts_iter)?, true)?;

    assert_with_msg(
        avs.avs().admin() == *admin.account().key,
        ProgramError::InvalidAccountData,
        "Admin is not the AVS admin",
    )?;

    let clock = Clock::get()?;
    assert_with_msg(
        avs_operator_list
            .avs_operator_list_mut()
            .remove_operator(*node_operator.account().key, clock.slot),
        ProgramError::InvalidAccountData,
        "Node operator does not exist in AVS operator list",
    )?;

    avs_operator_list.save()?;

    Ok(())
}
