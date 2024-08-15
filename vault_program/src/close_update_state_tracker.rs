use jito_account_traits::AccountDeserialize;
use jito_jsm_core::close_program_account;
use jito_jsm_core::loader::load_signer;
use jito_vault_core::config::Config;
use jito_vault_core::loader::{load_config, load_vault, load_vault_update_state_tracker};
use jito_vault_core::vault::Vault;
use jito_vault_core::vault_update_state_tracker::VaultUpdateStateTracker;
use jito_vault_sdk::error::VaultError;
use solana_program::account_info::AccountInfo;
use solana_program::clock::Clock;
use solana_program::entrypoint::ProgramResult;
use solana_program::msg;
use solana_program::program_error::ProgramError;
use solana_program::pubkey::Pubkey;
use solana_program::sysvar::Sysvar;

pub fn process_close_vault_update_state_tracker(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let [config, vault, vault_update_state_tracker_info, payer] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    let slot = Clock::get()?.slot;

    load_config(program_id, config, false)?;
    load_vault(program_id, vault, true)?;
    let config_data = config.data.borrow();
    let config = Config::try_from_slice(&config_data)?;
    let ncn_epoch = slot.checked_div(config.epoch_length).unwrap();
    load_vault_update_state_tracker(
        program_id,
        vault_update_state_tracker_info,
        vault,
        ncn_epoch,
        true,
    )?;
    load_signer(payer, true)?;

    let mut vault_data = vault.data.borrow_mut();
    let vault = Vault::try_from_slice_mut(&mut vault_data)?;

    let mut vault_update_state_tracker_data = vault_update_state_tracker_info.data.borrow_mut();
    let vault_update_state_tracker =
        VaultUpdateStateTracker::try_from_slice_mut(&mut vault_update_state_tracker_data)?;

    // The VaultUpdateStateTracker shall be up-to-date before closing
    if ncn_epoch != vault_update_state_tracker.ncn_epoch {
        msg!("VaultUpdateStateTracker is an invalid epoch");
        return Err(VaultError::VaultUpdateStateTrackerInvalid.into());
    }
    // The VaultUpdateStateTracker shall have updated every operator ticket before closing
    if vault_update_state_tracker.last_updated_index != vault.operator_count.saturating_sub(1) {
        msg!("VaultUpdateStateTracker is not fully updated");
        return Err(VaultError::VaultUpdateStateTrackerInvalid.into());
    }

    vault.amount_delegated = vault_update_state_tracker.amount_delegated;
    vault.last_full_state_update_slot = slot;

    drop(vault_update_state_tracker_data);
    close_program_account(program_id, vault_update_state_tracker_info, payer)?;

    Ok(())
}
