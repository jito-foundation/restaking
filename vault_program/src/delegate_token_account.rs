use jito_bytemuck::AccountDeserialize;
use jito_jsm_core::loader::{
    load_signer, load_token_2022_program, load_token_account, load_token_mint, load_token_program,
};
use jito_vault_core::{config::Config, vault::Vault};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program::invoke_signed,
    program_error::ProgramError, pubkey::Pubkey,
};

/// Processes the delegate token account instruction: [`crate::VaultInstruction::DelegateTokenAccount`]
///
/// This instruction handles the delegation of a token account to a specified delegate.
/// The vault admin might call this instruction when the vault receives tokens through an airdrop
/// or a transfer, and the admin needs to delegate authority over these tokens to another account.
///
/// # Arguments
/// * `program_id` - The public key of the program to ensure the correct program is being executed.
/// * `accounts` - A slice of `AccountInfo` representing the accounts required for this instruction.
/// * `amount` - The number of tokens to delegate to the delegate account.
///
/// # Returns
/// * `ProgramResult` - Returns `Ok(())` if the delegation is successful, otherwise returns an appropriate `ProgramError`.
pub fn process_delegate_token_account(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64,
) -> ProgramResult {
    let [config, vault_info, admin, token_mint, token_account, delegate, token_program_info] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    Config::load(program_id, config, false)?;
    Vault::load(program_id, vault_info, false)?;
    load_signer(admin, false)?;
    load_token_mint(token_mint)?;
    load_token_account(token_account, token_mint.key, token_program_info)?;

    match (*token_mint.owner, *token_account.owner) {
        (spl_token::ID, spl_token::ID) => {
            load_token_program(token_program_info)?;
        }
        (spl_token_2022::ID, spl_token_2022::ID) => {
            load_token_2022_program(token_program_info)?;
        }
        _ => {
            msg!("token_mint and token_account owner does not match");
            return Err(ProgramError::InvalidAccountData);
        }
    }

    let mut vault_data = vault_info.data.borrow_mut();
    let vault = Vault::try_from_slice_unchecked_mut(&mut vault_data)?;
    if vault.supported_mint.eq(token_mint.key) {
        msg!("Invalid Token mint");
        return Err(ProgramError::InvalidAccountData);
    }

    // The Vault admin shall be the signer of the transaction
    vault.check_admin(admin.key)?;

    let (vault_pubkey, vault_bump, mut vault_seeds) =
        Vault::find_program_address(program_id, &vault.base);
    vault_seeds.push(vec![vault_bump]);

    drop(vault_data);

    let ix = if token_program_info.key.eq(&spl_token::id()) {
        spl_token::instruction::approve(
            token_program_info.key,
            token_account.key,
            delegate.key,
            &vault_pubkey,
            &[],
            amount,
        )?
    } else {
        spl_token_2022::instruction::approve(
            token_program_info.key,
            token_account.key,
            delegate.key,
            &vault_pubkey,
            &[],
            amount,
        )?
    };

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
