use jito_bytemuck::AccountDeserialize;
use jito_jsm_core::{close_program_account, get_epoch, loader::load_signer};
use jito_vault_core::{
    config::Config, vault::Vault, vault_update_state_tracker::VaultUpdateStateTracker,
};
use jito_vault_sdk::error::VaultError;
use solana_program::{
    account_info::AccountInfo, clock::Clock, entrypoint::ProgramResult, msg,
    program_error::ProgramError, pubkey::Pubkey, sysvar::Sysvar,
};

/// Close the VaultUpdateStateTracker
/// Can close previous epochs to get rent back, but it shall not update the current epoch
pub fn process_close_vault_update_state_tracker(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    ncn_epoch: u64,
) -> ProgramResult {
    let [config, vault_info, vault_update_state_tracker_info, payer] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    let slot = Clock::get()?.slot;

    Config::load(program_id, config, false)?;
    let config_data = config.data.borrow();
    let config = Config::try_from_slice_unchecked(&config_data)?;
    Vault::load(program_id, vault_info, true)?;
    let mut vault_data: std::cell::RefMut<'_, &mut [u8]> = vault_info.data.borrow_mut();
    let vault = Vault::try_from_slice_unchecked_mut(&mut vault_data)?;
    VaultUpdateStateTracker::load(
        program_id,
        vault_update_state_tracker_info,
        vault_info,
        ncn_epoch,
        true,
    )?;
    let vault_update_state_tracker_data = vault_update_state_tracker_info.data.borrow();
    let vault_update_state_tracker =
        VaultUpdateStateTracker::try_from_slice_unchecked(&vault_update_state_tracker_data)?;
    load_signer(payer, true)?;

    vault.check_is_paused()?;

    let epoch_length = config.epoch_length();
    let current_ncn_epoch = get_epoch(slot, epoch_length)?;
    let last_updated_epoch = get_epoch(vault.last_full_state_update_slot(), epoch_length)?;

    // The VaultUpdateStateTracker shall be up-to-date before closing
    if ncn_epoch != current_ncn_epoch {
        msg!(
            "Warning: VaultUpdateStateTracker is from an old epoch ({}), current epoch is {}",
            ncn_epoch,
            current_ncn_epoch
        );
    } else {
        // The VaultUpdateStateTracker shall have updated every operator ticket before closing
        if vault.operator_count() > 0
            && !vault_update_state_tracker.all_operators_updated(vault.operator_count())?
        {
            msg!("VaultUpdateStateTracker is not fully updated");
            return Err(VaultError::VaultUpdateStateNotFinishedUpdating.into());
        }

        if vault.additional_assets_need_unstaking() > 0 {
            msg!("This should not happen: additional assets need unstaking cannot be non-zero at the end of an update");
            return Err(VaultError::NonZeroAdditionalAssetsNeededForWithdrawalAtEndOfUpdate.into());
        }

        msg!("Finished updating VaultUpdateStateTracker");

        vault.delegation_state = vault_update_state_tracker.delegation_state;
        vault.set_last_full_state_update_slot(slot);

        // shift the VRT amounts down by one, accumulating in vrt_ready_to_claim_amount
        // at max, two epochs are needed to run through the cycle
        let epoch_diff = current_ncn_epoch
            .checked_sub(last_updated_epoch)
            .ok_or(VaultError::VaultUnderflow)?;
        for _ in 0..epoch_diff.min(2) {
            vault.increment_vrt_ready_to_claim_amount(vault.vrt_cooling_down_amount())?;
            vault.set_vrt_cooling_down_amount(vault.vrt_enqueued_for_cooldown_amount());
            vault.set_vrt_enqueued_for_cooldown_amount(0);
        }
    }

    msg!("Closing VaultUpdateStateTracker");
    drop(vault_update_state_tracker_data);
    close_program_account(program_id, vault_update_state_tracker_info, payer)?;

    Ok(())
}
