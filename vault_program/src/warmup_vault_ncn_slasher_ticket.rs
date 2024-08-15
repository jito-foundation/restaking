use jito_account_traits::AccountDeserialize;
use jito_jsm_core::loader::load_signer;
use jito_restaking_core::{
    loader::{load_ncn, load_ncn_vault_slasher_ticket},
    ncn_vault_slasher_ticket::NcnVaultSlasherTicket,
};
use jito_vault_core::{
    config::Config,
    loader::{load_config, load_vault, load_vault_ncn_slasher_ticket},
    vault::Vault,
    vault_ncn_slasher_ticket::VaultNcnSlasherTicket,
};
use jito_vault_sdk::error::VaultError;
use solana_program::{
    account_info::AccountInfo, clock::Clock, entrypoint::ProgramResult, msg,
    program_error::ProgramError, pubkey::Pubkey, sysvar::Sysvar,
};

/// Instruction: [`crate::VaultInstruction::WarmupVaultNcnSlasherTicket`]
pub fn process_warmup_vault_ncn_slasher_ticket(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let [config, vault, ncn, slasher, ncn_vault_slasher_ticket, vault_ncn_slasher_ticket, vault_slasher_admin] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    load_config(program_id, config, false)?;
    load_vault(program_id, vault, false)?;
    let config_data = config.data.borrow();
    let config = Config::try_from_slice(&config_data)?;
    load_ncn(&config.restaking_program, ncn, false)?;
    load_ncn_vault_slasher_ticket(
        &config.restaking_program,
        ncn_vault_slasher_ticket,
        ncn,
        vault,
        slasher,
        false,
    )?;
    load_vault_ncn_slasher_ticket(
        program_id,
        vault_ncn_slasher_ticket,
        vault,
        ncn,
        slasher,
        false,
    )?;
    load_signer(vault_slasher_admin, false)?;

    // The Vault slasher admin shall be the signer of the transaction
    let vault_data = vault.data.borrow();
    let vault = Vault::try_from_slice(&vault_data)?;
    if vault.slasher_admin.ne(vault_slasher_admin.key) {
        msg!("Invalid slasher admin for vault");
        return Err(VaultError::VaultSlasherAdminInvalid.into());
    }

    // The Vault shall be up-to-date before warming up the slasher
    if vault.is_update_needed(Clock::get()?.slot, config.epoch_length) {
        msg!("Vault update is needed");
        return Err(VaultError::VaultUpdateNeeded.into());
    }

    // The NcnVaultSlasherTicket shall be active
    let ncn_vault_slasher_ticket_data = ncn_vault_slasher_ticket.data.borrow();
    let ncn_vault_slasher_ticket =
        NcnVaultSlasherTicket::try_from_slice(&ncn_vault_slasher_ticket_data)?;
    if !ncn_vault_slasher_ticket
        .state
        .is_active(Clock::get()?.slot, config.epoch_length)
    {
        msg!("NcnVaultSlasherTicket is not active");
        return Err(VaultError::NcnVaultSlasherTicketNotActive.into());
    }

    // The VaultNcnSlasherTicket shall be ready to be activated
    let mut vault_ncn_slasher_ticket_data = vault_ncn_slasher_ticket.data.borrow_mut();
    let vault_ncn_slasher_ticket =
        VaultNcnSlasherTicket::try_from_slice_mut(&mut vault_ncn_slasher_ticket_data)?;
    if !vault_ncn_slasher_ticket
        .state
        .activate(Clock::get()?.slot, config.epoch_length)
    {
        msg!("Slasher is not ready to be activated");
        return Err(VaultError::VaultNcnSlasherTicketFailedWarmup.into());
    }

    Ok(())
}
