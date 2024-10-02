use jito_bytemuck::AccountDeserialize;
use jito_jsm_core::loader::{load_signer, load_token_account, load_token_mint};
use jito_restaking_core::ncn::Ncn;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program::invoke_signed,
    program_error::ProgramError, pubkey::Pubkey,
};

/// Processes the ncn delegate token account instruction: [`crate::RestakingInstruction::NcnDelegateTokenAccount`]
///
/// This instruction handles the delegation of tokens from a token account managed by an NCN to a delegate account.
/// The NCN delegate_admin might call this instruction to delegate authority over tokens managed by the NCN to another account.
///
/// # Arguments
/// * `program_id` - The public key of the program, used to ensure the correct program is being executed.
/// * `accounts` - A slice of `AccountInfo` representing the accounts required for this instruction.
/// * `amount` - The number of tokens to delegate to the delegate account.
///
/// # Returns
/// * `ProgramResult` - Returns `Ok(())` if the delegation is successful, otherwise returns an appropriate `ProgramError`.
pub fn process_ncn_delegate_token_account(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64,
) -> ProgramResult {
    let [ncn_info, delegate_admin, token_mint, token_account, delegate, token_program_info] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    Ncn::load(program_id, ncn_info, false)?;
    load_signer(delegate_admin, false)?;
    load_token_mint(token_mint)?;
    load_token_account(
        token_account,
        ncn_info.key,
        token_mint.key,
        token_program_info,
    )?;
    spl_token_2022::check_spl_token_program_account(token_program_info.key)?;

    if token_mint.owner.ne(token_account.owner) {
        return Err(ProgramError::InvalidAccountData);
    }
    // match (*token_mint.owner, *token_account.owner) {
    //     (spl_token::ID, spl_token::ID) => {
    //         load_token_program(token_program_info)?;
    //     }
    //     (spl_token_2022::ID, spl_token_2022::ID) => {
    //         load_token_2022_program(token_program_info)?;
    //     }
    //     _ => {
    //         msg!("token_mint and token_account owner does not match");
    //         return Err(ProgramError::InvalidAccountData);
    //     }
    // }

    let ncn_data = ncn_info.data.borrow();
    let ncn = Ncn::try_from_slice_unchecked(&ncn_data)?;

    // The Ncn delegate_admin shall be the signer of the transaction
    ncn.check_delegate_admin(delegate_admin.key)?;

    let mut ncn_seeds = Ncn::seeds(&ncn.base);
    ncn_seeds.push(vec![ncn.bump]);
    let ncn_seeds_slice = ncn_seeds
        .iter()
        .map(|seed| seed.as_slice())
        .collect::<Vec<&[u8]>>();

    drop(ncn_data);

    let ix = spl_token_2022::instruction::approve(
        token_program_info.key,
        token_account.key,
        delegate.key,
        ncn_info.key,
        &[],
        amount,
    )?;

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
