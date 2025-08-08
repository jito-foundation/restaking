use jito_bytemuck::AccountDeserialize;
use jito_jsm_core::loader::load_signer;
use jito_vault_core::config::Config;
use jito_vault_sdk::instruction::ConfigAdminRole;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
    pubkey::Pubkey,
};

/// Sets a secondary admin for a specific role in the configuration.
///
/// This function updates the configuration to set a new account as a secondary administrator
/// with a specific role.
///
/// - Fee Admin
pub fn process_set_config_secondary_admin(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    role: ConfigAdminRole,
) -> ProgramResult {
    let [config, admin, new_admin] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    Config::load(program_id, config, true)?;
    let mut config_data = config.data.borrow_mut();
    let config = Config::try_from_slice_unchecked_mut(&mut config_data)?;
    load_signer(admin, false)?;

    config.check_admin(admin.key)?;

    match role {
        ConfigAdminRole::FeeAdmin => {
            config.fee_admin = *new_admin.key;
            msg!("Fee admin set to {:?}", new_admin.key);
        }
    }

    Ok(())
}
