use jito_account_traits::AccountDeserialize;
use jito_jsm_core::loader::{load_signer, load_system_program};
use jito_restaking_core::loader::load_operator;
use jito_vault_core::{
    config::Config,
    loader::{load_config, load_vault, load_vault_operator_ticket},
    vault::Vault,
    vault_delegation_list::VaultDelegationList,
    vault_operator_ticket::VaultOperatorTicket,
};
use solana_program::{
    account_info::AccountInfo, clock::Clock, entrypoint::ProgramResult, msg,
    program_error::ProgramError, pubkey::Pubkey, sysvar::Sysvar,
};

pub fn process_add_delegation(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64,
) -> ProgramResult {
    let [config, vault, operator, vault_operator_ticket, vault_delegation_list, vault_delegation_admin, payer, system_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    load_config(program_id, config, false)?;
    load_vault(&program_id, vault, false)?;
    let config_data = config.data.borrow();
    let config = Config::try_from_slice(&config_data)?;
    load_operator(&config.restaking_program, operator, false)?;
    load_vault_operator_ticket(program_id, vault_operator_ticket, vault, operator, false)?;
    load_signer(vault_delegation_admin, false)?;
    load_signer(payer, true)?;
    load_system_program(system_program)?;

    let vault_data = vault.data.borrow();
    let vault = Vault::try_from_slice(&vault_data)?;
    if vault.delegation_admin.ne(&vault_delegation_admin.key) {
        msg!("Invalid delegation admin for vault");
        return Err(ProgramError::InvalidAccountData);
    }

    let vault_operator_ticket_data = vault_operator_ticket.data.borrow();
    let vault_operator_ticket = VaultOperatorTicket::try_from_slice(&vault_operator_ticket_data)?;
    if !vault_operator_ticket
        .state
        .is_active_or_cooldown(Clock::get()?.slot, config.epoch_length)
    {
        msg!("Vault operator ticket is not active or in cooldown");
        return Err(ProgramError::InvalidAccountData);
    }

    let mut vault_delegation_list_data = vault_delegation_list.data.borrow_mut();
    let mut vault_delegation_list =
        VaultDelegationList::try_from_slice_mut(&mut vault_delegation_list_data)?;
    if vault_delegation_list.is_update_needed(Clock::get()?.slot, config.epoch_length) {
        vault_delegation_list.check_update_needed(Clock::get()?.slot, config.epoch_length)?;
    }

    vault_delegation_list.delegate(*operator.key, amount, vault.max_delegation_amount()?)?;

    Ok(())
}
