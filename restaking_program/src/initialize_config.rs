use std::mem::size_of;

use jito_account_traits::{AccountDeserialize, Discriminator};
use jito_jsm_core::{
    create_account,
    loader::{load_signer, load_system_account, load_system_program},
};
use jito_restaking_core::config::Config;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, epoch_schedule::DEFAULT_SLOTS_PER_EPOCH,
    msg, program_error::ProgramError, pubkey::Pubkey, rent::Rent, sysvar::Sysvar,
};

/// Initializes the global configuration for the restaking program
/// [`crate::RestakingInstruction::InitializeConfig`]
pub fn process_initialize_config(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let [config, admin, vault_program, system_program] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    load_system_account(config, true)?;
    load_signer(admin, true)?;
    load_system_program(system_program)?;

    let (config_pubkey, config_bump, mut config_seeds) = Config::find_program_address(program_id);
    config_seeds.push(vec![config_bump]);
    if config.key.ne(&config_pubkey) {
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
        (8 + size_of::<Config>()) as u64,
        &config_seeds,
    )?;

    let mut config_data = config.try_borrow_mut_data()?;
    config_data[0] = Config::DISCRIMINATOR;
    let config = Config::try_from_slice_mut(&mut config_data)?;
    config.admin = *admin.key;
    config.vault_program = *vault_program.key;
    config.ncn_count = 0;
    config.operator_count = 0;
    config.epoch_length = DEFAULT_SLOTS_PER_EPOCH;
    config.bump = config_bump;

    Ok(())
}
