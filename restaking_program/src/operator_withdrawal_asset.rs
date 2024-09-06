use jito_bytemuck::AccountDeserialize;
use jito_jsm_core::loader::{load_associated_token_account, load_signer, load_token_program};
use jito_restaking_core::operator::Operator;
use jito_restaking_sdk::error::RestakingError;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program::invoke_signed,
    program_error::ProgramError, program_pack::Pack, pubkey::Pubkey,
};

pub fn process_operator_withdrawal_asset(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64,
) -> ProgramResult {
    let [operator_info, operator_withdraw_admin, mint, operator_token_account, receiver_token_account, token_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    Operator::load(program_id, operator_info, false)?;
    load_signer(operator_withdraw_admin, false)?;
    load_associated_token_account(operator_token_account, operator_info.key, &mint.key)?;
    let operator_data = operator_info.data.borrow();
    let operator = Operator::try_from_slice_unchecked(&operator_data)?;
    load_associated_token_account(
        receiver_token_account,
        &operator.withdrawal_fee_wallet,
        &mint.key,
    )?;
    load_token_program(token_program)?;

    // The Operator withdraw admin shall be the signer of the transaction
    if operator.withdrawal_admin.ne(operator_withdraw_admin.key) {
        msg!("Invalid operator withdraw admin");
        return Err(RestakingError::OperatorWithdrawAdminInvalid.into());
    }

    let mut operator_seeds = Operator::seeds(&operator.base);
    operator_seeds.push(vec![operator.bump]);
    let ncn_seeds_slice = operator_seeds
        .iter()
        .map(|seed| seed.as_slice())
        .collect::<Vec<&[u8]>>();
    _withdraw_operator_asset(
        operator_info,
        mint,
        operator_token_account,
        receiver_token_account,
        token_program,
        &ncn_seeds_slice,
        amount,
    )?;

    Ok(())
}

fn _withdraw_operator_asset<'a, 'info>(
    operator: &'a AccountInfo<'info>,
    mint: &'a AccountInfo<'info>,
    operator_token_account: &'a AccountInfo<'info>,
    receiver_token_account: &'a AccountInfo<'info>,
    token_program: &'a AccountInfo<'info>,
    seeds: &[&[u8]],
    amount: u64,
) -> ProgramResult {
    let mint_account = spl_token_2022::state::Mint::unpack(&mint.data.borrow())?;

    invoke_signed(
        &spl_token_2022::instruction::transfer_checked(
            token_program.key,
            operator_token_account.key,
            mint.key,
            receiver_token_account.key,
            operator.key,
            &[],
            amount,
            mint_account.decimals,
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
