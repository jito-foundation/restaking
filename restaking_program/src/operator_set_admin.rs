use jito_restaking_core::node_operator::SanitizedNodeOperator;
use jito_restaking_sanitization::{assert_with_msg, signer::SanitizedSignerAccount};
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
    let accounts_iter = &mut accounts.iter();

    let mut node_operator =
        SanitizedNodeOperator::sanitize(program_id, next_account_info(accounts_iter)?, true)?;
    let old_admin = SanitizedSignerAccount::sanitize(next_account_info(accounts_iter)?, false)?;
    let new_admin = SanitizedSignerAccount::sanitize(next_account_info(accounts_iter)?, true)?;

    assert_with_msg(
        node_operator.node_operator().admin() == *old_admin.account().key,
        ProgramError::InvalidAccountData,
        "Old admin is not the node operator admin",
    )?;

    node_operator
        .node_operator_mut()
        .set_admin(*new_admin.account().key);

    node_operator.save()?;

    Ok(())
}
