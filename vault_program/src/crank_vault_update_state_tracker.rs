use jito_account_traits::AccountDeserialize;
use jito_restaking_core::operator::Operator;
use jito_vault_core::{
    config::Config, vault::Vault, vault_operator_delegation::VaultOperatorDelegation,
    vault_update_state_tracker::VaultUpdateStateTracker,
};
use solana_program::{
    account_info::AccountInfo, clock::Clock, entrypoint::ProgramResult,
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
    let config_data = config.data.borrow();
    let config = Config::try_from_slice_unchecked(&config_data)?;
    Vault::load(program_id, vault, false)?;
    Operator::load(&config.restaking_program, operator, false)?;
    VaultOperatorDelegation::load(program_id, vault_operator_delegation, vault, operator, true)?;
    let mut vault_operator_delegation_data = vault_operator_delegation.data.borrow_mut();
    let vault_operator_delegation =
        VaultOperatorDelegation::try_from_slice_unchecked_mut(&mut vault_operator_delegation_data)?;
    let ncn_epoch = slot.checked_div(config.epoch_length).unwrap();
    VaultUpdateStateTracker::load(
        program_id,
        vault_update_state_tracker,
        vault,
        ncn_epoch,
        true,
    )?;
    let mut vault_update_state_tracker_data = vault_update_state_tracker.data.borrow_mut();
    let vault_update_state_tracker = VaultUpdateStateTracker::try_from_slice_unchecked_mut(
        &mut vault_update_state_tracker_data,
    )?;

    vault_update_state_tracker.check_and_update_index(vault_operator_delegation.index)?;
    vault_operator_delegation.update(slot, config.epoch_length);
    vault_update_state_tracker
        .delegation_state
        .accumulate(&vault_operator_delegation.delegation_state)?;

    Ok(())
}
