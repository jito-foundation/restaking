use solana_program::{
    account_info::AccountInfo, msg, program_error::ProgramError, program_pack::Pack,
    pubkey::Pubkey, system_program,
};
use spl_associated_token_account::get_associated_token_address;
use spl_token::state::Mint;

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

pub fn load_system_program(info: &AccountInfo) -> Result<(), ProgramError> {
    if info.key.ne(&system_program::id()) {
        msg!("Account is not the system program");
        return Err(ProgramError::IncorrectProgramId);
    }

    Ok(())
}

pub fn load_token_program(info: &AccountInfo) -> Result<(), ProgramError> {
    if info.key.ne(&spl_token::id()) {
        msg!("Account is not the token program");
        return Err(ProgramError::IncorrectProgramId);
    }

    Ok(())
}

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

pub fn load_token_mint(info: &AccountInfo) -> Result<(), ProgramError> {
    if info.owner.ne(&spl_token::id()) {
        msg!("Account is not owned by the token program");
        return Err(ProgramError::InvalidAccountOwner);
    }

    if info.data_is_empty() {
        msg!("Account data is empty");
        return Err(ProgramError::InvalidAccountData);
    }

    let _mint = Mint::unpack(&info.data.borrow())?;

    Ok(())
}
