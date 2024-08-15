use jito_account_traits::AccountDeserialize;
use jito_jsm_core::loader::{load_signer, load_system_program};
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

pub fn process_add_delegation(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64,
) -> ProgramResult {
    let [config, vault, operator, vault_operator_ticket, vault_delegation_admin, payer, system_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    load_config(program_id, config, false)?;
    load_vault(program_id, vault, true)?;
    let config_data = config.data.borrow();
    let config = Config::try_from_slice(&config_data)?;
    load_operator(&config.restaking_program, operator, false)?;
    load_vault_operator_ticket(program_id, vault_operator_ticket, vault, operator, true)?;
    load_signer(vault_delegation_admin, false)?;
    load_signer(payer, true)?;
    load_system_program(system_program)?;

    // The Vault delegation admin shall be the signer of the transaction
    let mut vault_data = vault.data.borrow_mut();
    let vault = Vault::try_from_slice_mut(&mut vault_data)?;
    if vault.delegation_admin.ne(vault_delegation_admin.key) {
        msg!("Invalid delegation admin for vault");
        return Err(VaultError::VaultDelegationAdminInvalid.into());
    }

    // The Vault shall be up-to-date before adding delegation
    if vault.is_update_needed(Clock::get()?.slot, config.epoch_length) {
        msg!("Vault update is needed");
        return Err(VaultError::VaultUpdateNeeded.into());
    }

    // The vault operator ticket must be active in order to add delegation to the operator
    let mut vault_operator_ticket_data = vault_operator_ticket.data.borrow_mut();
    let vault_operator_ticket =
        VaultOperatorTicket::try_from_slice_mut(&mut vault_operator_ticket_data)?;
    if !vault_operator_ticket
        .state
        .is_active(Clock::get()?.slot, config.epoch_length)
    {
        msg!("Vault operator ticket is not active");
        return Err(VaultError::VaultOperatorTicketNotActive.into());
    }

    // The vault shall not over allocate assets for delegation
    // TODO (LB): need to check withdrawable reserve amount
    let assets_available_for_staking = vault
        .tokens_deposited
        .checked_sub(vault.amount_delegated)
        .ok_or(VaultError::VaultOverflow)?;
    if amount > assets_available_for_staking {
        msg!("Insufficient funds in vault for delegation");
        return Err(VaultError::VaultInsufficientFunds.into());
    }

    vault_operator_ticket.staked_amount = vault_operator_ticket
        .staked_amount
        .checked_add(amount)
        .ok_or(VaultError::VaultOverflow)?;

    vault.amount_delegated = vault
        .amount_delegated
        .checked_add(amount)
        .ok_or(VaultError::VaultOverflow)?;

    Ok(())
}
