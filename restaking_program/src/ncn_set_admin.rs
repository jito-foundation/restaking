use jito_account_traits::AccountDeserialize;
use jito_jsm_core::loader::load_signer;
use jito_restaking_core::{loader::load_ncn, ncn::Ncn};
use jito_restaking_sdk::error::RestakingError;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
    pubkey::Pubkey,
};

pub fn process_ncn_set_admin(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let [ncn, old_admin, new_admin] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    load_ncn(program_id, ncn, true)?;
    load_signer(old_admin, false)?;
    load_signer(new_admin, false)?;

    // The Ncn admin shall be the signer of the transaction
    let mut ncn_data = ncn.data.borrow_mut();
    let ncn = Ncn::try_from_slice_mut(&mut ncn_data)?;
    if ncn.admin.ne(old_admin.key) {
        msg!("Invalid admin for NCN");
        return Err(RestakingError::NcnAdminInvalid.into());
    }

    ncn.admin = *new_admin.key;

    Ok(())
}
