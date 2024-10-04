use jito_bytemuck::AccountDeserialize;
use jito_jsm_core::loader::load_signer;
use jito_restaking_core::operator::Operator;
use jito_restaking_sdk::error::RestakingError;
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

    Operator::load(program_id, operator, true)?;
    load_signer(old_admin, false)?;
    load_signer(new_admin, false)?;

    // The Operator admin shall be the signer of the transaction
    let mut operator_data = operator.data.borrow_mut();
    let operator = Operator::try_from_slice_unchecked_mut(&mut operator_data)?;
    if operator.admin.ne(old_admin.key) {
        msg!("Invalid operator admin");
        return Err(RestakingError::OperatorAdminInvalid.into());
    }

    operator.admin = *new_admin.key;

    operator.update_secondary_admin(old_admin.key, new_admin.key);

    Ok(())
}
