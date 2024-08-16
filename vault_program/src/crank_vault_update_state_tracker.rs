use jito_account_traits::AccountDeserialize;
use jito_restaking_core::loader::load_operator;
use jito_vault_core::{
    config::Config,
    loader::{
        load_config, load_vault, load_vault_operator_delegation, load_vault_update_state_tracker,
    },
    vault_operator_delegation::VaultOperatorDelegation,
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
    let [config, vault, operator, vault_operator_delegation, vault_delegations_update_ticket] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    let slot = Clock::get()?.slot;

    load_config(program_id, config, false)?;
    load_vault(program_id, vault, false)?;
    let config_data = config.data.borrow();
    let config = Config::try_from_slice(&config_data)?;
    load_operator(&config.restaking_program, operator, false)?;
    load_vault_operator_delegation(program_id, vault_operator_delegation, vault, operator, true)?;
    let ncn_epoch = slot.checked_div(config.epoch_length).unwrap();
    load_vault_update_state_tracker(
        program_id,
        vault_delegations_update_ticket,
        vault,
        ncn_epoch,
        true,
    )?;

    let mut vault_operator_delegation_data = vault_operator_delegation.data.borrow_mut();
    let vault_operator_delegation =
        VaultOperatorDelegation::try_from_slice_mut(&mut vault_operator_delegation_data)?;

    let mut vault_delegations_update_ticket_data =
        vault_delegations_update_ticket.data.borrow_mut();
    let vault_delegations_update_ticket =
        VaultUpdateStateTracker::try_from_slice_mut(&mut vault_delegations_update_ticket_data)?;

    // 0 is a special case
    if vault_operator_delegation.index == 0 {
        if vault_delegations_update_ticket.last_updated_index != u64::MAX {
            msg!("VaultUpdateStateTracker incorrect index");
            return Err(VaultError::VaultUpdateIncorrectIndex.into());
        }
    } else if vault_operator_delegation.index
        != vault_delegations_update_ticket
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

    vault_delegations_update_ticket.amount_delegated = vault_delegations_update_ticket
        .amount_delegated
        .checked_add(vault_operator_delegation.total_security()?)
        .ok_or(VaultError::VaultDelegationUpdateOverflow)?;
    vault_delegations_update_ticket.amount_enqueued_for_cooldown = vault_delegations_update_ticket
        .amount_enqueued_for_cooldown
        .checked_add(vault_operator_delegation.enqueued_for_cooldown_amount)
        .and_then(|v| v.checked_add(vault_operator_delegation.enqueued_for_withdraw_amount))
        .ok_or(VaultError::VaultDelegationUpdateOverflow)?;
    vault_delegations_update_ticket.amount_cooling_down = vault_delegations_update_ticket
        .amount_cooling_down
        .checked_add(vault_operator_delegation.cooling_down_amount)
        .and_then(|v| v.checked_add(vault_operator_delegation.cooling_down_for_withdraw_amount))
        .ok_or(VaultError::VaultDelegationUpdateOverflow)?;

    vault_delegations_update_ticket.last_updated_index = vault_operator_delegation.index;

    Ok(())
}
