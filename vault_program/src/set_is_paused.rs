use jito_bytemuck::AccountDeserialize;
use jito_jsm_core::loader::load_signer;
use jito_vault_core::{config::Config, vault::Vault};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
    pubkey::Pubkey,
};

/// Processes the set `is_paused` instruction: [`crate::VaultInstruction::SetIsPaused`]
pub fn process_set_is_paused(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    is_paused: bool,
) -> ProgramResult {
    let [config, vault, admin] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    Config::load(program_id, config, false)?;
    Vault::load(program_id, vault, true)?;
    let mut vault_data = vault.data.borrow_mut();
    let vault = Vault::try_from_slice_unchecked_mut(&mut vault_data)?;
    load_signer(admin, false)?;

    vault.check_admin(admin.key)?;

    vault.set_is_paused(is_paused);

    Ok(())
}
