use jito_account_traits::AccountDeserialize;
use jito_jsm_core::loader::{load_associated_token_account, load_token_program};
use jito_restaking_core::{
    loader::{
        load_ncn, load_ncn_operator_ticket, load_ncn_vault_slasher_ticket, load_ncn_vault_ticket,
        load_operator, load_operator_ncn_ticket, load_operator_vault_ticket,
    },
    ncn_operator_ticket::NcnOperatorTicket,
    ncn_vault_slasher_ticket::NcnVaultSlasherTicket,
    ncn_vault_ticket::NcnVaultTicket,
    operator_ncn_ticket::OperatorNcnTicket,
    operator_vault_ticket::OperatorVaultTicket,
};
use jito_vault_core::{
    config::Config,
    loader::{
        load_config, load_vault, load_vault_delegation_list,
        load_vault_ncn_slasher_operator_ticket, load_vault_ncn_ticket, load_vault_operator_ticket,
    },
    vault::Vault,
    vault_delegation_list::VaultDelegationList,
    vault_ncn_slasher_operator_ticket::VaultNcnSlasherOperatorTicket,
    vault_ncn_slasher_ticket::VaultNcnSlasherTicket,
    vault_ncn_ticket::VaultNcnTicket,
    vault_operator_ticket::VaultOperatorTicket,
};
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
    let [config, vault_info, ncn, operator, slasher, ncn_operator_ticket, operator_ncn_ticket, ncn_vault_ticket, operator_vault_ticket, vault_ncn_ticket, vault_operator_ticket, ncn_vault_slasher_ticket, vault_ncn_slasher_ticket, vault_delegation_list, vault_ncn_slasher_operator_ticket, vault_token_account, slasher_token_account, token_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    load_config(program_id, config, false)?;
    load_vault(program_id, vault_info, false)?;
    let config_data = config.data.borrow();
    let config = Config::try_from_slice(&config_data)?;
    load_ncn(&config.restaking_program, ncn, false)?;
    load_operator(&config.restaking_program, operator, false)?;
    // slasher
    load_ncn_operator_ticket(
        &config.restaking_program,
        ncn_operator_ticket,
        ncn,
        operator,
        false,
    )?;
    load_operator_ncn_ticket(
        &config.restaking_program,
        operator_ncn_ticket,
        operator,
        ncn,
        false,
    )?;
    load_ncn_vault_ticket(
        &config.restaking_program,
        ncn_vault_ticket,
        ncn,
        vault_info,
        false,
    )?;
    load_operator_vault_ticket(
        &config.restaking_program,
        operator_vault_ticket,
        operator,
        vault_info,
        false,
    )?;
    load_vault_ncn_ticket(program_id, vault_ncn_ticket, vault_info, ncn, false)?;
    load_vault_operator_ticket(
        program_id,
        vault_operator_ticket,
        vault_info,
        operator,
        false,
    )?;
    load_ncn_vault_slasher_ticket(
        &config.restaking_program,
        ncn_vault_slasher_ticket,
        ncn,
        vault_info,
        slasher,
        false,
    )?;
    load_vault_delegation_list(program_id, vault_delegation_list, vault_info, false)?;
    let ncn_epoch = Clock::get()?.slot.checked_div(config.epoch_length).unwrap();
    load_vault_ncn_slasher_operator_ticket(
        program_id,
        vault_ncn_slasher_operator_ticket,
        vault_info,
        ncn,
        slasher,
        operator,
        ncn_epoch,
        true,
    )?;
    let mut vault_data = vault_info.data.borrow_mut();
    let vault = Vault::try_from_slice_mut(&mut vault_data)?;
    load_associated_token_account(vault_token_account, vault_info.key, &vault.supported_mint)?;
    load_associated_token_account(slasher_token_account, slasher.key, &vault.supported_mint)?;
    load_token_program(token_program)?;

    let slot = Clock::get()?.slot;
    let epoch_length = config.epoch_length;

    // The vault delegation list shall not need an update
    let mut vault_delegation_list_data = vault_delegation_list.data.borrow_mut();
    let vault_delegation_list =
        VaultDelegationList::try_from_slice_mut(&mut vault_delegation_list_data)?;
    if vault_delegation_list.is_update_needed(slot, epoch_length) {
        msg!("Vault delegation list update needed");
        return Err(ProgramError::InvalidAccountData);
    }

    // The VaultNcnTicket shall be active or cooling down to get slashed
    let vault_ncn_ticket_data = vault_ncn_ticket.data.borrow();
    let vault_ncn_ticket = VaultNcnTicket::try_from_slice(&vault_ncn_ticket_data)?;
    if !vault_ncn_ticket
        .state
        .is_active_or_cooldown(slot, epoch_length)
    {
        msg!("Vault NCN ticket is not active or in cooldown");
        return Err(ProgramError::InvalidAccountData);
    }

    // The NcnVaultTicket shall be active or cooling down to get slashed
    let ncn_vault_ticket_data = ncn_vault_ticket.data.borrow();
    let ncn_vault_ticket = NcnVaultTicket::try_from_slice(&ncn_vault_ticket_data)?;
    if !ncn_vault_ticket
        .state
        .is_active_or_cooldown(slot, epoch_length)
    {
        msg!("NCN vault ticket is not active or in cooldown");
        return Err(ProgramError::InvalidAccountData);
    }

    // The OperatorVaultTicket shall be active or cooling down to get slashed
    let operator_vault_ticket_data = operator_vault_ticket.data.borrow();
    let operator_vault_ticket = OperatorVaultTicket::try_from_slice(&operator_vault_ticket_data)?;
    if !operator_vault_ticket
        .state
        .is_active_or_cooldown(slot, epoch_length)
    {
        msg!("Operator vault ticket is not active or in cooldown");
        return Err(ProgramError::InvalidAccountData);
    }

    // The VaultOperatorTicket shall be active or cooling down to get slashed
    let vault_operator_ticket_data = vault_operator_ticket.data.borrow();
    let vault_operator_ticket = VaultOperatorTicket::try_from_slice(&vault_operator_ticket_data)?;
    if !vault_operator_ticket
        .state
        .is_active_or_cooldown(slot, epoch_length)
    {
        msg!("Vault operator ticket is not active or in cooldown");
        return Err(ProgramError::InvalidAccountData);
    }

    // The NcnOperatorTicket shall be active or cooling down to get slashed
    let ncn_operator_ticket_data = ncn_operator_ticket.data.borrow();
    let ncn_operator_ticket = NcnOperatorTicket::try_from_slice(&ncn_operator_ticket_data)?;
    if !ncn_operator_ticket
        .state
        .is_active_or_cooldown(slot, epoch_length)
    {
        msg!("NCN operator ticket is not active or in cooldown");
        return Err(ProgramError::InvalidAccountData);
    }

    // The OperatorNcnTicket shall be active or cooling down to get slashed
    let operator_ncn_ticket_data = operator_ncn_ticket.data.borrow();
    let operator_ncn_ticket = OperatorNcnTicket::try_from_slice(&operator_ncn_ticket_data)?;
    if !operator_ncn_ticket
        .state
        .is_active_or_cooldown(slot, epoch_length)
    {
        msg!("Operator NCN ticket is not active or in cooldown");
        return Err(ProgramError::InvalidAccountData);
    }

    // The NcnVaultSlasherTicket shall be active or cooling down to slash
    let ncn_vault_slasher_ticket_data = ncn_vault_slasher_ticket.data.borrow();
    let ncn_vault_slasher_ticket =
        NcnVaultSlasherTicket::try_from_slice(&ncn_vault_slasher_ticket_data)?;
    if !ncn_vault_slasher_ticket
        .state
        .is_active_or_cooldown(slot, epoch_length)
    {
        msg!("NCN vault slasher ticket is not active or in cooldown");
        return Err(ProgramError::InvalidAccountData);
    }

    // The VaultNcnSlasherTicket shall be active or cooling down to slash
    let vault_ncn_slasher_ticket_data = vault_ncn_slasher_ticket.data.borrow();
    let vault_ncn_slasher_ticket =
        VaultNcnSlasherTicket::try_from_slice(&vault_ncn_slasher_ticket_data)?;
    if !vault_ncn_slasher_ticket
        .state
        .is_active_or_cooldown(slot, epoch_length)
    {
        msg!("Vault NCN slasher ticket is not active or in cooldown");
        return Err(ProgramError::InvalidAccountData);
    }

    // The amount slashed for this operator shall not exceed the maximum slashable amount per epoch
    let mut vault_ncn_slasher_operator_ticket_data =
        vault_ncn_slasher_operator_ticket.data.borrow_mut();
    let vault_ncn_slasher_operator_ticket = VaultNcnSlasherOperatorTicket::try_from_slice_mut(
        &mut vault_ncn_slasher_operator_ticket_data,
    )?;
    let amount_after_slash = vault_ncn_slasher_operator_ticket
        .slashed
        .checked_add(slash_amount)
        .ok_or(ProgramError::ArithmeticOverflow)?;
    if amount_after_slash > vault_ncn_slasher_ticket.max_slashable_per_epoch {
        msg!("Slash amount exceeds the maximum slashable amount per epoch");
        return Err(ProgramError::InvalidArgument);
    }

    // slash and update the slashed amount
    vault_delegation_list.slash(operator.key, slash_amount)?;
    vault_ncn_slasher_operator_ticket.slashed = vault_ncn_slasher_operator_ticket
        .slashed
        .checked_add(slash_amount)
        .ok_or(ProgramError::ArithmeticOverflow)?;
    vault.tokens_deposited = vault
        .tokens_deposited
        .checked_sub(slash_amount)
        .ok_or(ProgramError::ArithmeticOverflow)?;

    // transfer the slashed funds
    {
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
    }

    Ok(())
}
