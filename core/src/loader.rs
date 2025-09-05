//! Loader functions for program accounts
use solana_address::Address;
use solana_program::{account_info::AccountInfo, msg, program_error::ProgramError, pubkey::Pubkey};
use solana_system_interface::program as system_program;
use spl_associated_token_account::get_associated_token_address;
use spl_token_2022::extension::StateWithExtensions;

const SPL_TOKEN_PROGRAM_ID: Address = Address::new_from_array(spl_token::id().to_bytes());
const SPL_ASSOCIATED_TOKEN_ACCOUNT_PROGRAM_ID: Address =
    Address::new_from_array(spl_associated_token_account::id().to_bytes());

/// Loads the account as a signer, returning an error if it is not or if it is not writable while
/// expected to be.
///
/// # Arguments
/// * `info` - The account to load the signer from
/// * `expect_writable` - Whether the account should be writable
///
/// # Returns
/// * `Result<(), ProgramError>` - The result of the operation
pub fn load_signer(info: &AccountInfo, expect_writable: bool) -> Result<(), ProgramError> {
    if !info.is_signer {
        msg!("Account is not a signer");
        return Err(ProgramError::MissingRequiredSignature);
    }
    if expect_writable && !info.is_writable {
        msg!("Signer is not writable");
        return Err(ProgramError::InvalidAccountData);
    }

    Ok(())
}

/// Loads the account as a system program, returning an error if it is not.
///
/// # Arguments
/// * `info` - The account to load the system program from
///
/// # Returns
/// * `Result<(), ProgramError>` - The result of the operation
pub fn load_system_program(info: &AccountInfo) -> Result<(), ProgramError> {
    if info.key.ne(&system_program::id()) {
        msg!("Account is not the system program");
        return Err(ProgramError::IncorrectProgramId);
    }

    Ok(())
}

/// Loads the account as the `spl_token` program, returning an error if it is not.
///
/// # Arguments
/// * `info` - The account to load the token program from
///
/// # Returns
/// * `Result<(), ProgramError>` - The result of the operation
pub fn load_associated_token_account_program(info: &AccountInfo) -> Result<(), ProgramError> {
    if info.key.ne(&SPL_ASSOCIATED_TOKEN_ACCOUNT_PROGRAM_ID) {
        msg!("Account is not the spl associated token program");
        return Err(ProgramError::IncorrectProgramId);
    }

    Ok(())
}

/// Loads the account as the `spl_token` program, returning an error if it is not.
///
/// # Arguments
/// * `info` - The account to load the token program from
///
/// # Returns
/// * `Result<(), ProgramError>` - The result of the operation
pub fn load_token_program(info: &AccountInfo) -> Result<(), ProgramError> {
    if info.key.ne(&SPL_TOKEN_PROGRAM_ID) {
        msg!("Account is not the spl token program");
        return Err(ProgramError::IncorrectProgramId);
    }

    Ok(())
}

/// Loads the account as the `spl_token_2022` program, returning an error if it is not.
///
/// # Arguments
/// * `info` - The account to load the token program from
///
/// # Returns
/// * `Result<(), ProgramError>` - The result of the operation
pub fn load_token_2022_program(info: &AccountInfo) -> Result<(), ProgramError> {
    if info.key.ne(&SPL_TOKEN_PROGRAM_ID) {
        msg!("Account is not the spl token 2022 program");
        return Err(ProgramError::IncorrectProgramId);
    }

    Ok(())
}

/// Loads the account as a system account, returning an error if it is not or if it is not writable
/// while expected to be.
///
/// # Arguments
/// * `info` - The account to load the system account from
/// * `is_writable` - Whether the account should be writable
///
/// # Returns
/// * `Result<(), ProgramError>` - The result of the operation
pub fn load_system_account(info: &AccountInfo, is_writable: bool) -> Result<(), ProgramError> {
    if info.owner.ne(&system_program::id()) {
        msg!("Account is not owned by the system program");
        return Err(ProgramError::InvalidAccountOwner);
    }

    if !info.data_is_empty() {
        msg!("Account data is not empty");
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    if is_writable && !info.is_writable {
        msg!("Account is not writable");
        return Err(ProgramError::InvalidAccountData);
    }

    Ok(())
}

/// Loads the account as a token account, returning an error if it is not or if it is not writable
/// while expected to be.
///
/// # Arguments
/// * `token_account` - The account to load the token account from
/// * `owner` - The owner of the token account
/// * `mint` - The mint of the token account
///
/// # Returns
/// * `Result<(), ProgramError>` - The result of the operation
pub fn load_associated_token_account(
    token_account: &AccountInfo,
    owner: &Pubkey,
    mint: &Pubkey,
) -> Result<(), ProgramError> {
    if token_account.owner.ne(&SPL_TOKEN_PROGRAM_ID) {
        msg!("Account is not owned by the spl token program");
        return Err(ProgramError::InvalidAccountOwner);
    }

    if token_account.data_is_empty() {
        msg!("Account data is empty");
        return Err(ProgramError::InvalidAccountData);
    }

    let (associated_token_account, _) = solana_pubkey::Pubkey::find_program_address(
        &[
            &owner.to_bytes(),
            &SPL_TOKEN_PROGRAM_ID.to_bytes(),
            &mint.to_bytes(),
        ],
        &SPL_ASSOCIATED_TOKEN_ACCOUNT_PROGRAM_ID,
    );
    if token_account.key.ne(&associated_token_account) {
        msg!("Account is not the associated token account");
        return Err(ProgramError::InvalidAccountData);
    }

    Ok(())
}

/// Loads the account as a token account, ensuring it is correctly linked to the specified mint and is owned by the expected token program.
///
/// This function performs the following checks:
/// 1. Verifies that the `token_account` is associated with the expected SPL Token program.
/// 2. Checks that the `token_account` is not empty and contains valid data.
/// 3. Confirms that the `token_account` is linked to the specified `mint`, ensuring it is the correct token account for that mint.
///
/// # Arguments
/// * `token_account` - The account to load the token account from
/// * `owner` - The owner of the token account
/// * `mint` - The mint of the token account
///
/// # Returns
/// * `Result<(), ProgramError>` - The result of the operation
///
/// # Errors
/// This function will return an error in the following cases:
/// * `ProgramError::InvalidAccountOwner` - If the `token_account` is not owned by the expected SPL Token program.
/// * `ProgramError::InvalidAccountData` - If the `token_account` data is empty or if the mint associated with the `token_account` does not match the provided `mint`.
pub fn load_token_account(
    token_account: &AccountInfo,
    owner: &Pubkey,
    mint: &Pubkey,
    token_program: &AccountInfo,
) -> Result<(), ProgramError> {
    if token_program.key.ne(&SPL_TOKEN_PROGRAM_ID) {
        msg!("Account is not owned by the spl token program");
        return Err(ProgramError::IncorrectProgramId);
    }

    if token_account.owner.ne(&SPL_TOKEN_PROGRAM_ID) {
        msg!("Account is not owned by the token program");
        return Err(ProgramError::InvalidAccountOwner);
    }

    if token_account.data_is_empty() {
        msg!("Account data is empty");
        return Err(ProgramError::InvalidAccountData);
    }

    let data = token_account.data.borrow();
    let token_account = StateWithExtensions::<spl_token_2022::state::Account>::unpack(&data)
        .map_err(|_| ProgramError::InvalidAccountData)?;
    let token_base_mint_address = Address::new_from_array(token_account.base.mint.to_bytes());
    let token_base_owner_address = Address::new_from_array(token_account.base.owner.to_bytes());

    if token_base_owner_address.ne(owner) {
        msg!(
            "The token_account has an incorrect owner, expected {}, received {}",
            owner,
            token_account.base.owner
        );
        return Err(ProgramError::InvalidAccountData);
    }

    if token_base_mint_address.ne(mint) {
        msg!(
            "The token_account has an incorrect mint, expected {}, received {}",
            mint,
            token_account.base.mint
        );
        return Err(ProgramError::InvalidAccountData);
    }

    Ok(())
}

/// Loads the account as a token mint, returning an error if it is not.
///
/// # Arguments
/// * `info` - The account to load the token mint from
///
/// # Returns
/// * `Result<(), ProgramError>` - The result of the operation
pub fn load_token_mint(info: &AccountInfo) -> Result<(), ProgramError> {
    if info.owner.ne(&SPL_TOKEN_PROGRAM_ID) {
        msg!("Account is not owned by the spl token program");
        return Err(ProgramError::InvalidAccountOwner);
    }

    if info.data_is_empty() {
        msg!("Account data is empty");
        return Err(ProgramError::InvalidAccountData);
    }

    let _mint = StateWithExtensions::<spl_token_2022::state::Mint>::unpack(&info.data.borrow())
        .map_err(|_| ProgramError::InvalidAccountData)?;

    Ok(())
}
