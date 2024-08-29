use std::mem::size_of;

use jito_bytemuck::{AccountDeserialize, Discriminator};
use jito_jsm_core::{
    create_account,
    loader::{load_signer, load_system_account, load_system_program},
};
use jito_restaking_core::{ncn::Ncn, ncn_vault_slasher_ticket::NcnVaultSlasherTicket};
use jito_vault_core::{
    config::Config, vault::Vault, vault_ncn_slasher_ticket::VaultNcnSlasherTicket,
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

    Config::load(program_id, config, false)?;
    let config_data = config.data.borrow();
    let config = Config::try_from_slice_unchecked(&config_data)?;
    Vault::load(program_id, vault_info, false)?;
    let mut vault_data = vault_info.data.borrow_mut();
    let vault = Vault::try_from_slice_unchecked_mut(&mut vault_data)?;
    Ncn::load(&config.restaking_program, ncn, false)?;
    NcnVaultSlasherTicket::load(
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

    // The VaultNcnSlasherTicket shall be at the canonical PDA
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

    let slot = Clock::get()?.slot;

    vault.check_slasher_admin(vault_slasher_admin.key)?;
    vault.check_update_state_ok(slot, config.epoch_length())?;

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

    let ncn_vault_slasher_ticket_data = ncn_slasher_ticket.data.borrow();
    let ncn_vault_slasher_ticket =
        NcnVaultSlasherTicket::try_from_slice_unchecked(&ncn_vault_slasher_ticket_data)?;

    let mut vault_ncn_slasher_ticket_data = vault_ncn_slasher_ticket.try_borrow_mut_data()?;
    vault_ncn_slasher_ticket_data[0] = VaultNcnSlasherTicket::DISCRIMINATOR;
    let vault_ncn_slasher_ticket =
        VaultNcnSlasherTicket::try_from_slice_unchecked_mut(&mut vault_ncn_slasher_ticket_data)?;
    *vault_ncn_slasher_ticket = VaultNcnSlasherTicket::new(
        *vault_info.key,
        *ncn.key,
        *slasher.key,
        ncn_vault_slasher_ticket.max_slashable_per_epoch(),
        vault.slasher_count(),
        vault_ncn_slasher_ticket_bump,
        slot,
    );

    vault.increment_slasher_count()?;

    Ok(())
}
