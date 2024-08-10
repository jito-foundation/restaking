use jito_account_traits::AccountDeserialize;
use jito_jsm_core::loader::load_signer;
use jito_restaking_core::loader::load_ncn;
use jito_vault_core::{
    config::Config,
    loader::{load_config, load_vault, load_vault_ncn_ticket},
    vault::Vault,
    vault_ncn_ticket::VaultNcnTicket,
};
use solana_program::{
    account_info::AccountInfo, clock::Clock, entrypoint::ProgramResult, msg,
    program_error::ProgramError, pubkey::Pubkey, sysvar::Sysvar,
};

/// Remove a vault from the vault's NCN list.
///
/// # Behavior:
/// * The vault admin shall have the ability to remove support for a previously supported vault
/// at any time, independent of whether the NCN still supports the vault or not.
///
/// Instruction: [`crate::VaultInstruction::CooldownNcn`]
pub fn process_vault_cooldown_ncn(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let [config, vault, ncn, vault_ncn_ticket, vault_ncn_admin] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    load_config(program_id, config, false)?;
    load_vault(program_id, vault, false)?;
    let config_data = config.data.borrow();
    let config = Config::try_from_slice(&config_data)?;
    load_ncn(&config.restaking_program, ncn, false)?;
    load_vault_ncn_ticket(program_id, vault_ncn_ticket, ncn, vault, true)?;
    load_signer(vault_ncn_admin, false)?;

    let vault_data = vault.data.borrow();
    let vault = Vault::try_from_slice(&vault_data)?;
    if vault.ncn_admin.ne(vault_ncn_admin.key) {
        msg!("Invalid NCN admin for vault");
        return Err(ProgramError::InvalidAccountData);
    }

    let mut vault_ncn_ticket_data = vault_ncn_ticket.data.borrow_mut();
    let vault_ncn_ticket = VaultNcnTicket::try_from_slice_mut(&mut vault_ncn_ticket_data)?;
    if !vault_ncn_ticket
        .state
        .deactivate(Clock::get()?.slot, config.epoch_length)
    {
        msg!("NCN is not ready to be deactivated");
        return Err(ProgramError::InvalidAccountData);
    }

    Ok(())
}
