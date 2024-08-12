use std::mem::size_of;

use jito_account_traits::{AccountDeserialize, Discriminator};
use jito_jsm_core::{
    create_account,
    loader::{load_signer, load_system_account, load_system_program},
};
use jito_restaking_core::{
    config::Config,
    loader::{load_config, load_ncn},
    ncn::Ncn,
    ncn_vault_ticket::NcnVaultTicket,
};
use jito_restaking_sdk::error::RestakingError;
use jito_vault_core::loader::load_vault;
use solana_program::{
    account_info::AccountInfo, clock::Clock, entrypoint::ProgramResult, msg,
    program_error::ProgramError, pubkey::Pubkey, rent::Rent, sysvar::Sysvar,
};

/// The NCN opts-in to vaults by storing the vault in the NCN vault list. It also CPI's into
/// the vault program and adds the NCN to the vault's NCN list.
///
/// [`crate::RestakingInstruction::InitializeNcnVaultTicket`]
pub fn process_initialize_ncn_vault_ticket(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let [config, ncn_info, vault, ncn_vault_ticket, ncn_vault_admin, payer, system_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    load_config(program_id, config, false)?;
    load_ncn(program_id, ncn_info, true)?;
    let config_data = config.data.borrow();
    let config = Config::try_from_slice(&config_data)?;
    load_vault(&config.vault_program, vault, false)?;
    load_system_account(ncn_vault_ticket, true)?;
    load_signer(ncn_vault_admin, false)?;
    load_signer(payer, true)?;
    load_system_program(system_program)?;

    let slot = Clock::get()?.slot;

    // The NcnVaultTicket shall be at the canonical PDA
    let (ncn_vault_ticket_pubkey, ncn_vault_ticket_bump, mut ncn_vault_ticket_seeds) =
        NcnVaultTicket::find_program_address(program_id, ncn_info.key, vault.key);
    ncn_vault_ticket_seeds.push(vec![ncn_vault_ticket_bump]);
    if ncn_vault_ticket_pubkey.ne(ncn_vault_ticket.key) {
        msg!("NCN vault ticket is not at the correct PDA");
        return Err(ProgramError::InvalidAccountData);
    }

    let mut ncn_data = ncn_info.data.borrow_mut();
    let ncn = Ncn::try_from_slice_mut(&mut ncn_data)?;
    if ncn.vault_admin.ne(ncn_vault_admin.key) {
        msg!("Invalid vault admin for NCN");
        return Err(RestakingError::NcnVaultAdminInvalid.into());
    }

    msg!(
        "Initializing NcnVaultTicket at address {}",
        ncn_vault_ticket.key
    );
    create_account(
        payer,
        ncn_vault_ticket,
        system_program,
        program_id,
        &Rent::get()?,
        8_u64
            .checked_add(size_of::<NcnVaultTicket>() as u64)
            .unwrap(),
        &ncn_vault_ticket_seeds,
    )?;

    let mut ncn_vault_ticket_data = ncn_vault_ticket.try_borrow_mut_data()?;
    ncn_vault_ticket_data[0] = NcnVaultTicket::DISCRIMINATOR;
    let ncn_vault_ticket = NcnVaultTicket::try_from_slice_mut(&mut ncn_vault_ticket_data)?;
    *ncn_vault_ticket = NcnVaultTicket::new(
        *ncn_info.key,
        *vault.key,
        ncn.vault_count,
        slot,
        ncn_vault_ticket_bump,
    );

    ncn.vault_count = ncn
        .vault_count
        .checked_add(1)
        .ok_or(ProgramError::InvalidAccountData)?;

    Ok(())
}
