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
    let SanitizedAccounts {
        operator,
        admin,
        operator_token_account,
        receiver_token_account,
    } = SanitizedAccounts::sanitize(program_id, accounts, &token_mint)?;

    operator.operator().check_admin(admin.account().key)?;

    assert_with_msg(
        operator_token_account.token_account().amount >= amount,
        ProgramError::InsufficientFunds,
        "Not enough funds in NCN token account",
    )?;

    _withdraw_operator_asset(
        &operator,
        &operator_token_account,
        receiver_token_account,
        amount,
    )?;

    Ok(())
}

fn _withdraw_operator_asset<'a, 'info>(
    operator: &SanitizedOperator<'a, 'info>,
    operator_token_account: &SanitizedTokenAccount<'a, 'info>,
    receiver_token_account: &AccountInfo<'info>,
    amount: u64,
) -> ProgramResult {
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

struct SanitizedAccounts<'a, 'info> {
    operator: SanitizedOperator<'a, 'info>,
    admin: SanitizedSignerAccount<'a, 'info>,
    operator_token_account: SanitizedTokenAccount<'a, 'info>,
    receiver_token_account: &'a AccountInfo<'info>,
}

impl<'a, 'info> SanitizedAccounts<'a, 'info> {
    fn sanitize(
        program_id: &Pubkey,
        accounts: &'a [AccountInfo<'info>],
        token_mint: &Pubkey,
    ) -> Result<SanitizedAccounts<'a, 'info>, ProgramError> {
        let accounts_iter = &mut accounts.iter();

        let operator =
            SanitizedOperator::sanitize(program_id, next_account_info(accounts_iter)?, false)?;
        let admin = SanitizedSignerAccount::sanitize(next_account_info(accounts_iter)?, false)?;
        let operator_token_account = SanitizedTokenAccount::sanitize(
            next_account_info(accounts_iter)?,
            token_mint,
            operator.account().key,
        )?;
        // token program handles verification of receiver_token_account
        let receiver_token_account = next_account_info(accounts_iter)?;
        let _token_program = SanitizedTokenProgram::sanitize(next_account_info(accounts_iter)?)?;

        Ok(SanitizedAccounts {
            operator,
            admin,
            operator_token_account,
            receiver_token_account,
        })
    }
}
