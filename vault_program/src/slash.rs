use solana_program::{
    account_info::AccountInfo, entrypoint_deprecated::ProgramResult, pubkey::Pubkey,
};

pub const fn process_slash(
    _program_id: &Pubkey,
    _accounts: &[AccountInfo],
    _amount: u64,
) -> ProgramResult {
    // #[account(0, name = "config")]
    // #[account(1, name = "vault")]
    // #[account(2, name = "vault_slasher_list")]
    // #[account(3, writable, name = "vault_operator_list")]
    // #[account(4, name = "avs")]
    // #[account(5, name = "operator")]
    // #[account(6, name = "slasher")]
    Ok(())
}
