use std::mem::size_of;

use jito_account_traits::{AccountDeserialize, Discriminator};
use jito_jsm_core::{
    create_account,
    loader::{
        load_associated_token_account, load_signer, load_system_account, load_system_program,
        load_token_program,
    },
};
use jito_vault_core::{
    config::Config,
    loader::{load_config, load_vault, load_vault_delegation_list},
    vault::Vault,
    vault_delegation_list::{UndelegateForWithdrawMethod, VaultDelegationList},
    vault_staker_withdrawal_ticket::VaultStakerWithdrawalTicket,
};
use solana_program::{
    account_info::AccountInfo, clock::Clock, entrypoint::ProgramResult, msg, program::invoke,
    program_error::ProgramError, pubkey::Pubkey, rent::Rent, sysvar::Sysvar,
};
use spl_token::instruction::transfer;

/// Enqueues a withdraw into the VaultStakerWithdrawalTicket account, transferring the amount from the
/// staker's VRT token account to the VaultStakerWithdrawalTicket VRT token account. It also queues
/// the withdrawal in the vault's delegation list.
///
/// The most obvious options for withdrawing are calculating the redemption ratio and withdrawing
/// the exact amount of collateral from operators. This may not be ideal in the case where the VRT:token
/// ratio increases due to rewards. However, if the vault has excess collateral that isn't staked, the vault
/// can withdraw that excess and return it to the staker. If there's no excess, they can withdraw the
/// amount that was set aside for withdraw.
///
/// One should call the [`crate::VaultInstruction::UpdateVault`] instruction before running this instruction
/// to ensure that any rewards that were accrued are accounted for.
pub fn process_enqueue_withdrawal(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    vrt_amount: u64,
) -> ProgramResult {
    let (required_accounts, optional_accounts) = accounts.split_at(11);

    let [config, vault_info, vault_delegation_list, vault_staker_withdrawal_ticket, vault_staker_withdrawal_ticket_token_account, vault_fee_token_account, staker, staker_vrt_token_account, base, token_program, system_program] =
        required_accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    load_config(program_id, config, false)?;
    load_vault(program_id, vault_info, true)?;
    load_vault_delegation_list(program_id, vault_delegation_list, vault_info, true)?;
    load_system_account(vault_staker_withdrawal_ticket, true)?;
    let mut vault_data = vault_info.data.borrow_mut();
    let vault = Vault::try_from_slice_mut(&mut vault_data)?;
    load_associated_token_account(
        vault_staker_withdrawal_ticket_token_account,
        vault_staker_withdrawal_ticket.key,
        &vault.vrt_mint,
    )?;
    load_associated_token_account(vault_fee_token_account, &vault.fee_wallet, &vault.vrt_mint)?;
    load_signer(staker, false)?;
    load_associated_token_account(staker_vrt_token_account, staker.key, &vault.vrt_mint)?;
    load_signer(base, false)?;
    load_token_program(token_program)?;
    load_system_program(system_program)?;

    // The VaultStakerWithdrawalTicket shall be at the canonical PDA
    let (
        vault_staker_withdrawal_ticket_pubkey,
        vault_staker_withdrawal_ticket_bump,
        mut vault_staker_withdrawal_ticket_seeds,
    ) = VaultStakerWithdrawalTicket::find_program_address(
        program_id,
        vault_info.key,
        staker.key,
        base.key,
    );
    vault_staker_withdrawal_ticket_seeds.push(vec![vault_staker_withdrawal_ticket_bump]);
    if vault_staker_withdrawal_ticket
        .key
        .ne(&vault_staker_withdrawal_ticket_pubkey)
    {
        msg!("Vault staker withdrawal ticket is not at the correct PDA");
        return Err(ProgramError::InvalidAccountData);
    }

    // If a Vault mint_burn_admin is present, it shall be a signer on the transaction
    if vault.mint_burn_admin.ne(&Pubkey::default()) {
        if let Some(burn_signer) = optional_accounts.first() {
            load_signer(burn_signer, false)?;
            if burn_signer.key.ne(&vault.mint_burn_admin) {
                msg!("Burn signer does not match vault burn signer");
                return Err(ProgramError::InvalidAccountData);
            }
        } else {
            msg!("Mint signer is required for vault mint");
            return Err(ProgramError::InvalidAccountData);
        }
    }

    // The vault_delegation_list shall be up-to-date
    let config_data = config.data.borrow();
    let config = Config::try_from_slice(&config_data)?;
    let mut vault_delegation_list_data = vault_delegation_list.data.borrow_mut();
    let vault_delegation_list =
        VaultDelegationList::try_from_slice_mut(&mut vault_delegation_list_data)?;
    if vault_delegation_list.is_update_needed(Clock::get()?.slot, config.epoch_length) {
        msg!("Vault delegation list is not up to date");
        return Err(ProgramError::InvalidAccountData);
    }

    // Calculate the amount to undelegate for withdrawal for the user, subtracting the fee
    let fee_amount = vault.calculate_withdraw_fee(vrt_amount)?;
    let amount_to_vault_staker_withdrawal_ticket = vrt_amount
        .checked_sub(fee_amount)
        .ok_or(ProgramError::ArithmeticOverflow)?;
    let amount_to_withdraw =
        vault.calculate_assets_returned_amount(amount_to_vault_staker_withdrawal_ticket)?;
    vault_delegation_list
        .undelegate_for_withdrawal(amount_to_withdraw, UndelegateForWithdrawMethod::ProRata)?;

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
    let vault_staker_withdrawal_ticket =
        VaultStakerWithdrawalTicket::try_from_slice_mut(&mut vault_staker_withdrawal_ticket_data)?;
    *vault_staker_withdrawal_ticket = VaultStakerWithdrawalTicket::new(
        *vault_info.key,
        *staker.key,
        *base.key,
        amount_to_withdraw,
        amount_to_vault_staker_withdrawal_ticket,
        Clock::get()?.slot,
        vault_staker_withdrawal_ticket_bump,
    );

    // Withdraw funds from the staker's VRT account, transferring them to an ATA owned
    // by the VaultStakerWithdrawalTicket
    invoke(
        &transfer(
            &spl_token::id(),
            staker_vrt_token_account.key,
            vault_staker_withdrawal_ticket_token_account.key,
            staker.key,
            &[],
            amount_to_vault_staker_withdrawal_ticket,
        )?,
        &[
            staker_vrt_token_account.clone(),
            vault_staker_withdrawal_ticket_token_account.clone(),
            staker.clone(),
        ],
    )?;

    // Withdraw the fee from the staker's VRT account, transferring them to an ATA owned
    // by the VaultStakerWithdrawalTicket
    invoke(
        &transfer(
            &spl_token::id(),
            staker_vrt_token_account.key,
            vault_fee_token_account.key,
            staker.key,
            &[],
            fee_amount,
        )?,
        &[
            staker_vrt_token_account.clone(),
            vault_fee_token_account.clone(),
            staker.clone(),
        ],
    )?;

    Ok(())
}
