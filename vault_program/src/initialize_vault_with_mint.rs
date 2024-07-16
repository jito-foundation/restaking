use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, pubkey::Pubkey};

pub const fn process_initialize_vault_with_mint(
    _program_id: &Pubkey,
    _accounts: &[AccountInfo],
) -> ProgramResult {
    Ok(())
}
