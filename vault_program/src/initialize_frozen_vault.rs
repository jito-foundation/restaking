use jito_bytemuck::AccountDeserialize;
use jito_vault_core::vault::Vault;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::initialize_vault::process_initialize_vault;

/// Processes the create instruction: [`crate::VaultInstruction::InitializeVault`]
pub fn process_initialize_frozen_vault(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    deposit_fee_bps: u16,
    withdrawal_fee_bps: u16,
    reward_fee_bps: u16,
    decimals: u8,
) -> ProgramResult {
    process_initialize_vault(
        program_id,
        accounts,
        deposit_fee_bps,
        withdrawal_fee_bps,
        reward_fee_bps,
        decimals,
    )?;

    let [_, vault, _, _, _, _, _, _] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    let mut vault_data = vault.try_borrow_mut_data()?;
    let vault = Vault::try_from_slice_unchecked_mut(&mut vault_data)?;

    vault.set_is_frozen();

    Ok(())
}
