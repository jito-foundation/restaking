//! Loader functions for program accounts
use solana_program::{
    account_info::AccountInfo, msg, program_error::ProgramError, pubkey::Pubkey, system_program,
};
use spl_associated_token_account::get_associated_token_address;
use spl_token_2022::extension::StateWithExtensionsOwned;

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

/// Loads the account as the token program, returning an error if it is not.
///
/// # Arguments
/// * `info` - The account to load the token program from
///
/// # Returns
/// * `Result<(), ProgramError>` - The result of the operation
pub fn load_token_program(info: &AccountInfo) -> Result<(), ProgramError> {
    if info.key.eq(&spl_token::id()) || info.key.eq(&spl_token_2022::id()) {
    } else {
        msg!("Account is not the token program");
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
    if token_account.owner.ne(&spl_token::id()) {
        msg!("Account is not owned by the token program");
        return Err(ProgramError::InvalidAccountOwner);
    }

    if token_account.data_is_empty() {
        msg!("Account data is empty");
        return Err(ProgramError::InvalidAccountData);
    }

    let associated_token_account = get_associated_token_address(owner, mint);
    if token_account.key.ne(&associated_token_account) {
        msg!("Account is not the associated token account");
        return Err(ProgramError::InvalidAccountData);
    }

    Ok(())
}

/// Loads the account as a token account, returning an error if it is not.
///
/// # Arguments
/// * `token_account` - The account to load the token account from
///
/// # Returns
/// * `Result<(), ProgramError>` - The result of the operation
pub fn load_token_account(
    token_account: &AccountInfo,
    token_program_info: &AccountInfo,
) -> Result<(), ProgramError> {
    if token_account.owner.ne(token_program_info.key) {
        msg!("Account is not owned by the token program");
        return Err(ProgramError::InvalidAccountOwner);
    }

    if token_account.data_is_empty() {
        msg!("Account data is empty");
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
    if info.owner.eq(&spl_token::id()) || info.owner.eq(&spl_token_2022::id()) {
    } else {
        msg!("Account is not owned by the token program");
        return Err(ProgramError::InvalidAccountOwner);
    }

    if info.data_is_empty() {
        msg!("Account data is empty");
        return Err(ProgramError::InvalidAccountData);
    }

    let data = info.data.borrow().to_vec();
    let _mint = StateWithExtensionsOwned::<spl_token_2022::state::Mint>::unpack(data);

    Ok(())
}
