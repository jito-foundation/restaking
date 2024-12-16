use jito_bytemuck::AccountDeserialize;
use jito_jsm_core::loader::{load_signer, load_token_account, load_token_mint, load_token_program};
use jito_vault_core::{config::Config, vault::Vault};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program::invoke_signed,
    program_error::ProgramError, pubkey::Pubkey,
};

/// Processes the delegate token account instruction: [`crate::VaultInstruction::DelegateTokenAccount`]
///
/// This instruction handles the delegation of a token account to a specified delegate.
/// The vault delegate_asset_admin might call this instruction when the vault receives tokens through an airdrop
/// or a transfer, and the admin needs to delegate authority over these tokens to another account.
///
/// # Arguments
/// * `program_id` - The public key of the program to ensure the correct program is being executed.
/// * `accounts` - A slice of `AccountInfo` representing the accounts required for this instruction.
///
/// # Returns
/// * `ProgramResult` - Returns `Ok(())` if the delegation is successful, otherwise returns an appropriate `ProgramError`.
pub fn process_delegate_token_account(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let [config, vault_info, delegate_asset_admin, token_mint, token_account, delegate, token_program_info] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    Config::load(program_id, config, false)?;
    Vault::load(program_id, vault_info, false)?;
    load_signer(delegate_asset_admin, false)?;
    load_token_mint(token_mint)?;
    load_token_account(
        token_account,
        vault_info.key,
        token_mint.key,
        token_program_info,
    )?;
    // Only the original spl token program is allowed
    load_token_program(token_program_info)?;

    // The owner of token mint and token account must match
    if token_mint.owner.ne(token_account.owner) {
        return Err(ProgramError::InvalidAccountData);
    }

    let vault_data = vault_info.data.borrow();
    let vault = Vault::try_from_slice_unchecked(&vault_data)?;
    if vault.supported_mint.eq(token_mint.key) {
        msg!("Cannot delegate away the supported mint for a vault!");
        return Err(ProgramError::InvalidAccountData);
    }

    // The Vault delegate_asset_admin shall be the signer of the transaction
    vault.check_delegate_asset_admin(delegate_asset_admin.key)?;

    let vault_seeds = vault.signing_seeds();

    drop(vault_data);

    let ix = spl_token_2022::instruction::approve(
        token_program_info.key,
        token_account.key,
        delegate.key,
        vault_info.key,
        &[],
        u64::MAX,
    )?;

    invoke_signed(
        &ix,
        &[
            token_program_info.clone(),
            token_account.clone(),
            delegate.clone(),
            vault_info.clone(),
        ],
        &[vault_seeds
            .iter()
            .map(|seed| seed.as_slice())
            .collect::<Vec<&[u8]>>()
            .as_slice()],
    )?;

    Ok(())
}
