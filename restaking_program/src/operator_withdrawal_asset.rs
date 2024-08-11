use jito_account_traits::AccountDeserialize;
use jito_jsm_core::loader::{load_associated_token_account, load_signer};
use jito_restaking_core::{loader::load_operator, operator::Operator};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program::invoke_signed,
    program_error::ProgramError, pubkey::Pubkey,
};
use spl_token::instruction::transfer;

pub fn process_operator_withdrawal_asset(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    token_mint: Pubkey,
    amount: u64,
) -> ProgramResult {
    let [operator_info, operator_withdraw_admin, operator_token_account, receiver_token_account, _token_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    load_operator(program_id, operator_info, false)?;
    load_signer(operator_withdraw_admin, false)?;
    load_associated_token_account(operator_token_account, operator_info.key, &token_mint)?;
    let operator_data = operator_info.data.borrow();
    let operator = Operator::try_from_slice(&operator_data)?;
    load_associated_token_account(
        receiver_token_account,
        &operator.withdraw_fee_wallet,
        &token_mint,
    )?;

    // The Operator withdraw admin shall be the signer of the transaction
    if operator.withdraw_admin.ne(operator_withdraw_admin.key) {
        msg!("Invalid operator withdraw admin");
        return Err(ProgramError::InvalidAccountData);
    }

    let mut operator_seeds = Operator::seeds(&operator.base);
    operator_seeds.push(vec![operator.bump]);
    let ncn_seeds_slice = operator_seeds
        .iter()
        .map(|seed| seed.as_slice())
        .collect::<Vec<&[u8]>>();
    _withdraw_operator_asset(
        operator_info,
        operator_token_account,
        receiver_token_account,
        &ncn_seeds_slice,
        amount,
    )?;

    Ok(())
}

fn _withdraw_operator_asset<'a, 'info>(
    operator: &'a AccountInfo<'info>,
    operator_token_account: &'a AccountInfo<'info>,
    receiver_token_account: &'a AccountInfo<'info>,
    seeds: &[&[u8]],
    amount: u64,
) -> ProgramResult {
    invoke_signed(
        &transfer(
            &spl_token::id(),
            operator_token_account.key,
            receiver_token_account.key,
            operator.key,
            &[],
            amount,
        )?,
        &[
            operator_token_account.clone(),
            receiver_token_account.clone(),
            operator.clone(),
        ],
        &[seeds],
    )?;

    Ok(())
}
