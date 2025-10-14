use solana_program::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    pubkey::Pubkey,
    rent::Rent,
};

pub fn create_account<'a, 'info>(
    _payer: &'a AccountInfo<'info>,
    new_account: &'a AccountInfo<'info>,
    _system_program: &'a AccountInfo<'info>,
    _program_owner: &Pubkey,
    _rent: &Rent,
    space: u64,
    _seeds: &[Vec<u8>],
) -> ProgramResult {

    new_account.realloc(space as usize, false).unwrap();
    Ok(())
}
