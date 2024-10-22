use std::mem::size_of;

use jito_bytemuck::{AccountDeserialize, Discriminator};
use jito_jsm_core::{
    create_account,
    loader::{load_signer, load_system_account, load_system_program},
};
use jito_vault_core::{config::Config, MAX_FEE_BPS};
use jito_vault_sdk::error::VaultError;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
    pubkey::Pubkey, rent::Rent, sysvar::Sysvar,
};

/// Processes the initialize config instruction: [`crate::VaultInstruction::InitializeConfig`]
pub fn process_initialize_config(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    program_fee_bps: u16,
) -> ProgramResult {
    let [config, admin, restaking_program, program_fee_wallet, system_program] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    load_system_account(config, true)?;
    load_signer(admin, true)?;
    load_system_program(system_program)?;

    // The config account shall be at the canonical PDA
    let (config_pubkey, config_bump, mut config_seeds) = Config::find_program_address(program_id);
    config_seeds.push(vec![config_bump]);
    if config_pubkey.ne(config.key) {
        msg!("Config account is not at the correct PDA");
        return Err(ProgramError::InvalidAccountData);
    }

    if program_fee_bps > MAX_FEE_BPS {
        msg!("Program fee exceeds maximum allowed fee");
        return Err(ProgramError::InvalidArgument);
    }

    msg!("Initializing config at address {}", config.key);
    create_account(
        admin,
        config,
        system_program,
        program_id,
        &Rent::get()?,
        8_u64
            .checked_add(size_of::<Config>() as u64)
            .ok_or(VaultError::ArithmeticOverflow)?,
        &config_seeds,
    )?;

    let mut config_data = config.try_borrow_mut_data()?;
    config_data[0] = Config::DISCRIMINATOR;
    let config = Config::try_from_slice_unchecked_mut(&mut config_data)?;
    *config = Config::new(
        *admin.key,
        *restaking_program.key,
        *program_fee_wallet.key,
        program_fee_bps,
        config_bump,
    );

    Ok(())
}
