use std::mem::size_of;

use jito_account_traits::{AccountDeserialize, Discriminator};
use jito_jsm_core::{
    create_account,
    loader::{load_signer, load_system_account, load_system_program},
};
use jito_restaking_core::{
    loader::{load_ncn, load_ncn_vault_ticket},
    ncn_vault_ticket::NcnVaultTicket,
};
use jito_vault_core::{
    config::Config,
    loader::{load_config, load_vault},
    vault::Vault,
    vault_ncn_ticket::VaultNcnTicket,
};
use solana_program::{
    account_info::AccountInfo, clock::Clock, entrypoint::ProgramResult, msg,
    program_error::ProgramError, pubkey::Pubkey, rent::Rent, sysvar::Sysvar,
};

/// Adds an NCN to the vault NCN list, which means delegation applied to operators staking to the NCN
/// will be applied.
///
/// # Behavior
/// * The vault admin shall have the ability to add support for a new NCN
/// if the NCN is actively supporting the vault
///
/// Instruction: [`crate::VaultInstruction::AddNcn`]
pub fn process_vault_add_ncn(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let [config, vault_info, ncn, ncn_vault_ticket, vault_ncn_ticket, vault_ncn_admin, payer, system_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    load_config(program_id, config, false)?;
    load_vault(program_id, vault_info, true)?;
    let config_data = config.data.borrow();
    let config = Config::try_from_slice(&config_data)?;
    load_ncn(&config.restaking_program, ncn, false)?;
    load_ncn_vault_ticket(
        &config.restaking_program,
        ncn_vault_ticket,
        ncn,
        vault_info,
        false,
    )?;
    load_system_account(vault_ncn_ticket, false)?;
    load_signer(vault_ncn_admin, false)?;
    load_signer(payer, true)?;
    load_system_program(system_program)?;

    let (vault_ncn_ticket_pubkey, vault_ncn_ticket_bump, mut vault_ncn_ticket_seeds) =
        VaultNcnTicket::find_program_address(program_id, vault_info.key, ncn.key);
    vault_ncn_ticket_seeds.push(vec![vault_ncn_ticket_bump]);
    if vault_ncn_ticket_pubkey.ne(vault_ncn_ticket.key) {
        msg!("Vault NCN ticket is not at the correct PDA");
        return Err(ProgramError::InvalidAccountData);
    }

    let mut vault_data = vault_info.data.borrow_mut();
    let vault = Vault::try_from_slice_mut(&mut vault_data)?;
    if vault.ncn_admin.ne(vault_ncn_admin.key) {
        msg!("Invalid NCN admin for vault");
        return Err(ProgramError::InvalidAccountData);
    }

    let ncn_vault_data = ncn_vault_ticket.data.borrow();
    let ncn_vault_ticket = NcnVaultTicket::try_from_slice(&ncn_vault_data)?;
    if !ncn_vault_ticket
        .state
        .is_active_or_cooldown(Clock::get()?.slot, config.epoch_length)
    {
        msg!("NCN vault ticket is not active or in cooldown");
        return Err(ProgramError::InvalidAccountData);
    }

    msg!(
        "Initializing VaultNcnTicket at address {}",
        vault_ncn_ticket.key
    );
    create_account(
        payer,
        vault_ncn_ticket,
        system_program,
        program_id,
        &Rent::get()?,
        8_u64
            .checked_add(size_of::<VaultNcnTicket>() as u64)
            .unwrap(),
        &vault_ncn_ticket_seeds,
    )?;
    let mut vault_ncn_ticket_data = vault_ncn_ticket.try_borrow_mut_data()?;
    vault_ncn_ticket_data[0] = VaultNcnTicket::DISCRIMINATOR;
    let vault_ncn_ticket = VaultNcnTicket::try_from_slice_mut(&mut vault_ncn_ticket_data)?;
    *vault_ncn_ticket = VaultNcnTicket::new(
        *vault_info.key,
        *ncn.key,
        vault.ncn_count,
        Clock::get()?.slot,
        vault_ncn_ticket_bump,
    );

    vault.ncn_count = vault
        .ncn_count
        .checked_add(1)
        .ok_or(ProgramError::InvalidAccountData)?;

    Ok(())
}
