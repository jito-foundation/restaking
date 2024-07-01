use jito_restaking_core::operator::SanitizedOperator;
use jito_restaking_sanitization::{assert_with_msg, signer::SanitizedSignerAccount};
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
    let accounts_iter = &mut accounts.iter();

    let mut node_operator =
        SanitizedOperator::sanitize(program_id, next_account_info(accounts_iter)?, true)?;
    let admin = SanitizedSignerAccount::sanitize(next_account_info(accounts_iter)?, true)?;
    let voter = next_account_info(accounts_iter)?;

    assert_with_msg(
        node_operator.operator().admin() == *admin.account().key,
        ProgramError::InvalidAccountData,
        "Admin is not the node operator admin",
    )?;

    node_operator.operator_mut().set_voter(*voter.key);

    node_operator.save()?;

    Ok(())
}
