use jito_bytemuck::AccountDeserialize;
use jito_jsm_core::loader::{
    load_associated_token_account, load_signer, load_token_mint, load_token_program,
};
use jito_vault_core::{
    config::Config,
    vault::{MintSummary, Vault},
};
use jito_vault_sdk::error::VaultError;
use solana_program::{
    account_info::AccountInfo,
    clock::Clock,
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvar::Sysvar,
};
use spl_token::instruction::{mint_to, transfer};

use crate::update_vault_balance::process_update_vault_balance;

/// Processes the mint instruction: [`crate::VaultInstruction::MintTo`]
///
/// Note: it's strongly encouraged to call [`jito_vault_sdk::instruction::VaultInstruction::UpdateVaultBalance`] before calling this instruction to ensure
/// the vault state is up-to-date.
///
/// Specification:
/// - If the vault has a mint burn admin, it must match be present and be a signer
/// - The vault must be up-to-date
/// - The vault VRT mint must be correct
/// - The amount to mint must be greater than zero
/// - The post-mint tokens deposited shall be less than or equal to the vault capacity
/// - The vault fee wallet must get the fee amount
/// - The transaction shall fail if the amount out is less than the minimum amount out
/// - The user's assets shall be deposited into the vault supported mint ATA
/// - The vault shall mint the pro-rata amount to the user and the fee wallet
pub fn process_mint(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount_in: u64,
    min_amount_out: u64,
) -> ProgramResult {
    let (required_accounts, optional_accounts) = accounts.split_at(9);

    let [config, vault_info, vrt_mint, depositor, depositor_token_account, vault_token_account, depositor_vrt_token_account, vault_fee_token_account, token_program] =
        required_accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // Pseudo CPI to update the vault balance
    {
        let accounts = [
            config.clone(),
            vault_info.clone(),
            vault_token_account.clone(),
            vrt_mint.clone(),
            vault_fee_token_account.clone(),
            token_program.clone(),
        ];
        process_update_vault_balance(program_id, &accounts)?;
    }

    Config::load(program_id, config, false)?;
    let config_data = config.data.borrow();
    let config = Config::try_from_slice_unchecked(&config_data)?;
    Vault::load(program_id, vault_info, true)?;
    let mut vault_data = vault_info.data.borrow_mut();
    let vault = Vault::try_from_slice_unchecked_mut(&mut vault_data)?;

    load_token_mint(vrt_mint)?;
    load_signer(depositor, false)?;
    load_associated_token_account(
        depositor_token_account,
        depositor.key,
        &vault.supported_mint,
    )?;
    load_associated_token_account(vault_token_account, vault_info.key, &vault.supported_mint)?;
    load_associated_token_account(depositor_vrt_token_account, depositor.key, vrt_mint.key)?;
    load_associated_token_account(vault_fee_token_account, &vault.fee_wallet, vrt_mint.key)?;

    // Only the original spl token program is allowed
    load_token_program(token_program)?;

    vault.check_mint_burn_admin(optional_accounts.first())?;
    vault.check_vrt_mint(vrt_mint.key)?;
    vault.check_update_state_ok(Clock::get()?.slot, config.epoch_length())?;
    vault.check_is_paused()?;

    // Currently, this is not possible, since the there are currently no instructions that allow the
    // vault to deposit tokens into the vault token account. This check is for future proofing.
    if depositor.key.eq(vault_info.key) {
        msg!("Depositor cannot be the vault");
        return Err(VaultError::InvalidDepositor.into());
    }

    // If the depositor token account is the same as the vault token account, there would be a zero balance change transfer
    // coupled with a `amount_in` mint. This would allow the vault to inflate it's the VRT's value.
    // vault -> 10 ST -> vault = +`amount_in` VRT
    if depositor_token_account.key.eq(vault_token_account.key) {
        msg!("Depositor token account cannot be the vault token account");
        return Err(VaultError::InvalidDepositTokenAccount.into());
    }

    let MintSummary {
        vrt_to_depositor,
        vrt_to_fee_wallet,
    } = vault.mint_with_fee(amount_in, min_amount_out)?;

    if vrt_to_depositor == 0 {
        msg!("Some VRT must be minted to the depositor. If you wish to donate to the vault, please send ST directly to the vault token account");
        return Err(VaultError::VrtOutCannotBeZero.into());
    }

    // transfer tokens from depositor to vault
    {
        invoke(
            &transfer(
                &spl_token::id(),
                depositor_token_account.key,
                vault_token_account.key,
                depositor.key,
                &[],
                amount_in,
            )?,
            &[
                depositor_token_account.clone(),
                vault_token_account.clone(),
                depositor.clone(),
            ],
        )?;
    }

    let signing_seeds = vault.signing_seeds();
    let seed_slices: Vec<&[u8]> = signing_seeds.iter().map(|seed| seed.as_slice()).collect();

    drop(vault_data); // no double borrow

    // mint to depositor and fee wallet
    {
        invoke_signed(
            &mint_to(
                &spl_token::id(),
                vrt_mint.key,
                depositor_vrt_token_account.key,
                vault_info.key,
                &[],
                vrt_to_depositor,
            )?,
            &[
                vrt_mint.clone(),
                depositor_vrt_token_account.clone(),
                vault_info.clone(),
            ],
            &[&seed_slices],
        )?;

        invoke_signed(
            &mint_to(
                &spl_token::id(),
                vrt_mint.key,
                vault_fee_token_account.key,
                vault_info.key,
                &[],
                vrt_to_fee_wallet,
            )?,
            &[
                vrt_mint.clone(),
                vault_fee_token_account.clone(),
                vault_info.clone(),
            ],
            &[&seed_slices],
        )?;
    }

    Ok(())
}
