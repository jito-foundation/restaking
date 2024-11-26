use std::cmp::min;

use jito_bytemuck::AccountDeserialize;
use jito_restaking_core::operator::Operator;
use jito_vault_core::{
    config::Config, vault::Vault, vault_operator_delegation::VaultOperatorDelegation,
    vault_update_state_tracker::VaultUpdateStateTracker,
};
use jito_vault_sdk::instruction::WithdrawalAllocationMethod;
use solana_program::{
    account_info::AccountInfo, clock::Clock, entrypoint::ProgramResult, msg,
    program_error::ProgramError, pubkey::Pubkey, sysvar::Sysvar,
};

pub fn process_crank_vault_update_state_tracker(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let [config, vault_info, operator, vault_operator_delegation, vault_update_state_tracker] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    let slot = Clock::get()?.slot;

    Config::load(program_id, config, false)?;
    let config_data = config.data.borrow();
    let config = Config::try_from_slice_unchecked(&config_data)?;
    Vault::load(program_id, vault_info, true)?;
    let mut vault_data = vault_info.data.borrow_mut();
    let vault = Vault::try_from_slice_unchecked_mut(&mut vault_data)?;
    Operator::load(&config.restaking_program, operator, false)?;
    VaultOperatorDelegation::load(
        program_id,
        vault_operator_delegation,
        vault_info,
        operator,
        true,
    )?;
    let mut vault_operator_delegation_data = vault_operator_delegation.data.borrow_mut();
    let vault_operator_delegation =
        VaultOperatorDelegation::try_from_slice_unchecked_mut(&mut vault_operator_delegation_data)?;
    let ncn_epoch = config.get_epoch_from_slot(slot)?;

    let last_full_state_update_slot = vault.last_full_state_update_slot();
    let last_full_state_update_epoch = config.get_epoch_from_slot(last_full_state_update_slot)?;

    let operator_last_update_slot = vault_operator_delegation.last_update_slot();
    let operator_last_updated_epoch = config.get_epoch_from_slot(operator_last_update_slot)?;

    // If an operator has been updated in an epoch where the vault has not been fully updated,
    // it would have unstaked it's fair share of assets. So no further unstaking is needed, however,
    // the vault_operator_delegation should be updated to reflect the new state. In the case that
    // all operators have been updated and close_vault_update_state_tracker has not been called,
    // there should be zero additional_assets_need_unstaking, and it'd be okay to 'skip' withdrawing
    // the assets from the operator.
    let has_been_partially_updated = last_full_state_update_epoch < operator_last_updated_epoch;

    VaultUpdateStateTracker::load(
        program_id,
        vault_update_state_tracker,
        vault_info,
        ncn_epoch,
        true,
    )?;
    let mut vault_update_state_tracker_data = vault_update_state_tracker.data.borrow_mut();
    let vault_update_state_tracker = VaultUpdateStateTracker::try_from_slice_unchecked_mut(
        &mut vault_update_state_tracker_data,
    )?;

    vault.check_is_paused()?;

    vault_operator_delegation.check_is_already_updated(slot, config.epoch_length())?;
    vault_update_state_tracker
        .check_and_update_index(vault_operator_delegation.index(), vault.operator_count())?;

    match WithdrawalAllocationMethod::try_from(
        vault_update_state_tracker.withdrawal_allocation_method,
    ) {
        Ok(WithdrawalAllocationMethod::Greedy) => {
            // If an operator has been updated in a previous, partial update cycle,
            // they should no longer be the destination for any remaining `additional_assets_need_unstaking`
            // additionally, this keeps all of the `additional_assets_need_unstaking` at the same cooldown level
            // since the operator_delegation is updated for X epochs since the operator's last update
            if !has_been_partially_updated
                && vault.additional_assets_need_unstaking() > 0
                && vault_operator_delegation.delegation_state.staked_amount() > 0
            {
                let max_cooldown = min(
                    vault_operator_delegation.delegation_state.staked_amount(),
                    vault.additional_assets_need_unstaking(),
                );

                msg!(
                    "Force cooling down {} assets from operator {}",
                    max_cooldown,
                    vault_operator_delegation.operator
                );

                vault_operator_delegation
                    .delegation_state
                    .cooldown(max_cooldown)?;
                vault.decrement_additional_assets_need_unstaking(max_cooldown)?;
            }
        }
        Err(e) => {
            msg!(
                "Invalid withdrawal allocation method: {:?}",
                vault_update_state_tracker.withdrawal_allocation_method
            );
            return Err(e);
        }
    }

    vault_operator_delegation.update(slot, config.epoch_length())?;
    vault_update_state_tracker
        .delegation_state
        .accumulate(&vault_operator_delegation.delegation_state)?;

    Ok(())
}
