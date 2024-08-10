use solana_program::{
    account_info::AccountInfo, msg, program_error::ProgramError, program_pack::Pack,
    pubkey::Pubkey, system_program,
};
use spl_token::state::{Account, Mint};

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

pub fn load_uninitialized_pda<'a, 'info>(
    info: &'a AccountInfo<'info>,
    seeds: &[&[u8]],
    bump: u8,
    program_id: &Pubkey,
) -> Result<(), ProgramError> {
    let pda = Pubkey::find_program_address(seeds, program_id);

    if info.key.ne(&pda.0) {
        return Err(ProgramError::InvalidSeeds);
    }

    if bump.ne(&pda.1) {
        return Err(ProgramError::InvalidSeeds);
    }

    load_system_account(info, true)
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

pub fn load_token_account(
    info: &AccountInfo,
    owner: &Pubkey,
    mint: &Pubkey,
) -> Result<(), ProgramError> {
    if info.owner.ne(&spl_token::id()) {
        msg!("Account is not owned by the token program");
        return Err(ProgramError::InvalidAccountOwner);
    }

    if info.data_is_empty() {
        msg!("Account data is empty");
        return Err(ProgramError::InvalidAccountData);
    }

    let token_account = Account::unpack(&info.data.borrow())?;
    if token_account.owner.ne(owner) {
        msg!("Token account owner is invalid");
        return Err(ProgramError::InvalidAccountData);
    }
    if token_account.mint.ne(mint) {
        msg!("Token account mint is invalid");
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
