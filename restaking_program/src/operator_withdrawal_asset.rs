use jito_restaking_core::operator::{Operator, SanitizedOperator};
use jito_restaking_sanitization::{
    assert_with_msg, signer::SanitizedSignerAccount, token_account::SanitizedTokenAccount,
    token_program::SanitizedTokenProgram,
};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program::invoke_signed,
    program_error::ProgramError,
    pubkey::Pubkey,
};
use spl_token::instruction::transfer;

pub fn process_operator_withdrawal_asset(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    token_mint: Pubkey,
    amount: u64,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    let operator =
        SanitizedOperator::sanitize(program_id, next_account_info(accounts_iter)?, false)?;
    let admin = SanitizedSignerAccount::sanitize(next_account_info(accounts_iter)?, false)?;
    let operator_token_account = SanitizedTokenAccount::sanitize(
        next_account_info(accounts_iter)?,
        &token_mint,
        operator.account().key,
    )?;
    // token program handles verification of receiver_token_account
    let receiver_token_account = next_account_info(accounts_iter)?;
    let _token_program = SanitizedTokenProgram::sanitize(next_account_info(accounts_iter)?)?;

    assert_with_msg(
        operator.operator().admin() == *admin.account().key,
        ProgramError::InvalidAccountData,
        "Admin does not match operator",
    )?;

    assert_with_msg(
        operator_token_account.token_account().amount >= amount,
        ProgramError::InsufficientFunds,
        "Not enough funds in AVS token account",
    )?;

    let mut operator_seeds = Operator::seeds(&operator.operator().base());
    operator_seeds.push(vec![operator.operator().bump()]);
    let operator_seeds_slice = operator_seeds
        .iter()
        .map(|seed| seed.as_slice())
        .collect::<Vec<&[u8]>>();

    invoke_signed(
        &transfer(
            &spl_token::id(),
            operator_token_account.account().key,
            receiver_token_account.key,
            operator.account().key,
            &[],
            amount,
        )?,
        &[
            operator_token_account.account().clone(),
            receiver_token_account.clone(),
            operator.account().clone(),
        ],
        &[operator_seeds_slice.as_slice()],
    )?;

    Ok(())
}
