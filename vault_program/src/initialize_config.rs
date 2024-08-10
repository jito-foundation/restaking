use std::mem::size_of;

use jito_account_traits::{AccountDeserialize, Discriminator};
use jito_jsm_core::{
    create_account,
    loader::{load_signer, load_system_account, load_system_program},
};
use jito_vault_core::config::Config;
use solana_program::{
    account_info::AccountInfo, clock::DEFAULT_SLOTS_PER_EPOCH, entrypoint::ProgramResult, msg,
    program_error::ProgramError, pubkey::Pubkey, rent::Rent, sysvar::Sysvar,
};

/// Processes the initialize config instruction: [`crate::VaultInstruction::InitializeConfig`]
pub fn process_initialize_config(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let [config, admin, restaking_program, system_program] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    load_system_account(config, true)?;
    load_signer(admin, true)?;
    load_system_program(system_program)?;

    let (config_pubkey, config_bump, mut config_seeds) = Config::find_program_address(program_id);
    config_seeds.push(vec![config_bump]);
    if config_pubkey.ne(config.key) {
        msg!("Config account is not at the correct PDA");
        return Err(ProgramError::InvalidAccountData);
    }

    msg!("Initializing config at address {}", config.key);
    create_account(
        admin,
        config,
        system_program,
        program_id,
        &Rent::get()?,
        8_u64.checked_add(size_of::<Config>() as u64).unwrap(),
        &config_seeds,
    )?;

    let mut config_data = config.try_borrow_mut_data()?;
    config_data[0] = Config::DISCRIMINATOR;
    let config = Config::try_from_slice_mut(&mut config_data)?;
    config.admin = *admin.key;
    config.restaking_program = *restaking_program.key;
    config.epoch_length = DEFAULT_SLOTS_PER_EPOCH;
    config.num_vaults = 0;
    config.bump = config_bump;

    Ok(())
}
