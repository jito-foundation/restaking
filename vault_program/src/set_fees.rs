use jito_bytemuck::AccountDeserialize;
use jito_jsm_core::loader::load_signer;
use jito_vault_core::{config::Config, vault::Vault};
use solana_program::{
    account_info::AccountInfo, clock::Clock, entrypoint::ProgramResult, msg,
    program_error::ProgramError, pubkey::Pubkey, sysvar::Sysvar,
};

/// Sets the deposit, withdrawal, and reward fees for the vault.
///
/// Specification:
/// - The fee can only be changed by the vault fee admin. The vault fee admin must sign the transaction.
/// - The fees can only be changed at most once per epoch.
/// - The fees can be changed the epoch after one full epoch has passed since the last fee change.
/// - The Vault last_fee_change_slot shall be updated to the current slot only if any fees were updated.
/// - The transaction shall fail if no fees are provided to update.
/// - The transaction shall fail if any of the fees exceed 10_000 bps.
pub fn process_set_fees(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    deposit_fee_bps: Option<u16>,
    withdrawal_fee_bps: Option<u16>,
    reward_fee_bps: Option<u16>,
) -> ProgramResult {
    let [config, vault, vault_fee_admin] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    Config::load(program_id, config, false)?;
    let config_data = config.data.borrow();
    let config = Config::try_from_slice_unchecked(&config_data)?;
    Vault::load(program_id, vault, true)?;
    let mut vault_data = vault.data.borrow_mut();
    let vault = Vault::try_from_slice_unchecked_mut(&mut vault_data)?;
    load_signer(vault_fee_admin, false)?;

    vault.check_fee_admin(vault_fee_admin.key)?;
    vault.check_can_modify_fees(Clock::get()?.slot, config.epoch_length())?;

    if deposit_fee_bps.is_none() && withdrawal_fee_bps.is_none() && reward_fee_bps.is_none() {
        msg!("No fees provided for update");
        return Err(ProgramError::InvalidInstructionData);
    }

    if let Some(deposit_fee_bps) = deposit_fee_bps {
        vault.set_deposit_fee_bps(
            deposit_fee_bps,
            config.deposit_withdrawal_fee_cap_bps(),
            config.fee_bump_bps(),
            config.fee_rate_of_change_bps(),
        )?;
    }

    if let Some(withdrawal_fee_bps) = withdrawal_fee_bps {
        vault.set_withdrawal_fee_bps(
            withdrawal_fee_bps,
            config.deposit_withdrawal_fee_cap_bps(),
            config.fee_bump_bps(),
            config.fee_rate_of_change_bps(),
        )?;
    }

    if let Some(reward_fee_bps) = reward_fee_bps {
        vault.set_reward_fee_bps(reward_fee_bps)?;
    }

    vault.set_last_fee_change_slot(Clock::get()?.slot);

    Ok(())
}
