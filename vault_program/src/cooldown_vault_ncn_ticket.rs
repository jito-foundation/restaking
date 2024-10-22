use jito_bytemuck::AccountDeserialize;
use jito_jsm_core::loader::load_signer;
use jito_restaking_core::ncn::Ncn;
use jito_vault_core::{config::Config, vault::Vault, vault_ncn_ticket::VaultNcnTicket};
use jito_vault_sdk::error::VaultError;
use solana_program::{
    account_info::AccountInfo, clock::Clock, entrypoint::ProgramResult, msg,
    program_error::ProgramError, pubkey::Pubkey, sysvar::Sysvar,
};

/// Remove a vault from the vault's NCN list.
///
/// # Behavior:
/// * The vault admin shall have the ability to remove support for a previously supported vault
///   at any time, independent of whether the NCN still supports the vault or not.
///
/// Instruction: [`crate::VaultInstruction::CooldownVaultNcnTicket`]
pub fn process_cooldown_vault_ncn_ticket(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let [config, vault_info, ncn, vault_ncn_ticket, vault_ncn_admin] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    Config::load(program_id, config, false)?;
    let config_data = config.data.borrow();
    let config = Config::try_from_slice_unchecked(&config_data)?;
    Vault::load(program_id, vault_info, false)?;
    let vault_data = vault_info.data.borrow();
    let vault = Vault::try_from_slice_unchecked(&vault_data)?;
    Ncn::load(&config.restaking_program, ncn, false)?;
    VaultNcnTicket::load(program_id, vault_ncn_ticket, ncn, vault_info, true)?;
    let mut vault_ncn_ticket_data = vault_ncn_ticket.data.borrow_mut();
    let vault_ncn_ticket =
        VaultNcnTicket::try_from_slice_unchecked_mut(&mut vault_ncn_ticket_data)?;
    load_signer(vault_ncn_admin, false)?;

    vault.check_ncn_admin(vault_ncn_admin.key)?;
    vault.check_update_state_ok(Clock::get()?.slot, config.epoch_length())?;
    vault.check_is_paused()?;

    // The VaultNcnTicket must be active in order to cooldown the NCN
    if !vault_ncn_ticket
        .state
        .deactivate(Clock::get()?.slot, config.epoch_length())?
    {
        msg!("NCN is not ready to be deactivated");
        return Err(VaultError::VaultNcnTicketFailedCooldown.into());
    }

    Ok(())
}
