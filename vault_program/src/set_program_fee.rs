use jito_bytemuck::{types::PodU16, AccountDeserialize};
use jito_jsm_core::loader::load_signer;
use jito_vault_core::{config::Config, vault::Vault, MAX_FEE_BPS};
use jito_vault_sdk::error::VaultError;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
    pubkey::Pubkey,
};

/// Sets the program fee for the vault program.
///
/// Specification:
/// - The fee can only be changed by the config admin. The config admin must sign the transaction.
/// - The transaction shall fail if the new fee exceeds MAX_FEE_BPS.
/// - The fee change must be acceptable according to the Config::check_fee_change_ok method.
/// - The Config program_fee_bps shall be updated to the new fee.
pub fn process_set_program_fee(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    new_fee_bps: u16,
) -> ProgramResult {
    let [config, config_admin] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    Config::load(program_id, config, true)?;
    let mut config_data = config.data.borrow_mut();
    let config = Config::try_from_slice_unchecked_mut(&mut config_data)?;
    load_signer(config_admin, false)?;

    if config_admin.key != &config.admin {
        msg!("Config admin does not match");
        return Err(VaultError::VaultConfigAdminInvalid.into());
    }

    if new_fee_bps > MAX_FEE_BPS {
        msg!("New fee exceeds maximum allowed fee");
        return Err(ProgramError::InvalidInstructionData);
    }

    Vault::check_fee_change_ok(
        config.program_fee_bps(),
        new_fee_bps,
        config.deposit_withdrawal_fee_cap_bps(),
        config.fee_bump_bps(),
        config.fee_rate_of_change_bps(),
    )?;

    config.program_fee_bps = PodU16::from(new_fee_bps);

    Ok(())
}
