//! Loader functions for program accounts
use solana_program::{account_info::AccountInfo, msg, program_error::ProgramError, pubkey::Pubkey};
use solana_system_interface::program as system_program;
use spl_associated_token_account_interface::address::get_associated_token_address;

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
    if info
        .key
        .ne(&spl_associated_token_account_interface::program::id())
    {
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
/*pub fn load_token_program(info: &AccountInfo) -> Result<(), ProgramError> {
    if info.key.ne(&spl_token_interface::id()) {
        msg!("Account is not the spl token program");
        return Err(ProgramError::IncorrectProgramId);
    }

    Ok(())
}*/

/// Loads the account as the `spl_token_2022` program, returning an error if it is not.
///
/// # Arguments
/// * `info` - The account to load the token program from
///
/// # Returns
/// * `Result<(), ProgramError>` - The result of the operation
/*pub fn load_token_2022_program(info: &AccountInfo) -> Result<(), ProgramError> {
    if info.key.ne(&spl_token_2022_interface::id()) {
        msg!("Account is not the spl token 2022 program");
        return Err(ProgramError::IncorrectProgramId);
    }

    Ok(())
}*/

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
