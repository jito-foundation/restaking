use jito_restaking_core::avs::{Avs, SanitizedAvs};
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

pub fn process_avs_withdrawal_asset(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    token_mint: Pubkey,
    amount: u64,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    let avs = SanitizedAvs::sanitize(program_id, next_account_info(accounts_iter)?, false)?;
    let admin = SanitizedSignerAccount::sanitize(next_account_info(accounts_iter)?, false)?;
    let avs_token_account = SanitizedTokenAccount::sanitize(
        next_account_info(accounts_iter)?,
        &token_mint,
        avs.account().key,
    )?;
    // token program handles verification of receiver_token_account
    let receiver_token_account = next_account_info(accounts_iter)?;
    let _token_program = SanitizedTokenProgram::sanitize(next_account_info(accounts_iter)?)?;

    assert_with_msg(
        avs.avs().admin() == *admin.account().key,
        ProgramError::InvalidAccountData,
        "Admin does not match AVS",
    )?;

    assert_with_msg(
        avs_token_account.token_account().amount >= amount,
        ProgramError::InsufficientFunds,
        "Not enough funds in AVS token account",
    )?;

    let mut avs_seeds = Avs::seeds(&avs.avs().base());
    avs_seeds.push(vec![avs.avs().bump()]);

    let avs_seeds_slice = avs_seeds
        .iter()
        .map(|seed| seed.as_slice())
        .collect::<Vec<&[u8]>>();

    invoke_signed(
        &transfer(
            &spl_token::id(),
            avs_token_account.account().key,
            receiver_token_account.key,
            avs.account().key,
            &[],
            amount,
        )?,
        &[
            avs_token_account.account().clone(),
            receiver_token_account.clone(),
            avs.account().clone(),
        ],
        &[avs_seeds_slice.as_slice()],
    )?;

    Ok(())
}
