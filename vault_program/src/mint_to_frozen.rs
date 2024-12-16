use jito_bytemuck::AccountDeserialize;
use jito_vault_core::vault::Vault;
use jito_vault_sdk::error::VaultError;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program::invoke_signed,
    program_error::ProgramError, pubkey::Pubkey,
};
use spl_token::instruction::freeze_account;

use crate::mint_to::process_mint;

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
pub fn process_mint_frozen(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount_in: u64,
    min_amount_out: u64,
) -> ProgramResult {
    process_mint(program_id, accounts, amount_in, min_amount_out, true)?;

    let (required_accounts, _) = accounts.split_at(9);

    let [_, vault_info, vrt_mint, _, _, _, depositor_vrt_token_account, vault_fee_token_account, _] =
        required_accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    Vault::load(program_id, vault_info, true)?;
    let vault_data = vault_info.data.borrow();
    let vault = Vault::try_from_slice_unchecked(&vault_data)?;

    if !vault.is_frozen() {
        msg!("Vault is not frozen");
        return Err(VaultError::VaultNotFrozen.into());
    }

    let signing_seeds = vault.signing_seeds();
    let seed_slices: Vec<&[u8]> = signing_seeds.iter().map(|seed| seed.as_slice()).collect();

    // freeze the token account
    {
        invoke_signed(
            &freeze_account(
                &spl_token::id(),
                depositor_vrt_token_account.key,
                vrt_mint.key,
                vault_info.key,
                &[],
            )?,
            &[
                vrt_mint.clone(),
                depositor_vrt_token_account.clone(),
                vault_info.clone(),
            ],
            &[&seed_slices],
        )?;

        invoke_signed(
            &freeze_account(
                &spl_token::id(),
                vault_fee_token_account.key,
                vrt_mint.key,
                vault_info.key,
                &[],
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
