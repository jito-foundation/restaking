use std::cmp::min;

use jito_account_traits::AccountDeserialize;
use jito_jsm_core::{
    close_program_account,
    loader::{
        load_associated_token_account, load_signer, load_system_program, load_token_mint,
        load_token_program,
    },
};
use jito_vault_core::{
    config::Config, vault::Vault, vault_staker_withdrawal_ticket::VaultStakerWithdrawalTicket,
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
/// One should call the [`crate::VaultInstruction::CrankVaultUpdateStateTracker`] instruction before running this instruction
/// to ensure that any rewards that were accrued are accounted for.
pub fn process_burn_withdrawal_ticket(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    min_amount_out: u64,
) -> ProgramResult {
    let [config, vault_info, vault_token_account, vrt_mint, staker, staker_token_account, staker_vrt_token_account, vault_staker_withdrawal_ticket_info, vault_staker_withdrawal_ticket_token_account, token_program, system_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    Config::load(program_id, config, false)?;
    Vault::load(program_id, vault_info, true)?;
    let mut vault_data = vault_info.data.borrow_mut();
    let vault = Vault::try_from_slice_unchecked_mut(&mut vault_data)?;
    load_associated_token_account(vault_token_account, vault_info.key, &vault.supported_mint)?;
    load_token_mint(vrt_mint)?;
    load_signer(staker, false)?;
    load_associated_token_account(staker_token_account, staker.key, &vault.supported_mint)?;
    load_associated_token_account(staker_vrt_token_account, staker.key, &vault.vrt_mint)?;
    VaultStakerWithdrawalTicket::load(
        program_id,
        vault_staker_withdrawal_ticket_info,
        vault_info,
        staker,
        true,
    )?;
    load_associated_token_account(
        vault_staker_withdrawal_ticket_token_account,
        vault_staker_withdrawal_ticket_info.key,
        &vault.vrt_mint,
    )?;
    load_token_program(token_program)?;
    load_system_program(system_program)?;

    let config_data = config.data.borrow();
    let config = Config::try_from_slice_unchecked(&config_data)?;

    if vault.vrt_mint.ne(vrt_mint.key) {
        msg!("Vault VRT mint mismatch");
        return Err(ProgramError::InvalidArgument);
    }
    if vault.is_update_needed(Clock::get()?.slot, config.epoch_length) {
        msg!("Vault update needed");
        return Err(VaultError::VaultUpdateNeeded.into());
    }

    let vault_staker_withdrawal_ticket_data = vault_staker_withdrawal_ticket_info.data.borrow();
    let vault_staker_withdrawal_ticket = VaultStakerWithdrawalTicket::try_from_slice_unchecked(
        &vault_staker_withdrawal_ticket_data,
    )?;
    if !vault_staker_withdrawal_ticket.is_withdrawable(Clock::get()?.slot, config.epoch_length)? {
        msg!("Vault staker withdrawal ticket is not withdrawable");
        return Err(VaultError::VaultStakerWithdrawalTicketNotWithdrawable.into());
    }

    let redemption_amount =
        vault.calculate_assets_returned_amount(vault_staker_withdrawal_ticket.vrt_amount)?;
    let max_withdrawable = vault
        .tokens_deposited
        .checked_sub(vault.delegation_state.total_security()?)
        .ok_or(VaultError::VaultUnderflow)?;

    let amount_to_withdraw = min(redemption_amount, max_withdrawable);
    if amount_to_withdraw < min_amount_out {
        msg!(
            "Slippage error, expected more than {} out, got {}",
            min_amount_out,
            amount_to_withdraw
        );
        return Err(VaultError::SlippageError.into());
    }
    let vrt_to_burn = vault.calculate_vrt_mint_amount(amount_to_withdraw)?;

    vault.vrt_supply = vault
        .vrt_supply
        .checked_sub(vrt_to_burn)
        .ok_or(ProgramError::ArithmeticOverflow)?;
    vault.tokens_deposited = vault
        .tokens_deposited
        .checked_sub(amount_to_withdraw)
        .ok_or(ProgramError::ArithmeticOverflow)?;
    vault.vrt_ready_to_claim_amount = vault
        .vrt_ready_to_claim_amount
        .checked_sub(vault_staker_withdrawal_ticket.vrt_amount)
        .ok_or(ProgramError::ArithmeticOverflow)?;

    // burn the VRT tokens
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
    // burn the VRT tokens
    invoke_signed(
        &burn(
            &spl_token::id(),
            vault_staker_withdrawal_ticket_token_account.key,
            vrt_mint.key,
            vault_staker_withdrawal_ticket_info.key,
            &[],
            vrt_to_burn,
        )?,
        &[
            vault_staker_withdrawal_ticket_token_account.clone(),
            vrt_mint.clone(),
            vault_staker_withdrawal_ticket_info.clone(),
        ],
        &[&seed_slices],
    )?;

    let vrt_token_excess_amount =
        Account::unpack(&vault_staker_withdrawal_ticket_token_account.data.borrow())?.amount;
    if vrt_token_excess_amount > 0 {
        invoke_signed(
            &transfer(
                &spl_token::id(),
                vault_staker_withdrawal_ticket_token_account.key,
                staker_vrt_token_account.key,
                vault_staker_withdrawal_ticket_info.key,
                &[],
                vrt_token_excess_amount,
            )?,
            &[
                vault_staker_withdrawal_ticket_token_account.clone(),
                staker_vrt_token_account.clone(),
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
            amount_to_withdraw,
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
