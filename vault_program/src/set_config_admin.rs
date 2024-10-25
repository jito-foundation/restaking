use jito_bytemuck::AccountDeserialize;
use jito_jsm_core::loader::load_signer;
use jito_vault_core::config::Config;
use jito_vault_sdk::error::VaultError;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
    pubkey::Pubkey,
};

/// Processes the set config admin instruction: [`crate::VaultInstruction::SetConfigAdmin`]
pub fn process_set_config_admin(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let [config, old_admin, new_admin] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    Config::load(program_id, config, true)?;
    let mut config_data = config.data.borrow_mut();
    let config = Config::try_from_slice_unchecked_mut(&mut config_data)?;
    load_signer(old_admin, false)?;
    load_signer(new_admin, false)?;

    if config.admin != *old_admin.key {
        return Err(VaultError::ConfigAdminInvalid.into());
    }
    config.set_admin(*new_admin.key);

    Ok(())
}
