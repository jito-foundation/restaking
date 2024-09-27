use std::mem::size_of;

use jito_bytemuck::{AccountDeserialize, Discriminator};
use jito_jsm_core::{
    create_account,
    loader::{load_signer, load_system_account, load_system_program},
};
use jito_restaking_core::{config::Config, operator::Operator};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
    pubkey::Pubkey, rent::Rent, sysvar::Sysvar,
};

/// Initializes a node operator and associated accounts.
/// [`crate::RestakingInstruction::InitializeOperator`]
pub fn process_initialize_operator(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    operator_fee_bps: u16,
) -> ProgramResult {
    let [config, operator, admin, base, system_program] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    Config::load(program_id, config, true)?;
    load_system_account(operator, true)?;
    load_signer(admin, true)?;
    load_signer(base, false)?;
    load_system_program(system_program)?;

    // The Operator shall be at the canonical PDA
    let (operator_pubkey, operator_bump, mut operator_seed) =
        Operator::find_program_address(program_id, base.key);
    operator_seed.push(vec![operator_bump]);
    if operator.key.ne(&operator_pubkey) {
        msg!("Operator account is not at the correct PDA");
        return Err(ProgramError::InvalidAccountData);
    }

    msg!("Initializing operator at address {}", operator.key);
    create_account(
        admin,
        operator,
        system_program,
        program_id,
        &Rent::get()?,
        8_u64.checked_add(size_of::<Operator>() as u64).unwrap(),
        &operator_seed,
    )?;

    let mut config_data = config.data.borrow_mut();
    let config = Config::try_from_slice_unchecked_mut(&mut config_data)?;

    let mut operator_data = operator.try_borrow_mut_data()?;
    operator_data[0] = Operator::DISCRIMINATOR;
    let operator = Operator::try_from_slice_unchecked_mut(&mut operator_data)?;
    *operator = Operator::new(
        *base.key,
        *admin.key,
        config.operator_count(),
        operator_fee_bps,
        operator_bump,
    );

    config.increment_operator_count()?;

    Ok(())
}
