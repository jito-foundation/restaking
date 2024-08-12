use jito_account_traits::AccountDeserialize;
use jito_jsm_core::loader::load_signer;
use jito_restaking_core::loader::load_operator;
use jito_vault_core::{
    config::Config,
    loader::{load_config, load_vault, load_vault_operator_ticket},
    vault::Vault,
    vault_operator_ticket::VaultOperatorTicket,
};
use jito_vault_sdk::error::VaultError;
use solana_program::{
    account_info::AccountInfo, clock::Clock, entrypoint::ProgramResult, msg,
    program_error::ProgramError, pubkey::Pubkey, sysvar::Sysvar,
};

/// Processes the vault remove operator instruction: [`crate::VaultInstruction::CooldownVaultOperatorTicket`]
pub fn process_cooldown_vault_operator_ticket(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let [config, vault, operator, vault_operator_ticket, vault_operator_admin] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    load_config(program_id, config, false)?;
    load_vault(program_id, vault, false)?;
    let config_data = config.data.borrow();
    let config = Config::try_from_slice(&config_data)?;
    load_operator(&config.restaking_program, operator, false)?;
    load_vault_operator_ticket(program_id, vault_operator_ticket, vault, operator, true)?;
    load_signer(vault_operator_admin, false)?;

    // The Vault operator admin shall be the signer of the transaction
    let vault_data = vault.data.borrow();
    let vault = Vault::try_from_slice(&vault_data)?;
    if vault.operator_admin.ne(vault_operator_admin.key) {
        msg!("Invalid operator admin for vault");
        return Err(VaultError::VaultOperatorAdminInvalid.into());
    }

    // The VaultOperatorTicket shall be active in order to cooldown the operator
    let mut vault_operator_ticket_data = vault_operator_ticket.data.borrow_mut();
    let vault_operator_ticket =
        VaultOperatorTicket::try_from_slice_mut(&mut vault_operator_ticket_data)?;
    if !vault_operator_ticket
        .state
        .deactivate(Clock::get()?.slot, config.epoch_length)
    {
        msg!("Operator is not ready to be deactivated");
        return Err(VaultError::VaultOperatorTicketFailedCooldown.into());
    }

    Ok(())
}
