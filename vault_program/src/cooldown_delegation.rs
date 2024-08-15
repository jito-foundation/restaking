use jito_account_traits::AccountDeserialize;
use jito_jsm_core::loader::load_signer;
use jito_restaking_core::loader::load_operator;
use jito_vault_core::loader::load_vault_operator_ticket;
use jito_vault_core::vault_operator_ticket::VaultOperatorTicket;
use jito_vault_core::{
    config::Config,
    loader::{load_config, load_vault},
    vault::Vault,
};
use jito_vault_sdk::error::VaultError;
use solana_program::{
    account_info::AccountInfo, clock::Clock, entrypoint::ProgramResult, msg,
    program_error::ProgramError, pubkey::Pubkey, sysvar::Sysvar,
};

pub fn process_cooldown_delegation(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64,
) -> ProgramResult {
    let [config, vault, operator, vault_operator_ticket, vault_delegation_admin] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    load_config(program_id, config, false)?;
    load_vault(program_id, vault, false)?;
    let config_data = config.data.borrow();
    let config = Config::try_from_slice(&config_data)?;
    load_operator(&config.restaking_program, operator, false)?;
    load_vault_operator_ticket(program_id, vault_operator_ticket, vault, operator, true)?;
    load_signer(vault_delegation_admin, false)?;

    // The Vault delegation admin shall be the signer of the transaction
    let vault_data = vault.data.borrow();
    let vault = Vault::try_from_slice(&vault_data)?;
    if vault.delegation_admin.ne(vault_delegation_admin.key) {
        msg!("Invalid delegation admin for vault");
        return Err(VaultError::VaultDelegationAdminInvalid.into());
    }

    // The Vault shall be up-to-date before removing delegation
    if vault.is_update_needed(Clock::get()?.slot, config.epoch_length) {
        msg!("Vault update is needed");
        return Err(VaultError::VaultUpdateNeeded.into());
    }

    let mut vault_operator_ticket_data = vault_operator_ticket.data.borrow_mut();
    let vault_operator_ticket =
        VaultOperatorTicket::try_from_slice_mut(&mut vault_operator_ticket_data)?;
    vault_operator_ticket.undelegate(amount)?;

    Ok(())
}
