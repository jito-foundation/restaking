use jito_account_traits::AccountDeserialize;
use jito_jsm_core::loader::{load_associated_token_account, load_signer, load_token_program};
use jito_restaking_core::{loader::load_ncn, ncn::Ncn};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program::invoke_signed,
    program_error::ProgramError, pubkey::Pubkey,
};
use spl_token::instruction::transfer;

pub fn process_ncn_withdraw_asset(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    token_mint: Pubkey,
    amount: u64,
) -> ProgramResult {
    let [ncn_info, ncn_token_account, receiver_token_account, withdraw_admin, token_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    load_ncn(program_id, ncn_info, false)?;
    load_associated_token_account(ncn_token_account, ncn_info.key, &token_mint)?;
    let ncn_data = ncn_info.data.borrow();
    let ncn = Ncn::try_from_slice(&ncn_data)?;
    load_associated_token_account(
        receiver_token_account,
        &ncn.withdraw_fee_wallet,
        &token_mint,
    )?;
    load_signer(withdraw_admin, false)?;
    load_token_program(token_program)?;

    if ncn.withdraw_admin.ne(withdraw_admin.key) {
        msg!("Invalid withdraw admin for NCN");
        return Err(ProgramError::InvalidAccountData);
    }

    let mut ncn_seeds = Ncn::seeds(&ncn.base);
    ncn_seeds.push(vec![ncn.bump]);
    let ncn_seeds_slice = ncn_seeds
        .iter()
        .map(|seed| seed.as_slice())
        .collect::<Vec<&[u8]>>();

    _withdraw_ncn_asset(
        ncn_info,
        ncn_token_account,
        receiver_token_account,
        &ncn_seeds_slice,
        amount,
    )?;

    Ok(())
}

fn _withdraw_ncn_asset<'a, 'info>(
    ncn: &'a AccountInfo<'info>,
    ncn_token_account: &'a AccountInfo<'info>,
    receiver_token_account: &'a AccountInfo<'info>,
    seeds: &[&[u8]],
    amount: u64,
) -> ProgramResult {
    invoke_signed(
        &transfer(
            &spl_token::id(),
            ncn_token_account.key,
            receiver_token_account.key,
            ncn.key,
            &[],
            amount,
        )?,
        &[
            ncn_token_account.clone(),
            receiver_token_account.clone(),
            ncn.clone(),
        ],
        &[&seeds],
    )?;

    Ok(())
}
