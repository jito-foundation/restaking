use jito_bytemuck::AccountDeserialize;
use jito_jsm_core::loader::{load_associated_token_account, load_signer, load_token_program};
use jito_restaking_core::{
    ncn::Ncn, ncn_operator_state::NcnOperatorState,
    ncn_vault_slasher_ticket::NcnVaultSlasherTicket, ncn_vault_ticket::NcnVaultTicket,
    operator::Operator, operator_vault_ticket::OperatorVaultTicket,
};
use jito_vault_core::{
    config::Config, vault::Vault, vault_ncn_slasher_operator_ticket::VaultNcnSlasherOperatorTicket,
    vault_ncn_slasher_ticket::VaultNcnSlasherTicket, vault_ncn_ticket::VaultNcnTicket,
    vault_operator_delegation::VaultOperatorDelegation,
};
use jito_vault_sdk::error::VaultError;
use solana_program::{
    account_info::AccountInfo, clock::Clock, entrypoint::ProgramResult, msg,
    program::invoke_signed, program_error::ProgramError, pubkey::Pubkey, sysvar::Sysvar,
};
use spl_token::instruction::transfer;

/// Processes the vault slash instruction: [`crate::VaultInstruction::Slash`]
pub fn process_slash(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    slash_amount: u64,
) -> ProgramResult {
    let [config, vault_info, ncn, operator, slasher, ncn_operator_state, ncn_vault_ticket, operator_vault_ticket, vault_ncn_ticket, vault_operator_delegation, ncn_vault_slasher_ticket, vault_ncn_slasher_ticket, vault_ncn_slasher_operator_ticket, vault_token_account, slasher_token_account, token_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    Config::load(program_id, config, false)?;
    let config_data = config.data.borrow();
    let config = Config::try_from_slice_unchecked(&config_data)?;
    Vault::load(program_id, vault_info, false)?;
    let mut vault_data = vault_info.data.borrow_mut();
    let vault = Vault::try_from_slice_unchecked_mut(&mut vault_data)?;
    Ncn::load(&config.restaking_program, ncn, false)?;
    Operator::load(&config.restaking_program, operator, false)?;
    load_signer(slasher, false)?;
    NcnOperatorState::load(
        &config.restaking_program,
        ncn_operator_state,
        ncn,
        operator,
        false,
    )?;
    let ncn_operator_state_data = ncn_operator_state.data.borrow();
    let ncn_operator_state = NcnOperatorState::try_from_slice_unchecked(&ncn_operator_state_data)?;
    NcnVaultTicket::load(
        &config.restaking_program,
        ncn_vault_ticket,
        ncn,
        vault_info,
        false,
    )?;
    let ncn_vault_ticket_data = ncn_vault_ticket.data.borrow();
    let ncn_vault_ticket = NcnVaultTicket::try_from_slice_unchecked(&ncn_vault_ticket_data)?;
    OperatorVaultTicket::load(
        &config.restaking_program,
        operator_vault_ticket,
        operator,
        vault_info,
        false,
    )?;
    let operator_vault_ticket_data = operator_vault_ticket.data.borrow();
    let operator_vault_ticket =
        OperatorVaultTicket::try_from_slice_unchecked(&operator_vault_ticket_data)?;
    VaultNcnTicket::load(program_id, vault_ncn_ticket, vault_info, ncn, false)?;
    let vault_ncn_ticket_data = vault_ncn_ticket.data.borrow();
    let vault_ncn_ticket = VaultNcnTicket::try_from_slice_unchecked(&vault_ncn_ticket_data)?;
    VaultOperatorDelegation::load(
        program_id,
        vault_operator_delegation,
        vault_info,
        operator,
        true,
    )?;
    let mut vault_operator_delegation_data = vault_operator_delegation.data.borrow_mut();
    let vault_operator_delegation =
        VaultOperatorDelegation::try_from_slice_unchecked_mut(&mut vault_operator_delegation_data)?;
    NcnVaultSlasherTicket::load(
        &config.restaking_program,
        ncn_vault_slasher_ticket,
        ncn,
        vault_info,
        slasher,
        false,
    )?;
    let ncn_vault_slasher_ticket_data = ncn_vault_slasher_ticket.data.borrow();
    let ncn_vault_slasher_ticket =
        NcnVaultSlasherTicket::try_from_slice_unchecked(&ncn_vault_slasher_ticket_data)?;
    VaultNcnSlasherTicket::load(
        program_id,
        vault_ncn_slasher_ticket,
        vault_info,
        ncn,
        slasher,
        false,
    )?;
    let vault_ncn_slasher_ticket_data = vault_ncn_slasher_ticket.data.borrow();
    let vault_ncn_slasher_ticket =
        VaultNcnSlasherTicket::try_from_slice_unchecked(&vault_ncn_slasher_ticket_data)?;
    let ncn_epoch = Clock::get()?
        .slot
        .checked_div(config.epoch_length())
        .unwrap();
    VaultNcnSlasherOperatorTicket::load(
        program_id,
        vault_ncn_slasher_operator_ticket,
        vault_info,
        ncn,
        slasher,
        operator,
        ncn_epoch,
        true,
    )?;
    let mut vault_ncn_slasher_operator_ticket_data =
        vault_ncn_slasher_operator_ticket.data.borrow_mut();
    let vault_ncn_slasher_operator_ticket =
        VaultNcnSlasherOperatorTicket::try_from_slice_unchecked_mut(
            &mut vault_ncn_slasher_operator_ticket_data,
        )?;
    load_associated_token_account(vault_token_account, vault_info.key, &vault.supported_mint)?;
    load_associated_token_account(slasher_token_account, slasher.key, &vault.supported_mint)?;
    load_token_program(token_program)?;

    let slot = Clock::get()?.slot;
    let epoch_length = config.epoch_length();

    // The vault shall be up-to-date before slashing
    vault.check_update_state_ok(Clock::get()?.slot, epoch_length)?;

    // All ticket states shall be active or cooling down
    check_states_active_or_cooling_down(
        vault_ncn_slasher_ticket,
        ncn_vault_slasher_ticket,
        ncn_operator_state,
        operator_vault_ticket,
        vault_ncn_ticket,
        ncn_vault_ticket,
        slot,
        epoch_length,
    )?;

    // The amount slashed for this operator shall not exceed the maximum slashable amount per epoch
    vault_ncn_slasher_operator_ticket.check_slashing_amount_not_exceeded(
        slash_amount,
        vault_ncn_slasher_ticket.max_slashable_per_epoch(),
    )?;

    // The VaultOperatorDelegation shall be slashed and the vault amounts shall be updated
    slash_and_update_vault(
        vault,
        vault_operator_delegation,
        vault_ncn_slasher_operator_ticket,
        slash_amount,
    )?;

    // transfer the slashed funds
    let mut vault_seeds = Vault::seeds(&vault.base);
    vault_seeds.push(vec![vault.bump]);
    let vault_seeds_slice = vault_seeds
        .iter()
        .map(|seed| seed.as_slice())
        .collect::<Vec<&[u8]>>();
    drop(vault_data);
    invoke_signed(
        &transfer(
            &spl_token::id(),
            vault_token_account.key,
            slasher_token_account.key,
            vault_info.key,
            &[],
            slash_amount,
        )?,
        &[
            vault_token_account.clone(),
            slasher_token_account.clone(),
            vault_info.clone(),
        ],
        &[vault_seeds_slice.as_slice()],
    )?;

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn check_states_active_or_cooling_down(
    vault_ncn_slasher_ticket: &VaultNcnSlasherTicket,
    ncn_vault_slasher_ticket: &NcnVaultSlasherTicket,
    ncn_operator_state: &NcnOperatorState,
    operator_vault_ticket: &OperatorVaultTicket,
    vault_ncn_ticket: &VaultNcnTicket,
    ncn_vault_ticket: &NcnVaultTicket,
    slot: u64,
    epoch_length: u64,
) -> ProgramResult {
    if !vault_ncn_slasher_ticket
        .state
        .is_active_or_cooldown(slot, epoch_length)
    {
        msg!("Vault NCN slasher ticket is not active or in cooldown");
        return Err(VaultError::VaultNcnSlasherTicketUnslashable.into());
    }
    if !ncn_vault_slasher_ticket
        .state
        .is_active_or_cooldown(slot, epoch_length)
    {
        msg!("NCN vault slasher ticket is not active or in cooldown");
        return Err(VaultError::NcnVaultSlasherTicketUnslashable.into());
    }
    if !ncn_operator_state
        .ncn_opt_in_state
        .is_active_or_cooldown(slot, epoch_length)
    {
        msg!("NCN opt-in to operator is not active or in cooldown");
        return Err(VaultError::NcnOperatorStateUnslashable.into());
    }
    if !ncn_operator_state
        .operator_opt_in_state
        .is_active_or_cooldown(slot, epoch_length)
    {
        msg!("Operator opt-in to NCN is not active or in cooldown");
        return Err(VaultError::NcnOperatorStateUnslashable.into());
    }
    if !operator_vault_ticket
        .state
        .is_active_or_cooldown(slot, epoch_length)
    {
        msg!("Operator vault ticket is not active or in cooldown");
        return Err(VaultError::OperatorVaultTicketUnslashable.into());
    }
    if !vault_ncn_ticket
        .state
        .is_active_or_cooldown(slot, epoch_length)
    {
        msg!("Vault NCN ticket is not active or in cooldown");
        return Err(VaultError::VaultNcnTicketUnslashable.into());
    }
    if !ncn_vault_ticket
        .state
        .is_active_or_cooldown(slot, epoch_length)
    {
        msg!("NCN vault ticket is not active or in cooldown");
        return Err(VaultError::NcnVaultTicketUnslashable.into());
    }
    Ok(())
}

/// Slashes the vault and updates the vault amounts based on the slashing amount.
fn slash_and_update_vault(
    vault: &mut Vault,
    vault_operator_delegation: &mut VaultOperatorDelegation,
    vault_ncn_slasher_operator_ticket: &mut VaultNcnSlasherOperatorTicket,
    slash_amount: u64,
) -> ProgramResult {
    // undo the delegation, slash then accumulate the delegation
    vault
        .delegation_state
        .subtract(&vault_operator_delegation.delegation_state)?;
    vault_operator_delegation
        .delegation_state
        .slash(slash_amount)?;
    vault
        .delegation_state
        .accumulate(&vault_operator_delegation.delegation_state)?;

    vault.decrement_tokens_deposited(slash_amount)?;
    vault_ncn_slasher_operator_ticket.increment_slashed(slash_amount)?;
    Ok(())
}
