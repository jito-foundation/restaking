use jito_account_traits::AccountDeserialize;
use jito_jsm_core::loader::load_signer;
use jito_restaking_core::loader::load_ncn;
use jito_vault_core::{
    config::Config,
    loader::{load_config, load_vault, load_vault_ncn_slasher_ticket},
    vault::Vault,
    vault_ncn_slasher_ticket::VaultNcnSlasherTicket,
};
use solana_program::{
    account_info::AccountInfo, clock::Clock, entrypoint::ProgramResult, msg,
    program_error::ProgramError, pubkey::Pubkey, sysvar::Sysvar,
};

/// Instruction: [`crate::VaultInstruction::CooldownVaultNcnSlasherTicket`]
pub fn process_cooldown_vault_ncn_slasher_ticket(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let [config, vault, ncn, slasher, vault_ncn_slasher_ticket, vault_slasher_admin] = accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    load_config(program_id, config, false)?;
    load_vault(program_id, vault, false)?;
    let config_data = config.data.borrow();
    let config = Config::try_from_slice(&config_data)?;
    load_ncn(&config.restaking_program, ncn, false)?;
    load_vault_ncn_slasher_ticket(
        program_id,
        vault_ncn_slasher_ticket,
        vault,
        ncn,
        slasher,
        true,
    )?;
    load_signer(vault_slasher_admin, false)?;

    let vault_data = vault.data.borrow();
    let vault = Vault::try_from_slice(&vault_data)?;
    if vault.slasher_admin.ne(vault_slasher_admin.key) {
        msg!("Invalid slasher admin for vault");
        return Err(ProgramError::InvalidAccountData);
    }

    let mut vault_ncn_slasher_ticket_data = vault_ncn_slasher_ticket.data.borrow_mut();
    let vault_ncn_slasher_ticket =
        VaultNcnSlasherTicket::try_from_slice_mut(&mut vault_ncn_slasher_ticket_data)?;
    if !vault_ncn_slasher_ticket
        .state
        .deactivate(Clock::get()?.slot, config.epoch_length)
    {
        msg!("Slasher is not ready to be deactivated");
        return Err(ProgramError::InvalidAccountData);
    }

    Ok(())
}
