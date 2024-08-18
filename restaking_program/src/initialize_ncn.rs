use std::mem::size_of;

use jito_account_traits::{AccountDeserialize, Discriminator};
use jito_jsm_core::{
    create_account,
    loader::{load_signer, load_system_account, load_system_program},
};
use jito_restaking_core::{config::Config, ncn::Ncn};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
    pubkey::Pubkey, rent::Rent, sysvar::Sysvar,
};

/// Initializes an NCN and associated accounts
/// [`crate::RestakingInstruction::InitializeNcn`]
pub fn process_initialize_ncn(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let [config, ncn, admin, base, system_program] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    Config::load(program_id, config, true)?;
    load_system_account(ncn, true)?;
    load_signer(admin, true)?;
    load_signer(base, false)?;
    load_system_program(system_program)?;

    // The Ncn shall be at the canonical PDA
    let (ncn_pubkey, ncn_bump, mut ncn_seeds) = Ncn::find_program_address(program_id, base.key);
    ncn_seeds.push(vec![ncn_bump]);
    if ncn.key.ne(&ncn_pubkey) {
        msg!("NCN account is not at the correct PDA");
        return Err(ProgramError::InvalidAccountData);
    }

    msg!("Initializing NCN at address {}", ncn.key);
    create_account(
        admin,
        ncn,
        system_program,
        program_id,
        &Rent::get()?,
        8_u64.checked_add(size_of::<Ncn>() as u64).unwrap(),
        &ncn_seeds,
    )?;

    let mut config_data = config.data.borrow_mut();
    let config = Config::try_from_slice_unchecked_mut(&mut config_data)?;

    let mut ncn_data = ncn.try_borrow_mut_data()?;
    ncn_data[0] = Ncn::DISCRIMINATOR;
    let ncn = Ncn::try_from_slice_unchecked_mut(&mut ncn_data)?;
    *ncn = Ncn::new(*base.key, *admin.key, config.ncn_count, ncn_bump);

    config.ncn_count = config
        .ncn_count
        .checked_add(1)
        .ok_or(ProgramError::InvalidAccountData)?;

    Ok(())
}
