use jito_account_traits::AccountDeserialize;
use jito_jsm_core::{
    close_program_account,
    loader::{
        load_associated_token_account, load_signer, load_system_program, load_token_mint,
        load_token_program,
    },
};
use jito_vault_core::{
    config::Config,
    loader::{
        load_config, load_vault, load_vault_delegation_list, load_vault_staker_withdrawal_ticket,
    },
    vault::Vault,
    vault_delegation_list::VaultDelegationList,
    vault_staker_withdrawal_ticket::VaultStakerWithdrawalTicket,
};
use jito_vault_sdk::error::VaultError;
use solana_program::{
    account_info::AccountInfo, clock::Clock, entrypoint::ProgramResult, msg,
    program::invoke_signed, program_error::ProgramError, program_pack::Pack, pubkey::Pubkey,
    sysvar::Sysvar,
};
use spl_token::{
    instruction::{burn, close_account, transfer},
    state::Account,
};

/// Burns the withdrawal ticket, transferring the assets to the staker and closing the withdrawal ticket.
///
/// One should call the [`crate::VaultInstruction::UpdateVault`] instruction before running this instruction
/// to ensure that any rewards that were accrued are accounted for.
pub fn process_burn_withdrawal_ticket(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let [config, vault_info, vault_delegation_list, vault_token_account, lrt_mint, staker, staker_token_account, staker_lrt_token_account, vault_staker_withdrawal_ticket_info, vault_staker_withdrawal_ticket_token_account, token_program, system_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    load_config(program_id, config, false)?;
    load_vault(program_id, vault_info, true)?;
    load_vault_delegation_list(program_id, vault_delegation_list, vault_info, true)?;
    let mut vault_data = vault_info.data.borrow_mut();
    let vault = Vault::try_from_slice_mut(&mut vault_data)?;
    load_associated_token_account(vault_token_account, vault_info.key, &vault.supported_mint)?;
    load_token_mint(lrt_mint)?;
    load_signer(staker, false)?;
    load_associated_token_account(staker_token_account, staker.key, &vault.supported_mint)?;
    load_associated_token_account(staker_lrt_token_account, staker.key, &vault.lrt_mint)?;
    load_vault_staker_withdrawal_ticket(
        program_id,
        vault_staker_withdrawal_ticket_info,
        vault_info,
        staker,
        true,
    )?;
    load_associated_token_account(
        vault_staker_withdrawal_ticket_token_account,
        vault_staker_withdrawal_ticket_info.key,
        &vault.lrt_mint,
    )?;
    load_token_program(token_program)?;
    load_system_program(system_program)?;

    if vault.lrt_mint.ne(lrt_mint.key) {
        msg!("Vault LRT mint mismatch");
        return Err(ProgramError::InvalidArgument);
    }

    let config_data = config.data.borrow();
    let config = Config::try_from_slice(&config_data)?;

    let mut vault_delegation_list_data = vault_delegation_list.data.borrow_mut();
    let vault_delegation_list =
        VaultDelegationList::try_from_slice_mut(&mut vault_delegation_list_data)?;
    if vault_delegation_list.is_update_needed(Clock::get()?.slot, config.epoch_length) {
        msg!("Vault delegation list needs to be updated");
        return Err(VaultError::VaultDelegationListUpdateNeeded.into());
    }

    let vault_staker_withdrawal_ticket_data = vault_staker_withdrawal_ticket_info.data.borrow();
    let vault_staker_withdrawal_ticket =
        VaultStakerWithdrawalTicket::try_from_slice(&vault_staker_withdrawal_ticket_data)?;
    if !vault_staker_withdrawal_ticket.is_withdrawable(Clock::get()?.slot, config.epoch_length)? {
        msg!("Vault staker withdrawal ticket is not withdrawable");
        return Err(VaultError::VaultStakerWithdrawalTicketNotWithdrawable.into());
    }

    // find the current redemption amount and the original redemption amount in the withdrawal ticket
    // TODO (LB): this logic is buggy no doubt
    let redemption_amount =
        vault.calculate_assets_returned_amount(vault_staker_withdrawal_ticket.lrt_amount)?;
    let original_redemption_amount = vault_staker_withdrawal_ticket.withdraw_allocation_amount;

    let actual_withdraw_amount = if redemption_amount > original_redemption_amount {
        // The program can guarantee the original redemption amount, but if the redemption amount
        // is greater than the original amount, there were rewards that accrued
        // to the LRT.
        // The program attempts to figure out how much more of the asset can be unstaked to fulfill
        // as much of the redemption amount as possible.
        // Available unstaked assets is equal to:
        // the amount of tokens deposited - any delegated security - the amount reserved for withdraw tickets
        let tokens_deposited_in_vault = vault.tokens_deposited;
        let delegated_security_in_vault = vault_delegation_list.total_security()?;
        let assets_reserved_for_withdrawal_tickets = vault.withdrawable_reserve_amount;

        let available_unstaked_assets = tokens_deposited_in_vault
            .checked_sub(delegated_security_in_vault)
            .and_then(|x| x.checked_sub(assets_reserved_for_withdrawal_tickets))
            .ok_or(ProgramError::InsufficientFunds)?;

        // Calculate the extra amount that can be withdrawn
        let extra_amount = redemption_amount
            .checked_sub(original_redemption_amount)
            .ok_or(ProgramError::ArithmeticOverflow)?;

        // Determine the actual amount to withdraw
        original_redemption_amount
            .checked_add(extra_amount.min(available_unstaked_assets))
            .ok_or(ProgramError::ArithmeticOverflow)?
    } else {
        redemption_amount
    };

    let lrt_to_burn = vault.calculate_lrt_mint_amount(actual_withdraw_amount)?;
    let lrt_to_burn = std::cmp::min(lrt_to_burn, vault_staker_withdrawal_ticket.lrt_amount);

    // burn the assets + close the token account + withdraw token account
    {
        let (_, vault_staker_withdraw_bump, mut vault_staker_withdraw_seeds) =
            VaultStakerWithdrawalTicket::find_program_address(
                program_id,
                vault_info.key,
                staker.key,
                &vault_staker_withdrawal_ticket.base,
            );
        vault_staker_withdraw_seeds.push(vec![vault_staker_withdraw_bump]);
        let seed_slices: Vec<&[u8]> = vault_staker_withdraw_seeds
            .iter()
            .map(|seed| seed.as_slice())
            .collect();

        drop(vault_staker_withdrawal_ticket_data);

        // burn the LRT tokens
        invoke_signed(
            &burn(
                &spl_token::id(),
                vault_staker_withdrawal_ticket_token_account.key,
                lrt_mint.key,
                vault_staker_withdrawal_ticket_info.key,
                &[],
                lrt_to_burn,
            )?,
            &[
                vault_staker_withdrawal_ticket_token_account.clone(),
                lrt_mint.clone(),
                vault_staker_withdrawal_ticket_info.clone(),
            ],
            &[&seed_slices],
        )?;

        // if there are any excess, transfer them to the staker
        let lrt_token_account =
            Account::unpack(&vault_staker_withdrawal_ticket_token_account.data.borrow())?;
        if lrt_token_account.amount > 0 {
            invoke_signed(
                &transfer(
                    &spl_token::id(),
                    vault_staker_withdrawal_ticket_token_account.key,
                    staker_lrt_token_account.key,
                    vault_staker_withdrawal_ticket_info.key,
                    &[],
                    lrt_token_account.amount,
                )?,
                &[
                    vault_staker_withdrawal_ticket_token_account.clone(),
                    staker_lrt_token_account.clone(),
                    vault_staker_withdrawal_ticket_info.clone(),
                ],
                &[&seed_slices],
            )?;
        }

        // close token account
        invoke_signed(
            &close_account(
                &spl_token::id(),
                vault_staker_withdrawal_ticket_token_account.key,
                staker.key,
                vault_staker_withdrawal_ticket_info.key,
                &[],
            )?,
            &[
                vault_staker_withdrawal_ticket_token_account.clone(),
                staker.clone(),
                vault_staker_withdrawal_ticket_info.clone(),
            ],
            &[&seed_slices],
        )?;

        close_program_account(program_id, vault_staker_withdrawal_ticket_info, staker)?;
    }

    vault.lrt_supply = vault
        .lrt_supply
        .checked_sub(lrt_to_burn)
        .ok_or(ProgramError::ArithmeticOverflow)?;
    // TODO (LB): https://github.com/jito-foundation/restaking/issues/24
    //  If a withdraw ticket is created and there is a slashing event before the withdraw ticket
    //  has fully matured, the program can end up in a situation where the original_redemption_amount
    //  is greater than the total withdrawable_reserve_amount. This is a bug and needs to be fixed.
    //  see test_burn_withdrawal_ticket_with_slashing_before_update
    msg!(
        "vault.withdrawable_reserve_amount before: {:?}",
        vault.withdrawable_reserve_amount
    );

    vault.withdrawable_reserve_amount = vault
        .withdrawable_reserve_amount
        .checked_sub(original_redemption_amount)
        .ok_or(ProgramError::ArithmeticOverflow)?;

    msg!(
        "vault.withdrawable_reserve_amount after: {:?}",
        vault.withdrawable_reserve_amount
    );

    vault.tokens_deposited = vault
        .tokens_deposited
        .checked_sub(actual_withdraw_amount)
        .ok_or(ProgramError::InsufficientFunds)?;

    // transfer the assets to the staker
    let (_, vault_bump, mut vault_seeds) = Vault::find_program_address(program_id, &vault.base);
    vault_seeds.push(vec![vault_bump]);
    let seed_slices: Vec<&[u8]> = vault_seeds.iter().map(|seed| seed.as_slice()).collect();
    drop(vault_data); // avoid double borrow
    invoke_signed(
        &transfer(
            &spl_token::id(),
            vault_token_account.key,
            staker_token_account.key,
            vault_info.key,
            &[],
            actual_withdraw_amount,
        )?,
        &[
            vault_token_account.clone(),
            staker_token_account.clone(),
            vault_info.clone(),
        ],
        &[&seed_slices],
    )?;

    Ok(())
}
