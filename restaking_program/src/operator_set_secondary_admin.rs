use jito_account_traits::AccountDeserialize;
use jito_jsm_core::loader::load_signer;
use jito_restaking_core::{loader::load_operator, operator::Operator};
use jito_restaking_sdk::OperatorAdminRole;
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
    load_operator(program_id, operator, false)?;
    load_signer(admin, false)?;

    let mut operator_data = operator.data.borrow_mut();
    let operator = Operator::try_from_slice_mut(&mut operator_data)?;
    if operator.admin.ne(&admin.key) {
        msg!("Invalid operator admin");
        return Err(ProgramError::InvalidAccountData);
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
            operator.withdraw_admin = *new_admin.key;
        }
        OperatorAdminRole::WithdrawWallet => {
            operator.withdraw_fee_wallet = *new_admin.key;
        }
    }

    Ok(())
}
