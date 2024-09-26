use jito_bytemuck::AccountDeserialize;
use jito_jsm_core::loader::{
    load_associated_token_account, load_signer, load_token_mint, load_token_program,
};
use jito_restaking_core::ncn::Ncn;
use jito_restaking_sdk::error::RestakingError;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program::invoke_signed,
    program_error::ProgramError, pubkey::Pubkey,
};
use spl_token::instruction::transfer;

pub fn process_ncn_withdraw_asset(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64,
) -> ProgramResult {
    let [ncn_info, ncn_token_account, receiver_token_account, withdraw_admin, token_mint, token_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    Ncn::load(program_id, ncn_info, false)?;
    load_token_mint(token_mint)?;
    load_associated_token_account(ncn_token_account, ncn_info.key, token_mint.key)?;
    let ncn_data = ncn_info.data.borrow();
    let ncn = Ncn::try_from_slice_unchecked(&ncn_data)?;
    load_associated_token_account(
        receiver_token_account,
        &ncn.withdraw_fee_wallet,
        token_mint.key,
    )?;
    load_signer(withdraw_admin, false)?;
    load_token_program(token_program)?;

    // The Ncn withdraw admin shall be the signer of the transaction
    if ncn.withdraw_admin.ne(withdraw_admin.key) {
        msg!("Invalid withdraw admin for NCN");
        return Err(RestakingError::NcnWithdrawAdminInvalid.into());
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
        &[seeds],
    )?;

    Ok(())
}
