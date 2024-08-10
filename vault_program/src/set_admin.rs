use jito_account_traits::AccountDeserialize;
use jito_jsm_core::loader::load_signer;
use jito_vault_core::{
    loader::{load_config, load_vault},
    vault::Vault,
};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
    pubkey::Pubkey,
};

/// Processes the set admin instruction: [`crate::VaultInstruction::SetAdmin`]
pub fn process_set_admin(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let [config, vault, old_admin, new_admin] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    load_config(program_id, config, false)?;
    load_vault(program_id, vault, false)?;
    load_signer(old_admin, false)?;
    load_signer(new_admin, false)?;

    let mut vault_data = vault.data.borrow_mut();
    let vault = Vault::try_from_slice_mut(&mut vault_data)?;
    if vault.admin.ne(old_admin.key) {
        msg!("Invalid admin for vault");
        return Err(ProgramError::InvalidAccountData);
    }
    vault.admin = *new_admin.key;

    Ok(())
}
