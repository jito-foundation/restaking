use jito_account_traits::AccountDeserialize;
use jito_jsm_core::loader::{load_signer, load_token_account, load_token_mint, load_token_program};
use jito_restaking_core::ncn::Ncn;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program::invoke_signed,
    program_error::ProgramError, pubkey::Pubkey,
};

/// Processes the ncn delegate token account instruction: [`crate::RestakingInstruction::NcnDelegateTokenAccount`]
///
/// Admin might call the instruction when the NCN is airdropped or transferred tokens
pub fn process_ncn_delegate_token_account(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64,
) -> ProgramResult {
    let [ncn_info, admin, token_mint, token_account, delegate, token_program_info] = accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    Ncn::load(program_id, ncn_info, false)?;
    load_signer(admin, false)?;
    load_token_mint(token_mint)?;

    let ncn_data = ncn_info.data.borrow();
    let ncn = Ncn::try_from_slice_unchecked(&ncn_data)?;

    // The Ncn admin shall be the signer of the transaction
    ncn.check_admin(admin)?;

    load_token_account(token_account, token_program_info)?;
    load_token_program(token_program_info)?;

    let mut ncn_seeds = Ncn::seeds(&ncn.base);
    ncn_seeds.push(vec![ncn.bump]);
    let ncn_seeds_slice = ncn_seeds
        .iter()
        .map(|seed| seed.as_slice())
        .collect::<Vec<&[u8]>>();

    drop(ncn_data);

    let ix = if token_program_info.key.eq(&spl_token::id()) {
        spl_token::instruction::approve(
            token_program_info.key,
            token_account.key,
            delegate.key,
            ncn_info.key,
            &[],
            amount,
        )?
    } else {
        spl_token_2022::instruction::approve(
            token_program_info.key,
            token_account.key,
            delegate.key,
            ncn_info.key,
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
            ncn_info.clone(),
        ],
        &[&ncn_seeds_slice],
    )?;

    Ok(())
}
