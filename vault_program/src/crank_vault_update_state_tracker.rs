use std::cmp::min;

use jito_bytemuck::AccountDeserialize;
use jito_restaking_core::operator::Operator;
use jito_vault_core::{
    config::Config, vault::Vault, vault_operator_delegation::VaultOperatorDelegation,
    vault_update_state_tracker::VaultUpdateStateTracker,
};
use jito_vault_sdk::{error::VaultError, instruction::WithdrawalAllocationMethod};
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
    Vault::load(program_id, vault_info, false)?;
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
    let ncn_epoch = slot
        .checked_div(config.epoch_length())
        .ok_or(VaultError::DivisionByZero)?;
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

    vault_update_state_tracker.check_and_update_index(vault_operator_delegation.index())?;

    match WithdrawalAllocationMethod::try_from(
        vault_update_state_tracker.withdrawal_allocation_method,
    ) {
        Ok(WithdrawalAllocationMethod::Greedy) => {
            if vault_update_state_tracker.additional_assets_need_unstaking() > 0 {
                let max_cooldown = min(
                    vault_operator_delegation.delegation_state.staked_amount(),
                    vault_update_state_tracker.additional_assets_need_unstaking(),
                );
                msg!(
                    "Force cooling down {} assets from operator {}",
                    max_cooldown,
                    vault_operator_delegation.operator
                );
                vault_operator_delegation
                    .delegation_state
                    .cooldown(max_cooldown)?;
                vault_update_state_tracker
                    .decrement_additional_assets_need_unstaking(max_cooldown)?;
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
