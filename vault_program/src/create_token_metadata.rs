use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, pubkey::Pubkey};

pub fn process_create_token_metadata(
    _program_id: &Pubkey,
    _accounts: &[AccountInfo],
    _name: String,
    _symbol: String,
    _uri: String,
) -> ProgramResult {
    Ok(())
}
