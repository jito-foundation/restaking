use jito_bytemuck::AccountDeserialize;
use jito_jsm_core::loader::load_signer;
use jito_restaking_core::{ncn::Ncn, operator::Operator};
use jito_restaking_sdk::error::RestakingError;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
    pubkey::Pubkey, rent::Rent, sysvar::Sysvar,
};

/// Withdraw Excess Lamports is used to recover Lamports transferred to any
/// TokenProgram owned account by moving them to another account
/// of the source account.
pub fn process_withdraw_excess_lamports(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let [source_info, destination_info, admin] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    load_signer(admin, false)?;

    let source_data = source_info.data.borrow();
    if let Ok(ncn) = Ncn::try_from_slice_unchecked(&source_data) {
        ncn.check_admin(admin.key)?;
    } else if let Ok(operator) = Operator::try_from_slice_unchecked(&source_data) {
        operator.check_admin(admin.key)?;
    } else {
        msg!("Source info can not be deserialized");
        return Err(ProgramError::InvalidAccountData);
    }

    let source_rent_exempt_reserve = Rent::get()?.minimum_balance(source_info.data_len());

    let transfer_amount = source_info
        .lamports()
        .checked_sub(source_rent_exempt_reserve)
        .ok_or(RestakingError::NotRentExempt)?;

    let source_starting_lamports = source_info.lamports();
    **source_info.lamports.borrow_mut() = source_starting_lamports
        .checked_sub(transfer_amount)
        .ok_or(RestakingError::VaultOverflow)?;

    let destination_starting_lamports = destination_info.lamports();
    **destination_info.lamports.borrow_mut() = destination_starting_lamports
        .checked_add(transfer_amount)
        .ok_or(RestakingError::VaultOverflow)?;

    Ok(())
}
