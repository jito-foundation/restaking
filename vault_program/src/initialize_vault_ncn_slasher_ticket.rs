use std::mem::size_of;

use jito_account_traits::{AccountDeserialize, Discriminator};
use jito_jsm_core::{
    create_account,
    loader::{load_signer, load_system_account, load_system_program},
};
use jito_restaking_core::{
    loader::{load_config, load_ncn, load_ncn_vault_slasher_ticket},
    ncn_vault_slasher_ticket::NcnVaultSlasherTicket,
};
use jito_vault_core::{
    config::Config, loader::load_vault, vault::Vault,
    vault_ncn_slasher_ticket::VaultNcnSlasherTicket,
};
use solana_program::{
    account_info::AccountInfo, clock::Clock, entrypoint::ProgramResult, msg,
    program_error::ProgramError, pubkey::Pubkey, rent::Rent, sysvar::Sysvar,
};

/// Processes the register slasher instruction: [`crate::VaultInstruction::InitializeVaultNcnSlasherTicket`]
pub fn process_initialize_vault_ncn_slasher_ticket(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let [config, vault_info, ncn, slasher, ncn_slasher_ticket, vault_ncn_slasher_ticket, vault_slasher_admin, payer, system_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    load_config(program_id, config, false)?;
    load_vault(program_id, vault_info, false)?;
    let mut config_data = config.data.borrow_mut();
    let config = Config::try_from_slice_mut(&mut config_data)?;
    load_ncn(&config.restaking_program, ncn, false)?;
    load_ncn_vault_slasher_ticket(
        &config.restaking_program,
        ncn_slasher_ticket,
        ncn,
        vault_info,
        slasher,
        false,
    )?;
    load_system_account(vault_ncn_slasher_ticket, true)?;
    load_signer(vault_slasher_admin, false)?;
    load_signer(payer, true)?;
    load_system_program(system_program)?;

    let (
        vault_ncn_slasher_ticket_pubkey,
        vault_ncn_slasher_ticket_bump,
        mut vault_ncn_slasher_ticket_seeds,
    ) = VaultNcnSlasherTicket::find_program_address(
        program_id,
        vault_info.key,
        ncn.key,
        slasher.key,
    );
    vault_ncn_slasher_ticket_seeds.push(vec![vault_ncn_slasher_ticket_bump]);
    if vault_ncn_slasher_ticket_pubkey.ne(vault_ncn_slasher_ticket.key) {
        msg!("Vault NCN slasher ticket is not at the correct PDA");
        return Err(ProgramError::InvalidAccountData);
    }

    let mut vault_data = vault_info.data.borrow_mut();
    let vault = Vault::try_from_slice_mut(&mut vault_data)?;
    if vault.slasher_admin.ne(vault_slasher_admin.key) {
        msg!("Invalid slasher admin for vault");
        return Err(ProgramError::InvalidAccountData);
    }

    let ncn_vault_slasher_ticket_data = ncn_slasher_ticket.data.borrow();
    let ncn_vault_slasher_ticket =
        NcnVaultSlasherTicket::try_from_slice(&ncn_vault_slasher_ticket_data)?;
    if !ncn_vault_slasher_ticket
        .state
        .is_active_or_cooldown(Clock::get()?.slot, config.epoch_length)
    {
        msg!("Slasher is not ready to be activated");
        return Err(ProgramError::InvalidAccountData);
    }

    msg!(
        "Initializing VaultNcnSlasherTicket at address {}",
        vault_ncn_slasher_ticket.key
    );
    create_account(
        payer,
        vault_ncn_slasher_ticket,
        system_program,
        program_id,
        &Rent::get()?,
        8_u64
            .checked_add(size_of::<VaultNcnSlasherTicket>() as u64)
            .unwrap(),
        &vault_ncn_slasher_ticket_seeds,
    )?;

    let mut vault_ncn_slasher_ticket_data = vault_ncn_slasher_ticket.try_borrow_mut_data()?;
    vault_ncn_slasher_ticket_data[0] = VaultNcnSlasherTicket::DISCRIMINATOR;
    let vault_ncn_slasher_ticket =
        VaultNcnSlasherTicket::try_from_slice_mut(&mut vault_ncn_slasher_ticket_data)?;
    *vault_ncn_slasher_ticket = VaultNcnSlasherTicket::new(
        *vault_info.key,
        *ncn.key,
        *slasher.key,
        ncn_vault_slasher_ticket.max_slashable_per_epoch,
        vault.slasher_count,
        Clock::get()?.slot,
        vault_ncn_slasher_ticket_bump,
    );

    vault.slasher_count = vault
        .slasher_count
        .checked_add(1)
        .ok_or(ProgramError::InvalidAccountData)?;

    Ok(())
}
