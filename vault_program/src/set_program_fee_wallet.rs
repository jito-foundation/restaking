use jito_bytemuck::AccountDeserialize;
use jito_jsm_core::loader::load_signer;
use jito_vault_core::config::Config;
use jito_vault_sdk::error::VaultError;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
    pubkey::Pubkey,
};

/// Sets the program fee wallet for the vault program.
///
/// Specification:
/// - The fee wallet can only be changed by the config fee admin. The config fee admin must sign the transaction.
/// - The Config program_fee_wallet shall be updated to the new fee wallet.
pub fn process_set_program_fee_wallet(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let [config, config_fee_admin, new_fee_wallet] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    Config::load(program_id, config, true)?;
    let mut config_data = config.data.borrow_mut();
    let config = Config::try_from_slice_unchecked_mut(&mut config_data)?;
    load_signer(config_fee_admin, false)?;

    if config_fee_admin.key != &config.fee_admin {
        msg!("Config fee admin does not match");
        return Err(VaultError::VaultConfigFeeAdminInvalid.into());
    }

    config.program_fee_wallet = *new_fee_wallet.key;

    msg!("Config fee wallet updated to: {:?}", new_fee_wallet.key);
    Ok(())
}
