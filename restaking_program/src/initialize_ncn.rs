use std::mem::size_of;

use jito_account_traits::{AccountDeserialize, Discriminator};
use jito_jsm_core::{
    create_account,
    loader::{load_signer, load_system_account, load_system_program},
};
use jito_restaking_core::{config::Config, loader::load_config, ncn::Ncn};
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
    load_config(program_id, config, true)?;
    load_system_account(ncn, true)?;
    load_signer(admin, true)?;
    load_signer(base, false)?;
    load_system_program(system_program)?;

    let (ncn_pubkey, ncn_bump, mut ncn_seed) = Ncn::find_program_address(program_id, base.key);
    ncn_seed.push(vec![ncn_bump]);
    if ncn.key.ne(&ncn_pubkey) {
        msg!("NCN account is not at the correct PDA");
        return Err(ProgramError::InvalidAccountData);
    }

    msg!("Initializing NCN at address {}", ncn.key);
    create_account(
        admin,
        config,
        system_program,
        program_id,
        &Rent::get()?,
        (8 + size_of::<Ncn>()) as u64,
        &ncn_seed,
    )?;

    let mut config_data = config.data.borrow_mut();
    let config = Config::try_from_slice_mut(&mut config_data)?;

    let mut ncn_data = ncn.try_borrow_mut_data()?;
    ncn_data[0] = Ncn::DISCRIMINATOR;
    let ncn = Ncn::try_from_slice_mut(&mut ncn_data)?;
    ncn.base = *base.key;
    ncn.admin = *admin.key;
    ncn.operator_admin = *admin.key;
    ncn.vault_admin = *admin.key;
    ncn.slasher_admin = *admin.key;
    ncn.withdraw_admin = *admin.key;
    ncn.withdraw_fee_wallet = *admin.key;
    ncn.index = config.ncn_count;
    ncn.operator_count = 0;
    ncn.vault_count = 0;
    ncn.slasher_count = 0;
    ncn.bump = ncn_bump;

    config.ncn_count = config
        .ncn_count
        .checked_add(1)
        .ok_or(ProgramError::InvalidAccountData)?;

    Ok(())
}
