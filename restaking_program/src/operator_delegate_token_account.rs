use jito_account_traits::AccountDeserialize;
use jito_jsm_core::loader::{load_signer, load_token_account, load_token_mint, load_token_program};
use jito_restaking_core::operator::Operator;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program::invoke_signed,
    program_error::ProgramError, pubkey::Pubkey,
};

pub fn process_operator_delegate_token_account(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64,
) -> ProgramResult {
    let [operator_info, admin, token_mint, token_account, delegate, token_program_info] = accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    Operator::load(program_id, operator_info, false)?;
    load_signer(admin, false)?;
    load_token_mint(token_mint)?;

    let operator_data = operator_info.data.borrow();
    let operator = Operator::try_from_slice_unchecked(&operator_data)?;

    // The Operator admin shall be the signer of the transaction
    operator.check_admin(admin)?;

    load_token_account(token_account, token_program_info)?;
    load_token_program(token_program_info)?;

    let mut operator_seeds = Operator::seeds(&operator.base);
    operator_seeds.push(vec![operator.bump]);
    let operator_seeds_slice = operator_seeds
        .iter()
        .map(|seed| seed.as_slice())
        .collect::<Vec<&[u8]>>();

    drop(operator_data);

    let ix = if token_program_info.key.eq(&spl_token::id()) {
        spl_token::instruction::approve(
            token_program_info.key,
            token_account.key,
            delegate.key,
            operator_info.key,
            &[],
            amount,
        )?
    } else {
        spl_token_2022::instruction::approve(
            token_program_info.key,
            token_account.key,
            delegate.key,
            operator_info.key,
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
            operator_info.clone(),
        ],
        &[&operator_seeds_slice],
    )?;

    Ok(())
}
