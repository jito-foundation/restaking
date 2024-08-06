pub const TEST: u64 = 2;
// use borsh::{BorshDeserialize, BorshSerialize};
// use shank::ShankInstruction;
// use solana_program::{
//     instruction::{AccountMeta, Instruction},
//     pubkey::Pubkey,
//     system_program,
// };

// #[rustfmt::skip]
// #[derive(Debug, BorshSerialize, BorshDeserialize, ShankInstruction)]
// pub enum VaultInstruction {
//     /// Initializes global configuration
//     #[account(0, writable, name = "config")]
//     #[account(1, writable, signer, name = "admin")]
//     #[account(2, name = "restaking_program")]
//     #[account(3, name = "system_program")]
//     InitializeConfig,
// }
