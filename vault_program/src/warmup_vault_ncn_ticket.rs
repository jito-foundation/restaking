use jito_account_traits::AccountDeserialize;
use jito_jsm_core::loader::load_signer;
use jito_restaking_core::{
    loader::{load_ncn, load_ncn_vault_ticket},
    ncn_vault_ticket::NcnVaultTicket,
};
use jito_vault_core::{
    config::Config,
    loader::{load_config, load_vault, load_vault_ncn_ticket},
    vault::Vault,
    vault_ncn_ticket::VaultNcnTicket,
};
use jito_vault_sdk::error::VaultError;
use solana_program::{
    account_info::AccountInfo, clock::Clock, entrypoint::ProgramResult, msg,
    program_error::ProgramError, pubkey::Pubkey, sysvar::Sysvar,
};

/// Instruction: [`crate::VaultInstruction::WarmupVaultNcnTicket`]
pub fn process_warmup_vault_ncn_ticket(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let [config, vault, ncn, ncn_vault_ticket, vault_ncn_ticket, vault_ncn_admin] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    load_config(program_id, config, false)?;
    load_vault(program_id, vault, false)?;
    let config_data = config.data.borrow();
    let config = Config::try_from_slice(&config_data)?;
    load_ncn(&config.restaking_program, ncn, false)?;
    load_ncn_vault_ticket(
        &config.restaking_program,
        ncn_vault_ticket,
        ncn,
        vault,
        false,
    )?;
    load_vault_ncn_ticket(program_id, vault_ncn_ticket, vault, ncn, true)?;
    load_signer(vault_ncn_admin, false)?;

    // The Vault NCN admin shall be the signer of the transaction
    let vault_data = vault.data.borrow();
    let vault = Vault::try_from_slice(&vault_data)?;
    if vault.ncn_admin.ne(vault_ncn_admin.key) {
        msg!("Invalid ncn admin for vault");
        return Err(VaultError::VaultNcnAdminInvalid.into());
    }

    // The NcnVaultTicket shall be active
    let ncn_vault_ticket_data = ncn_vault_ticket.data.borrow();
    let ncn_vault_ticket = NcnVaultTicket::try_from_slice(&ncn_vault_ticket_data)?;
    if !ncn_vault_ticket
        .state
        .is_active(Clock::get()?.slot, config.epoch_length)
    {
        msg!("Ncn vault ticket is not active");
        return Err(VaultError::NcnVaultTicketNotActive.into());
    }

    // The VaultNcnTicket shall be ready to be activated
    let mut vault_ncn_ticket_data = vault_ncn_ticket.data.borrow_mut();
    let vault_ncn_ticket = VaultNcnTicket::try_from_slice_mut(&mut vault_ncn_ticket_data)?;
    if !vault_ncn_ticket
        .state
        .activate(Clock::get()?.slot, config.epoch_length)
    {
        msg!("VaultNcnTicket is not ready to be activated");
        return Err(VaultError::VaultNcnTicketFailedWarmup.into());
    }

    Ok(())
}
