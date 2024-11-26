use jito_bytemuck::AccountDeserialize;
use jito_jsm_core::loader::{load_signer, load_token_account, load_token_mint, load_token_program};
use jito_restaking_core::operator::Operator;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program::invoke_signed,
    program_error::ProgramError, pubkey::Pubkey,
};

/// Processes the operator delegate token account instruction: [`crate::RestakingInstruction::OperatorDelegateTokenAccount`]
///
/// This instruction handles the delegation of tokens from a token account managed by an Operator to a delegate account.
/// The delegate_admin might call this instruction when the Operator receives tokens through an airdrop or transfer,
/// and the delegate_admin needs to delegate authority over these tokens to another account.
///
/// # Arguments
/// * `program_id` - The public key of the program, used to ensure the correct program is being executed.
/// * `accounts` - A slice of `AccountInfo` representing the accounts required for this instruction.
///
/// # Returns
/// * `ProgramResult` - Returns `Ok(())` if the delegation is successful, otherwise returns an appropriate `ProgramError`.
pub fn process_operator_delegate_token_account(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let [operator_info, delegate_admin, token_mint, token_account, delegate, token_program_info] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    Operator::load(program_id, operator_info, false)?;
    load_signer(delegate_admin, false)?;
    load_token_mint(token_mint)?;
    load_token_account(
        token_account,
        operator_info.key,
        token_mint.key,
        token_program_info,
    )?;
    // Only the original spl token program is allowed
    load_token_program(token_program_info)?;

    // We support SPL Token and SPL Token 2022 standards
    // The owner of token mint and token account must match
    if token_mint.owner.ne(token_account.owner) {
        return Err(ProgramError::InvalidAccountData);
    }

    let operator_data = operator_info.data.borrow();
    let operator = Operator::try_from_slice_unchecked(&operator_data)?;

    // The Operator delegate_admin shall be the signer of the transaction
    operator.check_delegate_admin(delegate_admin.key)?;

    let mut operator_seeds = Operator::seeds(&operator.base);
    operator_seeds.push(vec![operator.bump]);
    let operator_seeds_slice = operator_seeds
        .iter()
        .map(|seed| seed.as_slice())
        .collect::<Vec<&[u8]>>();

    drop(operator_data);

    let ix = spl_token_2022::instruction::approve(
        token_program_info.key,
        token_account.key,
        delegate.key,
        operator_info.key,
        &[],
        u64::MAX,
    )?;

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
