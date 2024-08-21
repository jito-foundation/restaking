use jito_bytemuck::AccountDeserialize;
use jito_jsm_core::loader::load_signer;
use jito_restaking_core::ncn::Ncn;
use jito_vault_core::{
    config::Config, vault::Vault, vault_ncn_slasher_ticket::VaultNcnSlasherTicket,
};
use jito_vault_sdk::error::VaultError;
use solana_program::{
    account_info::AccountInfo, clock::Clock, entrypoint::ProgramResult, msg,
    program_error::ProgramError, pubkey::Pubkey, sysvar::Sysvar,
};

/// Instruction: [`crate::VaultInstruction::CooldownVaultNcnSlasherTicket`]
pub fn process_cooldown_vault_ncn_slasher_ticket(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let [config, vault_info, ncn, slasher, vault_ncn_slasher_ticket, vault_slasher_admin] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    Config::load(program_id, config, false)?;
    let config_data = config.data.borrow();
    let config = Config::try_from_slice_unchecked(&config_data)?;
    Vault::load(program_id, vault_info, false)?;
    let vault_data = vault_info.data.borrow();
    let vault = Vault::try_from_slice_unchecked(&vault_data)?;
    Ncn::load(&config.restaking_program, ncn, false)?;
    VaultNcnSlasherTicket::load(
        program_id,
        vault_ncn_slasher_ticket,
        vault_info,
        ncn,
        slasher,
        true,
    )?;
    let mut vault_ncn_slasher_ticket_data = vault_ncn_slasher_ticket.data.borrow_mut();
    let vault_ncn_slasher_ticket =
        VaultNcnSlasherTicket::try_from_slice_unchecked_mut(&mut vault_ncn_slasher_ticket_data)?;
    load_signer(vault_slasher_admin, false)?;

    vault.check_slasher_admin(vault_slasher_admin.key)?;
    vault.check_update_state_ok(Clock::get()?.slot, config.epoch_length())?;

    // The vault slasher ticket must be active in order to cooldown the slasher
    if !vault_ncn_slasher_ticket
        .state
        .deactivate(Clock::get()?.slot, config.epoch_length())
    {
        msg!("Slasher is not ready to be deactivated");
        return Err(VaultError::VaultNcnSlasherTicketFailedCooldown.into());
    }

    Ok(())
}
