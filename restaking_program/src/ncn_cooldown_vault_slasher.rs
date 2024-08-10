use jito_account_traits::AccountDeserialize;
use jito_jsm_core::loader::load_signer;
use jito_restaking_core::{
    config::Config,
    loader::{load_ncn, load_ncn_vault_slasher_ticket},
    ncn::Ncn,
    ncn_vault_slasher_ticket::NcnVaultSlasherTicket,
};
use jito_vault_core::loader::{load_config, load_vault};
use solana_program::{
    account_info::AccountInfo, clock::Clock, entrypoint::ProgramResult, msg,
    program_error::ProgramError, pubkey::Pubkey, sysvar::Sysvar,
};

pub fn process_ncn_remove_slasher(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let [config, ncn, vault, slasher, ncn_vault_slasher_ticket, ncn_slasher_admin] = accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    load_config(program_id, config, false)?;
    load_ncn(program_id, ncn, false)?;
    let mut config_data = config.data.borrow_mut();
    let config = Config::try_from_slice_mut(&mut config_data)?;
    load_vault(&config.vault_program, vault, false)?;
    load_ncn_vault_slasher_ticket(
        program_id,
        ncn_vault_slasher_ticket,
        ncn,
        vault,
        slasher,
        true,
    )?;
    load_signer(ncn_slasher_admin, false)?;

    let ncn_data = ncn.data.borrow();
    let ncn = Ncn::try_from_slice(&ncn_data)?;
    if ncn.slasher_admin.ne(ncn_slasher_admin.key) {
        msg!("Invalid slasher admin for NCN");
        return Err(ProgramError::InvalidAccountData);
    }

    let mut ncn_vault_slasher_ticket_data = ncn_vault_slasher_ticket.data.borrow_mut();
    let ncn_vault_slasher_ticket =
        NcnVaultSlasherTicket::try_from_slice_mut(&mut ncn_vault_slasher_ticket_data)?;

    if !ncn_vault_slasher_ticket
        .state
        .deactivate(Clock::get()?.slot, config.epoch_length)
    {
        msg!("Slasher is not ready to be deactivated");
        return Err(ProgramError::InvalidAccountData);
    }

    Ok(())
}
