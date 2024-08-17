use std::mem::size_of;

use jito_account_traits::{AccountDeserialize, Discriminator};
use jito_jsm_core::{
    create_account,
    loader::{load_signer, load_system_account, load_system_program},
};
use jito_restaking_core::{
    config::Config,
    loader::{load_config, load_ncn, load_ncn_vault_ticket},
    ncn::Ncn,
    ncn_vault_slasher_ticket::NcnVaultSlasherTicket,
};
use jito_restaking_sdk::error::RestakingError;
use jito_vault_core::loader::load_vault;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
    pubkey::Pubkey, rent::Rent, sysvar::Sysvar,
};

pub fn process_initialize_ncn_vault_slasher_ticket(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    max_slashable_per_epoch: u64,
) -> ProgramResult {
    let [config, ncn_info, vault, slasher, ncn_vault_ticket, ncn_vault_slasher_ticket, ncn_slasher_admin, payer, system_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    load_config(program_id, config, false)?;
    load_ncn(program_id, ncn_info, true)?;
    let mut config_data = config.data.borrow_mut();
    let config = Config::try_from_slice_mut(&mut config_data)?;
    load_vault(&config.vault_program, vault, false)?;
    load_ncn_vault_ticket(program_id, ncn_vault_ticket, ncn_info, vault, false)?;
    load_system_account(ncn_vault_slasher_ticket, true)?;
    load_signer(ncn_slasher_admin, false)?;
    load_signer(payer, true)?;
    load_system_program(system_program)?;

    // The NcnVaultSlasherTicket shall be at the canonical PDA
    let (
        ncn_vault_slasher_ticket_pubkey,
        ncn_vault_slasher_ticket_bump,
        mut ncn_vault_slasher_ticket_seeds,
    ) = NcnVaultSlasherTicket::find_program_address(
        program_id,
        ncn_info.key,
        vault.key,
        slasher.key,
    );
    ncn_vault_slasher_ticket_seeds.push(vec![ncn_vault_slasher_ticket_bump]);
    if ncn_vault_slasher_ticket
        .key
        .ne(&ncn_vault_slasher_ticket_pubkey)
    {
        msg!("Ncn vault slasher ticket account is not at the correct PDA");
        return Err(ProgramError::InvalidAccountData);
    }

    let mut ncn_data = ncn_info.data.borrow_mut();
    let ncn = Ncn::try_from_slice_mut(&mut ncn_data)?;
    if ncn.slasher_admin.ne(ncn_slasher_admin.key) {
        msg!("Admin is not the slasher admin");
        return Err(RestakingError::NcnSlasherAdminInvalid.into());
    }

    msg!(
        "Initializing NcnVaultSlasherTicket at address {}",
        ncn_vault_slasher_ticket.key
    );
    create_account(
        payer,
        ncn_vault_slasher_ticket,
        system_program,
        program_id,
        &Rent::get()?,
        8_u64
            .checked_add(size_of::<NcnVaultSlasherTicket>() as u64)
            .unwrap(),
        &ncn_vault_slasher_ticket_seeds,
    )?;
    let mut ncn_vault_slasher_ticket_data = ncn_vault_slasher_ticket.try_borrow_mut_data()?;
    ncn_vault_slasher_ticket_data[0] = NcnVaultSlasherTicket::DISCRIMINATOR;
    let ncn_vault_slasher_ticket =
        NcnVaultSlasherTicket::try_from_slice_mut(&mut ncn_vault_slasher_ticket_data)?;
    *ncn_vault_slasher_ticket = NcnVaultSlasherTicket::new(
        *ncn_info.key,
        *vault.key,
        *slasher.key,
        max_slashable_per_epoch,
        ncn.slasher_count,
        ncn_vault_slasher_ticket_bump,
    );

    ncn.slasher_count = ncn
        .slasher_count
        .checked_add(1)
        .ok_or(ProgramError::InvalidAccountData)?;

    Ok(())
}
