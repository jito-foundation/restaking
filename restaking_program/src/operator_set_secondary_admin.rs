use jito_bytemuck::AccountDeserialize;
use jito_jsm_core::loader::load_signer;
use jito_restaking_core::operator::Operator;
use jito_restaking_sdk::{error::RestakingError, instruction::OperatorAdminRole};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
    pubkey::Pubkey,
};

/// The node operator admin can set a new voter for the node operator.
/// This method is permissioned to the node operator admin.
///
/// [`crate::RestakingInstruction::OperatorSetSecondaryAdmin`]
pub fn process_set_operator_secondary_admin(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    role: OperatorAdminRole,
) -> ProgramResult {
    let [operator, admin, new_admin] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    Operator::load(program_id, operator, true)?;
    load_signer(admin, false)?;

    // The Operator admin shall be the signer of the transaction
    let mut operator_data = operator.data.borrow_mut();
    let operator = Operator::try_from_slice_unchecked_mut(&mut operator_data)?;
    if operator.admin.ne(admin.key) {
        msg!("Invalid operator admin");
        return Err(RestakingError::OperatorAdminInvalid.into());
    }

    match role {
        OperatorAdminRole::NcnAdmin => {
            operator.ncn_admin = *new_admin.key;
        }
        OperatorAdminRole::VaultAdmin => {
            operator.vault_admin = *new_admin.key;
        }
        OperatorAdminRole::VoterAdmin => {
            operator.voter = *new_admin.key;
        }
        OperatorAdminRole::WithdrawAdmin => {
            operator.withdrawal_admin = *new_admin.key;
        }
        OperatorAdminRole::WithdrawWallet => {
            operator.withdrawal_fee_wallet = *new_admin.key;
        }
    }

    Ok(())
}
