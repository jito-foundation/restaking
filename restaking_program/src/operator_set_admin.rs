use jito_account_traits::AccountDeserialize;
use jito_jsm_core::loader::load_signer;
use jito_restaking_core::{loader::load_operator, operator::Operator};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
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
    let [operator, old_admin, new_admin] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    load_operator(program_id, operator, false)?;
    load_signer(old_admin, false)?;
    load_signer(new_admin, false)?;

    let mut operator_data = operator.data.borrow_mut();
    let operator = Operator::try_from_slice_mut(&mut operator_data)?;
    if operator.admin.ne(old_admin.key) {
        msg!("Invalid operator admin");
        return Err(ProgramError::InvalidAccountData);
    }

    operator.admin = *new_admin.key;

    Ok(())
}
