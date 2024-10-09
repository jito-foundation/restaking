use jito_bytemuck::AccountDeserialize;
use jito_jsm_core::loader::load_signer;
use jito_restaking_core::ncn::Ncn;
use jito_restaking_sdk::{error::RestakingError, instruction::NcnAdminRole};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
    pubkey::Pubkey,
};

pub fn process_ncn_set_secondary_admin(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    role: NcnAdminRole,
) -> ProgramResult {
    let [ncn, admin, new_admin] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    Ncn::load(program_id, ncn, true)?;
    load_signer(admin, false)?;

    // The Ncn admin shall be the signer of the transaction
    let mut ncn_data = ncn.data.borrow_mut();
    let ncn = Ncn::try_from_slice_unchecked_mut(&mut ncn_data)?;
    if ncn.admin.ne(admin.key) {
        msg!("Invalid admin for NCN");
        return Err(RestakingError::NcnAdminInvalid.into());
    }

    match role {
        NcnAdminRole::OperatorAdmin => {
            ncn.operator_admin = *new_admin.key;
        }
        NcnAdminRole::VaultAdmin => {
            ncn.vault_admin = *new_admin.key;
        }
        NcnAdminRole::SlasherAdmin => {
            ncn.slasher_admin = *new_admin.key;
        }
        NcnAdminRole::DelegateAdmin => {
            ncn.delegate_admin = *new_admin.key;
        }
        NcnAdminRole::MetadataAdmin => {
            ncn.metadata_admin = *new_admin.key;
        }
    }

    Ok(())
}
