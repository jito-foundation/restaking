use jito_account_traits::AccountDeserialize;
use jito_jsm_core::loader::load_signer;
use jito_restaking_core::{ncn::Ncn, ncn_vault_slasher_ticket::NcnVaultSlasherTicket};
use jito_vault_core::{
    config::Config, vault::Vault, vault_ncn_slasher_ticket::VaultNcnSlasherTicket,
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
    Config::load(program_id, config, false)?;
    Vault::load(program_id, vault, false)?;
    let config_data = config.data.borrow();
    let config = Config::try_from_slice_unchecked(&config_data)?;
    Ncn::load(&config.restaking_program, ncn, false)?;
    NcnVaultSlasherTicket::load(
        &config.restaking_program,
        ncn_vault_slasher_ticket,
        ncn,
        vault,
        slasher,
        false,
    )?;
    VaultNcnSlasherTicket::load(
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
    let vault = Vault::try_from_slice_unchecked(&vault_data)?;
    if vault.slasher_admin.ne(vault_slasher_admin.key) {
        msg!("Invalid slasher admin for vault");
        return Err(VaultError::VaultSlasherAdminInvalid.into());
    }

    // The Vault shall be up-to-date before warming up the slasher
    if vault.is_update_needed(Clock::get()?.slot, config.epoch_length) {
        msg!("Vault update is needed");
        return Err(VaultError::VaultUpdateNeeded.into());
    }

    // The VaultNcnSlasherTicket shall be ready to be activated
    let mut vault_ncn_slasher_ticket_data = vault_ncn_slasher_ticket.data.borrow_mut();
    let vault_ncn_slasher_ticket =
        VaultNcnSlasherTicket::try_from_slice_unchecked_mut(&mut vault_ncn_slasher_ticket_data)?;
    if !vault_ncn_slasher_ticket
        .state
        .activate(Clock::get()?.slot, config.epoch_length)
    {
        msg!("Slasher is not ready to be activated");
        return Err(VaultError::VaultNcnSlasherTicketFailedWarmup.into());
    }

    Ok(())
}
