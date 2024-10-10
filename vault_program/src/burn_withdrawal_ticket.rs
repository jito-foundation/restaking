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
    program::invoke_signed, program_error::ProgramError, pubkey::Pubkey, sysvar::Sysvar,
};
use spl_token::instruction::{burn, close_account, transfer};

/// Burns the withdrawal ticket, transferring the assets to the staker and closing the withdrawal ticket.
///
/// One should call the [`crate::VaultInstruction::CrankVaultUpdateStateTracker`] instruction before running this instruction
/// to ensure that any rewards that were accrued are accounted for.
pub fn process_burn_withdrawal_ticket(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    min_amount_out: u64,
) -> ProgramResult {
    let (required_accounts, optional_accounts) = accounts.split_at(12);
    let [config, vault_info, vault_token_account, vrt_mint, staker, staker_token_account, vault_staker_withdrawal_ticket_info, vault_staker_withdrawal_ticket_token_account, vault_fee_token_account, program_fee_token_account, token_program, system_program] =
        required_accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    msg!("loading config");
    Config::load(program_id, config, false)?;
    let config_data = config.data.borrow();
    let config = Config::try_from_slice_unchecked(&config_data)?;
    msg!("loading vault");
    Vault::load(program_id, vault_info, true)?;
    let mut vault_data = vault_info.data.borrow_mut();
    let vault = Vault::try_from_slice_unchecked_mut(&mut vault_data)?;
    msg!("loading vault token account");
    load_associated_token_account(vault_token_account, vault_info.key, &vault.supported_mint)?;
    msg!("loading vrt mint");
    load_token_mint(vrt_mint)?;
    // staker
    msg!("loading staker token account");
    load_associated_token_account(staker_token_account, staker.key, &vault.supported_mint)?;
    msg!("loading vault staker withdrawal ticket");
    VaultStakerWithdrawalTicket::load(
        program_id,
        vault_staker_withdrawal_ticket_info,
        vault_info,
        true,
    )?;
    let vault_staker_withdrawal_ticket_data = vault_staker_withdrawal_ticket_info.data.borrow();
    let vault_staker_withdrawal_ticket = VaultStakerWithdrawalTicket::try_from_slice_unchecked(
        &vault_staker_withdrawal_ticket_data,
    )?;
    msg!("loading vault staker withdrawal ticket token account");
    load_associated_token_account(
        vault_staker_withdrawal_ticket_token_account,
        vault_staker_withdrawal_ticket_info.key,
        &vault.vrt_mint,
    )?;
    msg!("loading vault fee token account");
    load_associated_token_account(vault_fee_token_account, &vault.fee_wallet, &vault.vrt_mint)?;
    msg!("loading program fee token account");
    load_associated_token_account(
        program_fee_token_account,
        &config.program_fee_wallet,
        &vault.vrt_mint,
    )?;
    msg!("loading token program");
    load_token_program(token_program)?;
    msg!("loading system program");
    load_system_program(system_program)?;

    vault.check_mint_burn_admin(optional_accounts.first())?;
    vault.check_vrt_mint(vrt_mint.key)?;
    vault.check_update_state_ok(Clock::get()?.slot, config.epoch_length())?;
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
    } = vault.burn_with_fee(
        config.program_fee_bps(),
        vault_staker_withdrawal_ticket.vrt_amount(),
        min_amount_out,
    )?;
    msg!("Decrementing VRT ready to claim amount");
    vault.decrement_vrt_ready_to_claim_amount(vault_staker_withdrawal_ticket.vrt_amount())?;

    msg!("Finding program address for VaultStakerWithdrawalTicket");
    let (_, vault_staker_withdraw_bump, mut vault_staker_withdraw_seeds) =
        VaultStakerWithdrawalTicket::find_program_address(
            program_id,
            vault_info.key,
            &vault_staker_withdrawal_ticket.base,
        );
    vault_staker_withdraw_seeds.push(vec![vault_staker_withdraw_bump]);
    let seed_slices: Vec<&[u8]> = vault_staker_withdraw_seeds
        .iter()
        .map(|seed| seed.as_slice())
        .collect();
    drop(vault_staker_withdrawal_ticket_data);

    // transfer fee to fee wallet
    msg!("Transferring fee to fee wallet");
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
    msg!("Transferring program fee to program fee wallet");
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
    msg!("Burning VRT tokens");
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
    msg!("Closing token account");
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
    msg!("Closing program account");
    close_program_account(program_id, vault_staker_withdrawal_ticket_info, staker)?;

    // transfer the assets to the staker
    msg!("Finding program address for Vault");
    let (_, vault_bump, mut vault_seeds) = Vault::find_program_address(program_id, &vault.base);
    vault_seeds.push(vec![vault_bump]);
    let seed_slices: Vec<&[u8]> = vault_seeds.iter().map(|seed| seed.as_slice()).collect();
    drop(vault_data); // avoid double borrow
    msg!("Transferring assets to staker");
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
