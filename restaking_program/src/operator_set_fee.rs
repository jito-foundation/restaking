use jito_bytemuck::AccountDeserialize;
use jito_jsm_core::loader::load_signer;
use jito_restaking_core::{operator::Operator, MAX_FEE_BPS};
use jito_restaking_sdk::error::RestakingError;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
    pubkey::Pubkey,
};

/// The node operator admin can set a new fee. This method is permissioned to the node operator admin.
///
/// [`crate::RestakingInstruction::OperatorSetFee`]
pub fn process_operator_set_fee(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    new_fee_bps: u16,
) -> ProgramResult {
    let [_config, operator_account, admin] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    Operator::load(program_id, operator_account, true)?;
    load_signer(admin, false)?;

    let mut operator_data = operator_account.try_borrow_mut_data()?;
    let operator = Operator::try_from_slice_unchecked_mut(&mut operator_data)?;
    operator.check_admin(admin.key)?;

    // Check that the new fee is not greater than the maximum allowed
    if new_fee_bps > MAX_FEE_BPS {
        msg!("New fee exceeds maximum allowed fee");
        return Err(RestakingError::OperatorFeeCapExceeded.into());
    }

    operator.operator_fee_bps = new_fee_bps.into();

    msg!("Operator fee updated to {} basis points", new_fee_bps);

    Ok(())
}
