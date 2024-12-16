use jito_bytemuck::AccountDeserialize;
use jito_jsm_core::{
    close_program_account,
    loader::{
        load_associated_token_account, load_system_program, load_token_mint, load_token_program,
    },
};
use jito_vault_core::{
    config::Config,
    vault::{BurnSummary, Vault},
    vault_staker_withdrawal_ticket::VaultStakerWithdrawalTicket,
};
use jito_vault_sdk::error::VaultError;
use solana_program::{
    account_info::AccountInfo, clock::Clock, entrypoint::ProgramResult, msg,
    program::invoke_signed, program_error::ProgramError, program_pack::Pack, pubkey::Pubkey,
    sysvar::Sysvar,
};
use spl_token::instruction::{burn, close_account, transfer};
use spl_token_2022::state::Account;

/// Burns the withdrawal ticket, transferring the assets to the staker and closing the withdrawal ticket.
///
/// One should call the [`crate::VaultInstruction::CrankVaultUpdateStateTracker`] instruction before running this instruction
/// to ensure that any rewards that were accrued are accounted for.
pub fn process_burn_withdrawal_ticket(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let (required_accounts, optional_accounts) = accounts.split_at(12);
    let [config, vault_info, vault_token_account, vrt_mint, staker, staker_token_account, vault_staker_withdrawal_ticket_info, vault_staker_withdrawal_ticket_token_account, vault_fee_token_account, program_fee_token_account, token_program, system_program] =
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
    load_associated_token_account(vault_token_account, vault_info.key, &vault.supported_mint)?;
    load_token_mint(vrt_mint)?;

    // staker
    load_associated_token_account(staker_token_account, staker.key, &vault.supported_mint)?;
    VaultStakerWithdrawalTicket::load(program_id, vault_staker_withdrawal_ticket_info, true)?;
    let vault_staker_withdrawal_ticket_data = vault_staker_withdrawal_ticket_info.data.borrow();
    let vault_staker_withdrawal_ticket = VaultStakerWithdrawalTicket::try_from_slice_unchecked(
        &vault_staker_withdrawal_ticket_data,
    )?;
    load_associated_token_account(
        vault_staker_withdrawal_ticket_token_account,
        vault_staker_withdrawal_ticket_info.key,
        &vault.vrt_mint,
    )?;

    let ticket_vrt_account =
        Account::unpack(&vault_staker_withdrawal_ticket_token_account.data.borrow())?;
    let ticket_vrt_amount = ticket_vrt_account.amount;

    load_associated_token_account(vault_fee_token_account, &vault.fee_wallet, &vault.vrt_mint)?;
    load_associated_token_account(
        program_fee_token_account,
        &config.program_fee_wallet,
        &vault.vrt_mint,
    )?;
    // Only the original spl token program is allowed
    load_token_program(token_program)?;

    load_system_program(system_program)?;

    vault.check_mint_burn_admin(optional_accounts.first())?;
    vault.check_vrt_mint(vrt_mint.key)?;
    vault.check_update_state_ok(Clock::get()?.slot, config.epoch_length())?;
    vault.check_is_paused()?;

    vault_staker_withdrawal_ticket.check_staker(staker.key)?;

    if !vault_staker_withdrawal_ticket.is_withdrawable(Clock::get()?.slot, config.epoch_length())? {
        msg!("Vault staker withdrawal ticket is not withdrawable");
        return Err(VaultError::VaultStakerWithdrawalTicketNotWithdrawable.into());
    }

    let BurnSummary {
        vault_fee_amount,
        program_fee_amount,
        burn_amount,
        out_amount,
    } = vault.burn_with_fee(vault_staker_withdrawal_ticket.vrt_amount())?;

    // To close the token account, the balance needs to be 0.
    // The only way for vault_staker_withdrawal_ticket.vrt_amount() != ticket_vrt_amount
    // Is if some party sent VRT to the ticket account after the ticket was created.
    // This extra VRT will be sent to the program fee wallet.
    let extra_vrt_in_account = ticket_vrt_amount
        .checked_sub(vault_staker_withdrawal_ticket.vrt_amount())
        .ok_or(VaultError::ArithmeticUnderflow)?;

    let program_fee_amount = program_fee_amount
        .checked_add(extra_vrt_in_account)
        .ok_or(VaultError::ArithmeticOverflow)?;

    vault.decrement_vrt_ready_to_claim_amount(vault_staker_withdrawal_ticket.vrt_amount())?;

    let vault_staker_withdrawal_seeds = vault_staker_withdrawal_ticket.signing_seeds();
    let seed_slices: Vec<&[u8]> = vault_staker_withdrawal_seeds
        .iter()
        .map(|seed| seed.as_slice())
        .collect();
    drop(vault_staker_withdrawal_ticket_data);

    // transfer fee to fee wallet
    invoke_signed(
        &transfer(
            &spl_token::id(),
            vault_staker_withdrawal_ticket_token_account.key,
            vault_fee_token_account.key,
            vault_staker_withdrawal_ticket_info.key,
            &[],
            vault_fee_amount,
        )?,
        &[
            vault_staker_withdrawal_ticket_token_account.clone(),
            vault_fee_token_account.clone(),
            vault_staker_withdrawal_ticket_info.clone(),
        ],
        &[&seed_slices],
    )?;
    // Transfer program fee to program fee wallet
    invoke_signed(
        &transfer(
            &spl_token::id(),
            vault_staker_withdrawal_ticket_token_account.key,
            program_fee_token_account.key,
            vault_staker_withdrawal_ticket_info.key,
            &[],
            program_fee_amount,
        )?,
        &[
            vault_staker_withdrawal_ticket_token_account.clone(),
            program_fee_token_account.clone(),
            vault_staker_withdrawal_ticket_info.clone(),
        ],
        &[&seed_slices],
    )?;

    // burn the VRT tokens
    invoke_signed(
        &burn(
            &spl_token::id(),
            vault_staker_withdrawal_ticket_token_account.key,
            vrt_mint.key,
            vault_staker_withdrawal_ticket_info.key,
            &[],
            burn_amount,
        )?,
        &[
            vault_staker_withdrawal_ticket_token_account.clone(),
            vrt_mint.clone(),
            vault_staker_withdrawal_ticket_info.clone(),
        ],
        &[&seed_slices],
    )?;

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
    let vault_signer_seeds = vault.signing_seeds();
    let seed_slices: Vec<&[u8]> = vault_signer_seeds
        .iter()
        .map(|seed| seed.as_slice())
        .collect();

    drop(vault_data); // avoid double borrow

    invoke_signed(
        &transfer(
            &spl_token::id(),
            vault_token_account.key,
            staker_token_account.key,
            vault_info.key,
            &[],
            out_amount,
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
