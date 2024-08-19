use jito_account_traits::AccountDeserialize;
use jito_jsm_core::loader::load_signer;
use jito_vault_core::{config::Config, vault::Vault};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
    pubkey::Pubkey,
};

/// Processes the set admin instruction: [`crate::VaultInstruction::SetAdmin`]
pub fn process_set_admin(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let [config, vault, old_admin, new_admin] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    Config::load(program_id, config, false)?;
    Vault::load(program_id, vault, false)?;
    let mut vault_data = vault.data.borrow_mut();
    let vault = Vault::try_from_slice_unchecked_mut(&mut vault_data)?;
    load_signer(old_admin, false)?;
    load_signer(new_admin, false)?;

    vault.check_admin(old_admin.key)?;
    vault.admin = *new_admin.key;
    vault.update_secondary_admin(old_admin.key, new_admin.key);

    Ok(())
}
