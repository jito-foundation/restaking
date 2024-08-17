use jito_jsm_core::loader::{load_signer, load_token_account, load_token_mint, load_token_program};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program::invoke,
    program_error::ProgramError, pubkey::Pubkey,
};
use spl_token::instruction::approve;

pub fn process_delegate_token_account(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64,
) -> ProgramResult {
    let [admin, token_mint, token_account, owner, delegate, token_program] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    load_signer(admin, false)?;
    load_token_mint(token_mint)?;
    load_token_account(token_account)?;
    load_signer(owner, false)?;
    load_token_program(token_program)?;

    invoke(
        &approve(
            token_program.key,
            token_account.key,
            delegate.key,
            owner.key,
            &[],
            amount,
        )?,
        &[
            token_program.clone(),
            token_account.clone(),
            delegate.clone(),
            owner.clone(),
        ],
    )?;

    Ok(())
}
