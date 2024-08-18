//! Loader functions for the vault program.
use jito_vault_sdk::inline_mpl_token_metadata::{self, pda::find_metadata_account};
use solana_program::{account_info::AccountInfo, msg, program_error::ProgramError, pubkey::Pubkey};

/// Loads the account as a mpl metadata program, returning an error if it is not.
///
/// # Arguments
/// * `info` - The account to load the mpl metadata program from
///
/// # Returns
/// * `Result<(), ProgramError>` - The result of the operation
pub fn load_mpl_metadata_program(info: &AccountInfo) -> Result<(), ProgramError> {
    if info.key.ne(&inline_mpl_token_metadata::id()) {
        msg!(
            "Expected mpl metadata program {}, received {}",
            inline_mpl_token_metadata::id(),
            info.key
        );
        return Err(ProgramError::IncorrectProgramId);
    }

    Ok(())
}

/// Loads the account as a mpl metadata account, returning an error if it is not.
///
/// # Arguments
/// * `info` - The account to load the mpl metadata program from
///
/// # Returns
/// * `Result<(), ProgramError>` - The result of the operation
pub fn load_mpl_metadata(info: &AccountInfo, vrt_mint: &Pubkey) -> Result<(), ProgramError> {
    let (metadata_account_pubkey, _) = find_metadata_account(vrt_mint);

    if metadata_account_pubkey.ne(info.key) {
        Err(ProgramError::InvalidAccountData)
    } else {
        Ok(())
    }
}
