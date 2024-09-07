use std::mem::size_of;

use jito_bytemuck::{AccountDeserialize, Discriminator};
use jito_jsm_core::{
    create_account,
    loader::{
        load_associated_token_account, load_signer, load_system_account, load_system_program,
        load_token_program,
    },
};
use jito_vault_core::{
    config::Config, vault::Vault, vault_staker_withdrawal_ticket::VaultStakerWithdrawalTicket,
};
use jito_vault_sdk::error::VaultError;
use solana_program::{
    account_info::AccountInfo, clock::Clock, entrypoint::ProgramResult, msg, program::invoke,
    program_error::ProgramError, pubkey::Pubkey, rent::Rent, sysvar::Sysvar,
};
use spl_token::instruction::transfer;

/// Enqueues a withdraw into the VaultStakerWithdrawalTicket account, transferring the amount from the
/// staker's VRT token account to the VaultStakerWithdrawalTicket VRT token account.
///
/// Specification:
/// - If the vault has a mint burn admin, it shall be present and be a signer of the transaction
/// - The vault shall be up to date
/// - The amount to withdraw must be greater than zero
/// - The VaultStakerWithdrawalTicket account shall be at the canonical PDA
/// - The vault shall accurately track the amount of VRT that has been enqueued for cooldown
/// - The staker's VRT tokens shall be transferred to the VaultStakerWithdrawalTicket associated token account
pub fn process_enqueue_withdrawal(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    vrt_amount: u64,
) -> ProgramResult {
    let (required_accounts, optional_accounts) = accounts.split_at(9);

    let [config, vault_info, vault_staker_withdrawal_ticket, vault_staker_withdrawal_ticket_token_account, staker, staker_vrt_token_account, base, token_program, system_program] =
        required_accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    Config::load(program_id, config, false)?;
    let config_data = config.data.borrow();
    let config = Config::try_from_slice_unchecked(&config_data)?;
    Vault::load(program_id, vault_info, true)?;
    let mut vault_data = vault_info.data.borrow_mut();
    let vault = Vault::try_from_slice_unchecked_mut(&mut vault_data)?;
    load_system_account(vault_staker_withdrawal_ticket, true)?;
    load_associated_token_account(
        vault_staker_withdrawal_ticket_token_account,
        vault_staker_withdrawal_ticket.key,
        &vault.vrt_mint,
    )?;
    load_signer(staker, false)?;
    load_associated_token_account(staker_vrt_token_account, staker.key, &vault.vrt_mint)?;
    load_signer(base, false)?;
    load_token_program(token_program)?;
    load_system_program(system_program)?;

    vault.check_mint_burn_admin(optional_accounts.first())?;
    vault.check_update_state_ok(Clock::get()?.slot, config.epoch_length())?;
    vault.check_is_paused()?;

    if vrt_amount == 0 {
        msg!("VRT amount must be greater than zero");
        return Err(VaultError::VaultEnqueueWithdrawalAmountZero.into());
    }

    // The VaultStakerWithdrawalTicket shall be at the canonical PDA
    let (
        vault_staker_withdrawal_ticket_pubkey,
        vault_staker_withdrawal_ticket_bump,
        mut vault_staker_withdrawal_ticket_seeds,
    ) = VaultStakerWithdrawalTicket::find_program_address(program_id, vault_info.key, base.key);
    vault_staker_withdrawal_ticket_seeds.push(vec![vault_staker_withdrawal_ticket_bump]);
    if vault_staker_withdrawal_ticket
        .key
        .ne(&vault_staker_withdrawal_ticket_pubkey)
    {
        msg!("Vault staker withdrawal ticket is not at the correct PDA");
        return Err(ProgramError::InvalidAccountData);
    }

    // Create the VaultStakerWithdrawalTicket account
    msg!(
        "Initializing vault staker withdraw ticket at address {}",
        vault_staker_withdrawal_ticket.key
    );
    create_account(
        staker,
        vault_staker_withdrawal_ticket,
        system_program,
        program_id,
        &Rent::get()?,
        8_u64
            .checked_add(size_of::<VaultStakerWithdrawalTicket>() as u64)
            .unwrap(),
        &vault_staker_withdrawal_ticket_seeds,
    )?;
    let mut vault_staker_withdrawal_ticket_data = vault_staker_withdrawal_ticket.data.borrow_mut();
    vault_staker_withdrawal_ticket_data[0] = VaultStakerWithdrawalTicket::DISCRIMINATOR;
    let vault_staker_withdrawal_ticket = VaultStakerWithdrawalTicket::try_from_slice_unchecked_mut(
        &mut vault_staker_withdrawal_ticket_data,
    )?;
    *vault_staker_withdrawal_ticket = VaultStakerWithdrawalTicket::new(
        *vault_info.key,
        *staker.key,
        *base.key,
        vrt_amount,
        Clock::get()?.slot,
        vault_staker_withdrawal_ticket_bump,
    );

    vault.increment_vrt_enqueued_for_cooldown_amount(vrt_amount)?;

    // Withdraw funds from the staker's VRT account, transferring them to an ATA owned
    // by the VaultStakerWithdrawalTicket
    invoke(
        &transfer(
            &spl_token::id(),
            staker_vrt_token_account.key,
            vault_staker_withdrawal_ticket_token_account.key,
            staker.key,
            &[],
            vrt_amount,
        )?,
        &[
            staker_vrt_token_account.clone(),
            vault_staker_withdrawal_ticket_token_account.clone(),
            staker.clone(),
        ],
    )?;

    Ok(())
}
