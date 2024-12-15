use std::mem::size_of;

use jito_bytemuck::{AccountDeserialize, Discriminator};
use jito_jsm_core::{
    create_account,
    loader::{
        load_associated_token_account, load_signer, load_system_account, load_system_program,
        load_token_mint, load_token_program,
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
use spl_token::instruction::{freeze_account, transfer};
use spl_token_2022::instruction::thaw_account;

use crate::enqueue_withdrawal::process_enqueue_withdrawal;

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
pub fn process_enqueue_withdrawal_frozen(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    vrt_amount: u64,
) -> ProgramResult {
    // Check Mint Okay
    {
        let (required_accounts, optional_accounts) = accounts.split_at(10);

        let [config, vault_info, vault_staker_withdrawal_ticket, vault_staker_withdrawal_ticket_token_account, staker, staker_vrt_token_account, base, token_program, system_program, vrt_mint] =
            required_accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        load_token_mint(vrt_mint)?;

        Vault::load(program_id, vault_info, true)?;
        let vault_data = vault_info.data.borrow();
        let vault = Vault::try_from_slice_unchecked(&vault_data)?;

        vault.check_vrt_mint(vrt_mint.key)?;

        if !vault.is_frozen() {
            msg!("Vault is not frozen");
            return Err(VaultError::VaultNotFrozen.into());
        }
    }

    // Unfreeze Token Account
    {
        let [config, vault_info, vault_staker_withdrawal_ticket, vault_staker_withdrawal_ticket_token_account, staker, staker_vrt_token_account, base, token_program, system_program, _, vrt_mint] =
            accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        Vault::load(program_id, vault_info, true)?;
        let vault_data = vault_info.data.borrow();
        let vault = Vault::try_from_slice_unchecked(&vault_data)?;

        let signing_seeds = vault.signing_seeds();
        let seed_slices: Vec<&[u8]> = signing_seeds.iter().map(|seed| seed.as_slice()).collect();

        {
            invoke_signed(
                &thaw_account(
                    &spl_token::id(),
                    staker_vrt_token_account.key,
                    vrt_mint.key,
                    vault_info.key,
                    &[],
                )?,
                &[
                    vrt_mint.clone(),
                    staker_vrt_token_account.clone(),
                    vault_info.clone(),
                ],
                &[&seed_slices],
            )?;
        }
    }

    process_enqueue_withdrawal(program_id, accounts, vrt_amount)?;

    // Refreeze Token Account
    {
        let (required_accounts, optional_accounts) = accounts.split_at(10);

        let [config, vault_info, vault_staker_withdrawal_ticket, vault_staker_withdrawal_ticket_token_account, staker, staker_vrt_token_account, base, token_program, system_program, vrt_mint] =
            required_accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        Vault::load(program_id, vault_info, true)?;
        let vault_data = vault_info.data.borrow();
        let vault = Vault::try_from_slice_unchecked(&vault_data)?;

        let signing_seeds = vault.signing_seeds();
        let seed_slices: Vec<&[u8]> = signing_seeds.iter().map(|seed| seed.as_slice()).collect();

        {
            invoke_signed(
                &freeze_account(
                    &spl_token::id(),
                    staker_vrt_token_account.key,
                    vrt_mint.key,
                    vault_info.key,
                    &[],
                )?,
                &[
                    vrt_mint.clone(),
                    staker_vrt_token_account.clone(),
                    vault_info.clone(),
                ],
                &[&seed_slices],
            )?;
        }
    }

    Ok(())
}
