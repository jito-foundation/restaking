use std::mem::size_of;

use jito_account_traits::{AccountDeserialize, Discriminator};
use jito_jsm_core::{
    create_account,
    loader::{load_signer, load_system_account, load_system_program},
};
use jito_restaking_core::{
    loader::{load_operator, load_operator_vault_ticket},
    operator_vault_ticket::OperatorVaultTicket,
};
use jito_vault_core::{
    config::Config,
    loader::{load_config, load_vault},
    vault::Vault,
    vault_operator_ticket::VaultOperatorTicket,
};
use solana_program::{
    account_info::AccountInfo, clock::Clock, entrypoint::ProgramResult, msg,
    program_error::ProgramError, pubkey::Pubkey, rent::Rent, sysvar::Sysvar,
};

/// Instruction: [`crate::VaultInstruction::InitializeVaultOperatorTicket`]
pub fn process_initialize_vault_operator_ticket(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let [config, vault_info, operator, operator_vault_ticket, vault_operator_ticket, vault_operator_admin, payer, system_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    load_config(program_id, config, false)?;
    load_vault(program_id, vault_info, false)?;
    let config_data = config.data.borrow();
    let config = Config::try_from_slice(&config_data)?;
    load_operator(&config.restaking_program, operator, false)?;
    load_operator_vault_ticket(
        &config.restaking_program,
        operator_vault_ticket,
        operator,
        vault_info,
        false,
    )?;
    load_system_account(vault_operator_ticket, true)?;
    load_signer(vault_operator_admin, false)?;
    load_signer(payer, true)?;
    load_system_program(system_program)?;

    let (vault_operator_ticket_pubkey, vault_operator_ticket_bump, mut vault_operator_ticket_seeds) =
        VaultOperatorTicket::find_program_address(program_id, vault_info.key, operator.key);
    vault_operator_ticket_seeds.push(vec![vault_operator_ticket_bump]);
    if vault_operator_ticket_pubkey.ne(vault_operator_ticket.key) {
        msg!("Vault operator ticket is not at the correct PDA");
        return Err(ProgramError::InvalidAccountData);
    }

    let mut vault_data = vault_info.data.borrow_mut();
    let vault = Vault::try_from_slice_mut(&mut vault_data)?;
    if vault.operator_admin.ne(vault_operator_admin.key) {
        msg!("Invalid operator admin for vault");
        return Err(ProgramError::InvalidAccountData);
    }

    let operator_vault_ticket_data = operator_vault_ticket.data.borrow();
    let operator_vault_ticket = OperatorVaultTicket::try_from_slice(&operator_vault_ticket_data)?;
    if !operator_vault_ticket
        .state
        .is_active_or_cooldown(Clock::get()?.slot, config.epoch_length)
    {
        msg!("Operator vault ticket is not active or in cooldown");
        return Err(ProgramError::InvalidAccountData);
    }

    msg!(
        "Initializing VaultOperatorTicket at address {}",
        vault_operator_ticket.key
    );
    create_account(
        payer,
        vault_operator_ticket,
        system_program,
        program_id,
        &Rent::get()?,
        8_u64
            .checked_add(size_of::<VaultOperatorTicket>() as u64)
            .unwrap(),
        &vault_operator_ticket_seeds,
    )?;

    let mut vault_operator_ticket_data = vault_operator_ticket.try_borrow_mut_data()?;
    vault_operator_ticket_data[0] = VaultOperatorTicket::DISCRIMINATOR;
    let vault_operator_ticket =
        VaultOperatorTicket::try_from_slice_mut(&mut vault_operator_ticket_data)?;
    *vault_operator_ticket = VaultOperatorTicket::new(
        *vault_info.key,
        *operator.key,
        vault.operator_count,
        Clock::get()?.slot,
        vault_operator_ticket_bump,
    );

    vault.operator_count = vault
        .operator_count
        .checked_add(1)
        .ok_or(ProgramError::InvalidAccountData)?;

    Ok(())
}
