use jito_account_traits::AccountDeserialize;
use jito_jsm_core::loader::{
    load_associated_token_account, load_signer, load_token_mint, load_token_program,
};
use jito_vault_core::{
    loader::{load_config, load_vault},
    vault::Vault,
};
use solana_program::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    pubkey::Pubkey,
};
use spl_token::instruction::{mint_to, transfer};

/// Processes the mint instruction: [`crate::VaultInstruction::MintTo`]
///
/// Note: it's strongly encouraged to call [`crate::VaultInstruction::UpdateVault`] before calling this instruction to ensure
/// the vault state is up-to-date.
pub fn process_mint(program_id: &Pubkey, accounts: &[AccountInfo], amount: u64) -> ProgramResult {
    let (required_accounts, optional_accounts) = accounts.split_at(9);

    let [config, vault_info, lrt_mint, depositor, depositor_token_account, vault_token_account, depositor_lrt_token_account, vault_fee_token_account, token_program] =
        required_accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    load_config(program_id, config, false)?;
    load_vault(program_id, vault_info, false)?;
    load_token_mint(lrt_mint)?;
    load_signer(depositor, false)?;
    let mut vault_data = vault_info.data.borrow_mut();
    let vault = Vault::try_from_slice_mut(&mut vault_data)?;
    load_associated_token_account(
        depositor_token_account,
        depositor.key,
        &vault.supported_mint,
    )?;
    load_associated_token_account(vault_token_account, vault_info.key, &vault.supported_mint)?;
    load_associated_token_account(depositor_lrt_token_account, depositor.key, lrt_mint.key)?;
    load_associated_token_account(vault_fee_token_account, &vault.fee_wallet, lrt_mint.key)?;
    load_token_program(token_program)?;

    // If the vault has a mint_burn_admin, it must be the signer
    if vault.mint_burn_admin.ne(&Pubkey::default()) {
        if let Some(mint_signer) = optional_accounts.first() {
            load_signer(mint_signer, false)?;
            if mint_signer.key.ne(&vault.mint_burn_admin) {
                msg!("Mint signer does not match vault mint signer");
                return Err(ProgramError::InvalidAccountData);
            }
        } else {
            msg!("Mint signer is required for vault mint");
            return Err(ProgramError::InvalidAccountData);
        }
    }

    // check correct LRT mint
    if lrt_mint.key.ne(&vault.lrt_mint) {
        msg!("LRT mint account does not match vault LRT mint");
        return Err(ProgramError::InvalidAccountData);
    }

    let vault_fee = vault.calculate_deposit_fee(amount)?;
    let vault_deposit_amount = amount
        .checked_sub(vault_fee)
        .ok_or(ProgramError::ArithmeticOverflow)?;
    let vault_token_amount_after_deposit = vault
        .tokens_deposited
        .checked_add(vault_deposit_amount)
        .ok_or(ProgramError::ArithmeticOverflow)?;

    // The vault capacity shall not be exceeded after deposit
    if vault_token_amount_after_deposit > vault.capacity {
        msg!("Amount exceeds vault capacity");
        return Err(ProgramError::InvalidInstructionData);
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
                amount,
            )?,
            &[
                depositor_token_account.clone(),
                vault_token_account.clone(),
                depositor.clone(),
            ],
        )?;
    }

    let lrt_to_depositor = vault.calculate_lrt_mint_amount(vault_deposit_amount)?;
    let lrt_to_vault_fee_wallet = vault.calculate_lrt_mint_amount(vault_fee)?;

    vault.lrt_supply = vault
        .lrt_supply
        .checked_add(lrt_to_depositor)
        .and_then(|supply| supply.checked_add(lrt_to_vault_fee_wallet))
        .ok_or(ProgramError::ArithmeticOverflow)?;
    vault.tokens_deposited = vault_token_amount_after_deposit;

    let (_, vault_bump, mut vault_seeds) = Vault::find_program_address(program_id, &vault.base);
    vault_seeds.push(vec![vault_bump]);
    let seed_slices: Vec<&[u8]> = vault_seeds.iter().map(|seed| seed.as_slice()).collect();

    drop(vault_data); // no double borrow

    // mint to depositor and fee wallet
    {
        invoke_signed(
            &mint_to(
                &spl_token::id(),
                lrt_mint.key,
                depositor_lrt_token_account.key,
                vault_info.key,
                &[],
                lrt_to_depositor,
            )?,
            &[
                lrt_mint.clone(),
                depositor_lrt_token_account.clone(),
                vault_info.clone(),
            ],
            &[&seed_slices],
        )?;

        invoke_signed(
            &mint_to(
                &spl_token::id(),
                lrt_mint.key,
                vault_fee_token_account.key,
                vault_info.key,
                &[],
                lrt_to_vault_fee_wallet,
            )?,
            &[
                lrt_mint.clone(),
                vault_fee_token_account.clone(),
                vault_info.clone(),
            ],
            &[&seed_slices],
        )?;
    }

    Ok(())
}
