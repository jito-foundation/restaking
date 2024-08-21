use jito_bytemuck::AccountDeserialize;
use jito_jsm_core::loader::{
    load_signer, load_token_2022_program, load_token_account, load_token_mint, load_token_program,
};
use jito_restaking_core::operator::Operator;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program::invoke_signed,
    program_error::ProgramError, pubkey::Pubkey,
};

/// Processes the operator delegate token account instruction: [`crate::RestakingInstruction::OperatorDelegateTokenAccount`]
///
/// Admin might call the instruction when the Operator is airdropped or transferred tokens
pub fn process_operator_delegate_token_account(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64,
) -> ProgramResult {
    let [operator_info, admin, token_mint, token_account, delegate, token_program_info] = accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    Operator::load(program_id, operator_info, false)?;
    load_signer(admin, false)?;
    load_token_mint(token_mint)?;

    let operator_data = operator_info.data.borrow();
    let operator = Operator::try_from_slice_unchecked(&operator_data)?;

    // The Operator admin shall be the signer of the transaction
    operator.check_admin(admin)?;

    load_token_account(token_account, token_program_info)?;

    match (*token_mint.owner, *token_account.owner) {
        (spl_token::ID, spl_token::ID) => {
            load_token_program(token_program_info)?;
        }
        (spl_token_2022::ID, spl_token_2022::ID) => {
            load_token_2022_program(token_program_info)?;
        }
        _ => {
            msg!("token_mint and token_account owner does not match");
            return Err(ProgramError::InvalidAccountData);
        }
    }

    let mut operator_seeds = Operator::seeds(&operator.base);
    operator_seeds.push(vec![operator.bump]);
    let operator_seeds_slice = operator_seeds
        .iter()
        .map(|seed| seed.as_slice())
        .collect::<Vec<&[u8]>>();

    drop(operator_data);

    let ix = if token_program_info.key.eq(&spl_token::id()) {
        spl_token::instruction::approve(
            token_program_info.key,
            token_account.key,
            delegate.key,
            operator_info.key,
            &[],
            amount,
        )?
    } else {
        spl_token_2022::instruction::approve(
            token_program_info.key,
            token_account.key,
            delegate.key,
            operator_info.key,
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
            operator_info.clone(),
        ],
        &[&operator_seeds_slice],
    )?;

    Ok(())
}
