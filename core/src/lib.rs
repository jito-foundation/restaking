use solana_program::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction,
};

pub mod loader;
pub mod slot_toggle;

/// Creates a new account or initializes an existing account
/// # Arguments
/// * `payer` - The account that will pay for the lamports
/// * `new_account` - The account to create or initialize
/// * `system_program` - The system program account
/// * `program_owner` - The owner of the program
/// * `rent` - The rent sysvar
/// * `space` - The space to allocate
/// * `seeds` - The seeds to use for the PDA
/// # Returns
/// * `ProgramResult` - The result of the operation
#[inline(always)]
pub fn create_account<'a, 'info>(
    payer: &'a AccountInfo<'info>,
    new_account: &'a AccountInfo<'info>,
    system_program: &'a AccountInfo<'info>,
    program_owner: &Pubkey,
    rent: &Rent,
    space: u64,
    seeds: &[Vec<u8>],
) -> ProgramResult {
    let current_lamports = **new_account.try_borrow_lamports()?;
    if current_lamports == 0 {
        // If there are no lamports in the new account, we create it with the create_account instruction
        invoke_signed(
            &system_instruction::create_account(
                payer.key,
                new_account.key,
                rent.minimum_balance(space as usize),
                space,
                program_owner,
            ),
            &[payer.clone(), new_account.clone(), system_program.clone()],
            &[seeds
                .iter()
                .map(|seed| seed.as_slice())
                .collect::<Vec<&[u8]>>()
                .as_slice()],
        )
    } else {
        // someone can transfer lamports to accounts before they're initialized
        // in that case, creating the account won't work.
        // in order to get around it, you need to find the account with enough lamports to be rent exempt,
        // then allocate the required space and set the owner to the current program
        let required_lamports = rent
            .minimum_balance(space as usize)
            .max(1)
            .saturating_sub(current_lamports);
        if required_lamports > 0 {
            invoke(
                &system_instruction::transfer(payer.key, new_account.key, required_lamports),
                &[payer.clone(), new_account.clone(), system_program.clone()],
            )?;
        }
        // Allocate space.
        invoke_signed(
            &system_instruction::allocate(new_account.key, space),
            &[new_account.clone(), system_program.clone()],
            &[seeds
                .iter()
                .map(|seed| seed.as_slice())
                .collect::<Vec<&[u8]>>()
                .as_slice()],
        )?;
        // Assign to the specified program
        invoke_signed(
            &system_instruction::assign(new_account.key, program_owner),
            &[new_account.clone(), system_program.clone()],
            &[seeds
                .iter()
                .map(|seed| seed.as_slice())
                .collect::<Vec<&[u8]>>()
                .as_slice()],
        )
    }
}

/// Closes the program account
pub fn close_program_account<'a>(
    program_id: &Pubkey,
    account_to_close: &AccountInfo<'a>,
    destination_account: &AccountInfo<'a>,
) -> ProgramResult {
    // Check if the account is owned by the program
    if account_to_close.owner != program_id {
        return Err(ProgramError::IllegalOwner);
    }

    **destination_account.lamports.borrow_mut() = destination_account
        .lamports()
        .checked_add(account_to_close.lamports())
        .ok_or(ProgramError::ArithmeticOverflow)?;
    **account_to_close.lamports.borrow_mut() = 0;

    account_to_close.assign(&solana_program::system_program::id());
    let mut account_data = account_to_close.data.borrow_mut();
    let data_len = account_data.len();
    solana_program::program_memory::sol_memset(*account_data, 0, data_len);

    Ok(())
}

pub fn realloc<'a, 'info>(
    account: &'a AccountInfo<'info>,
    new_size: usize,
    payer: &'a AccountInfo<'info>,
    rent: &Rent,
) -> ProgramResult {
    let new_minimum_balance = rent.minimum_balance(new_size);

    let lamports_diff = new_minimum_balance.saturating_sub(account.lamports());
    invoke(
        &system_instruction::transfer(payer.key, account.key, lamports_diff),
        &[payer.clone(), account.clone()],
    )?;
    account.realloc(new_size, false)?;
    Ok(())
}
