#![allow(dead_code)]

use jito_vault_core::{vault::Vault, vault_staker_withdrawal_ticket::VaultStakerWithdrawalTicket};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
    pubkey::Pubkey,
};

pub fn spl_token_account_amount(vault_token_account: &AccountInfo) -> Result<u64, ProgramError> {
    Ok(cvlr_solana::token::spl_token_account_get_amount(
        &vault_token_account,
    ))
}

pub fn mint_vrt_rewards_to_fee_wallet<'a>(
    _program_id: &Pubkey,
    vrt_mint: &AccountInfo<'a>,
    vault_fee_token_account: &AccountInfo<'a>,
    vault_info: &AccountInfo<'a>,
    amount: u64,
    _seed_slices: &Vec<&[u8]>,
) -> ProgramResult {
    cvlr_solana::token::spl_mint_to(vrt_mint, vault_fee_token_account, vault_info, amount)
}

pub fn transfer_from_depositor_to_vault<'a>(
    _program_id: &Pubkey,
    depositor_token_account: &AccountInfo<'a>,
    vault_token_account: &AccountInfo<'a>,
    depositor: &AccountInfo<'a>,
    amount: u64,
) -> ProgramResult {
    cvlr_solana::token::spl_token_transfer(
        depositor_token_account,
        vault_token_account,
        depositor,
        amount,
    )
}

pub fn mint_to_depositor<'a>(
    _program_id: &Pubkey,
    vrt_mint: &AccountInfo<'a>,
    depositor_vrt_token_account: &AccountInfo<'a>,
    vault_info: &AccountInfo<'a>,
    amount: u64,
    _seed_slices: &Vec<&[u8]>,
) -> ProgramResult {
    cvlr_solana::token::spl_mint_to(vrt_mint, depositor_vrt_token_account, vault_info, amount)
}

pub fn mint_to_fee_wallet<'a>(
    _program_id: &Pubkey,
    vrt_mint: &AccountInfo<'a>,
    vault_fee_token_account: &AccountInfo<'a>,
    vault_info: &AccountInfo<'a>,
    amount: u64,
    _seed_slices: &Vec<&[u8]>,
) -> ProgramResult {
    cvlr_solana::token::spl_mint_to(vrt_mint, vault_fee_token_account, vault_info, amount)
}

pub fn transfer_staker_to_ata_account<'a>(
    _program_id: &Pubkey,
    staker_vrt_token_account: &AccountInfo<'a>,
    vault_staker_withdrawal_ticket_token_account: &AccountInfo<'a>,
    staker: &AccountInfo<'a>,
    amount: u64,
) -> ProgramResult {
    cvlr_solana::token::spl_token_transfer(
        staker_vrt_token_account,
        vault_staker_withdrawal_ticket_token_account,
        staker,
        amount,
    )
}

/// Summary for SPL Token transfer fee to fee wallet
pub fn transfer_fee_to_fee_wallet<'a>(
    _program_id: &Pubkey,
    vault_staker_withdrawal_ticket_token_account: &AccountInfo<'a>,
    fee_token_account: &AccountInfo<'a>,
    vault_staker_withdrawal_ticket_info: &AccountInfo<'a>,
    _seed_slices: &Vec<&[u8]>,
    amount: u64,
) -> ProgramResult {
    cvlr_solana::token::spl_token_transfer(
        vault_staker_withdrawal_ticket_token_account,
        fee_token_account,
        vault_staker_withdrawal_ticket_info,
        amount,
    )
}

pub fn get_vrt_amount_from_vault_staker_withdrawal_ticket(
    vault_staker_withdrawal_ticket_token_account: &AccountInfo,
) -> Result<u64, ProgramError> {
    let ticket_vrt_amount = cvlr_solana::token::spl_token_account_get_amount(
        &vault_staker_withdrawal_ticket_token_account,
    );
    Ok(ticket_vrt_amount)
}

pub fn burn_vrt<'a>(
    _program_id: &Pubkey,
    vault_staker_withdrawal_ticket_token_account: &AccountInfo<'a>,
    vrt_mint: &AccountInfo<'a>,
    vault_staker_withdrawal_ticket_info: &AccountInfo<'a>,
    _seed_slices: &Vec<&[u8]>,
    amount: u64,
) -> ProgramResult {
    cvlr_solana::token::spl_burn(
        vrt_mint,
        vault_staker_withdrawal_ticket_token_account,
        vault_staker_withdrawal_ticket_info,
        amount,
    )
}

pub fn close_withdrawal_ticket_token_account<'a>(
    _program_id: &Pubkey,
    vault_staker_withdrawal_ticket_token_account: &AccountInfo<'a>,
    staker: &AccountInfo<'a>,
    vault_staker_withdrawal_ticket_info: &AccountInfo<'a>,
    _seed_slices: &Vec<&[u8]>,
) -> ProgramResult {
    cvlr_solana::token::spl_close_account(
        vault_staker_withdrawal_ticket_token_account,
        staker,
        vault_staker_withdrawal_ticket_info,
    )
}

pub fn transfer_from_vault_to_staker<'a>(
    _program_id: &Pubkey,
    vault_token_account: &AccountInfo<'a>,
    staker_token_account: &AccountInfo<'a>,
    vault_info: &AccountInfo<'a>,
    _seed_slices: &Vec<&[u8]>,
    amount: u64,
) -> ProgramResult {
    cvlr_solana::token::spl_token_transfer(
        vault_token_account,
        staker_token_account,
        vault_info,
        amount,
    )
}

pub fn staker_withdrawal_ticket_signing_seeds(
    _vault_staker_withdrawal_ticket: &VaultStakerWithdrawalTicket,
    _vault: &Pubkey,
) -> Vec<Vec<u8>> {
    vec![]
}

pub fn vault_signing_seeds(_vault: &Vault) -> Vec<Vec<u8>> {
    vec![]
}
