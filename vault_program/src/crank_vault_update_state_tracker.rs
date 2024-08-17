use jito_account_traits::AccountDeserialize;
use jito_restaking_core::operator::Operator;
use jito_vault_core::{
    config::Config, vault::Vault, vault_operator_delegation::VaultOperatorDelegation,
    vault_update_state_tracker::VaultUpdateStateTracker,
};
use jito_vault_sdk::error::VaultError;
use solana_program::{
    account_info::AccountInfo, clock::Clock, entrypoint::ProgramResult, msg,
    program_error::ProgramError, pubkey::Pubkey, sysvar::Sysvar,
};

pub fn process_crank_vault_update_state_tracker(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let [config, vault, operator, vault_operator_delegation, vault_update_state_tracker] = accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    let slot = Clock::get()?.slot;

    Config::load(program_id, config, false)?;
    Vault::load(program_id, vault, false)?;
    let config_data = config.data.borrow();
    let config = Config::try_from_slice_unchecked(&config_data)?;
    Operator::load(&config.restaking_program, operator, false)?;
    VaultOperatorDelegation::load(program_id, vault_operator_delegation, vault, operator, true)?;
    let ncn_epoch = slot.checked_div(config.epoch_length).unwrap();
    VaultUpdateStateTracker::load(
        program_id,
        vault_update_state_tracker,
        vault,
        ncn_epoch,
        true,
    )?;

    let mut vault_operator_delegation_data = vault_operator_delegation.data.borrow_mut();
    let vault_operator_delegation =
        VaultOperatorDelegation::try_from_slice_unchecked_mut(&mut vault_operator_delegation_data)?;

    let mut vault_update_state_tracker_data = vault_update_state_tracker.data.borrow_mut();
    let vault_update_state_tracker = VaultUpdateStateTracker::try_from_slice_unchecked_mut(
        &mut vault_update_state_tracker_data,
    )?;

    if vault_update_state_tracker.last_updated_index == u64::MAX {
        if vault_operator_delegation.index != 0 {
            msg!("VaultUpdateStateTracker incorrect index");
            return Err(VaultError::VaultUpdateIncorrectIndex.into());
        }
    } else if vault_operator_delegation.index
        != vault_update_state_tracker
            .last_updated_index
            .checked_add(1)
            .unwrap()
    {
        msg!("VaultUpdateStateTracker incorrect index");
        return Err(VaultError::VaultUpdateIncorrectIndex.into());
    }

    // There's a possibility the VaultOperatorDelegation was partially updated in the past, so to avoid
    // over-updating it, we look at the last_update_slot in the VaultOperatorDelegation to calculate the
    // last update epoch instead of looking at the epoch in the VaultUpdateStateTracker
    let last_update_epoch = vault_operator_delegation
        .last_update_slot
        .checked_div(config.epoch_length)
        .unwrap();
    let current_epoch = slot.checked_div(config.epoch_length).unwrap();

    let epoch_diff = current_epoch.checked_sub(last_update_epoch).unwrap();
    match epoch_diff {
        0 => {
            // this shouldn't be possible
        }
        1 => {
            vault_operator_delegation.update(slot);
        }
        _ => {
            // max 2 transitions needeed
            vault_operator_delegation.update(slot);
            vault_operator_delegation.update(slot);
        }
    }

    vault_update_state_tracker
        .delegation_state
        .accumulate(&vault_operator_delegation.delegation_state)?;
    vault_update_state_tracker.last_updated_index = vault_operator_delegation.index;

    Ok(())
}
