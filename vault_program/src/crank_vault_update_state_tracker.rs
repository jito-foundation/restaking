use jito_account_traits::AccountDeserialize;
use jito_restaking_core::loader::load_operator;
use jito_vault_core::loader::{load_vault_operator_ticket, load_vault_update_state_tracker};
use jito_vault_core::vault_operator_ticket::VaultOperatorTicket;
use jito_vault_core::vault_update_state_tracker::VaultUpdateStateTracker;
use jito_vault_core::{
    config::Config,
    loader::{load_config, load_vault},
};
use jito_vault_sdk::error::VaultError;
use solana_program::clock::Clock;
use solana_program::sysvar::Sysvar;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
    pubkey::Pubkey,
};

pub fn process_crank_vault_update_state_tracker(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let [config, vault, operator, vault_operator_ticket, vault_delegations_update_ticket] =
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
    load_vault_operator_ticket(program_id, vault_operator_ticket, vault, operator, true)?;
    let ncn_epoch = slot.checked_div(config.epoch_length).unwrap();
    load_vault_update_state_tracker(
        program_id,
        vault_delegations_update_ticket,
        vault,
        ncn_epoch,
        true,
    )?;

    let mut vault_operator_ticket_data = vault_operator_ticket.data.borrow_mut();
    let vault_operator_ticket =
        VaultOperatorTicket::try_from_slice_mut(&mut vault_operator_ticket_data)?;

    let mut vault_delegations_update_ticket_data =
        vault_delegations_update_ticket.data.borrow_mut();
    let vault_delegations_update_ticket =
        VaultUpdateStateTracker::try_from_slice_mut(&mut vault_delegations_update_ticket_data)?;

    // 0 is a special case
    if vault_operator_ticket.index == 0 {
        if vault_delegations_update_ticket.last_updated_index != u64::MAX {
            msg!("VaultUpdateStateTracker incorrect index");
            return Err(VaultError::VaultUpdateIncorrectIndex.into());
        }
    } else if vault_operator_ticket.index
        != vault_delegations_update_ticket
            .last_updated_index
            .checked_add(1)
            .unwrap()
    {
        msg!("VaultUpdateStateTracker incorrect index");
        return Err(VaultError::VaultUpdateIncorrectIndex.into());
    }

    // There's a possibility the VaultOperatorTicket was partially updated in the past, so to avoid
    // over-updating it, we look at the last_update_slot in the VaultOperatorTicket to calculate the
    // last update epoch instead of looking at the epoch in the VaultUpdateStateTracker
    let last_update_epoch = vault_operator_ticket
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
            vault_operator_ticket.update(slot);
        }
        _ => {
            // max 2 transitions needeed
            vault_operator_ticket.update(slot);
            vault_operator_ticket.update(slot);
        }
    }

    vault_delegations_update_ticket.amount_delegated = vault_delegations_update_ticket
        .amount_delegated
        .checked_add(vault_operator_ticket.total_security()?)
        .ok_or(VaultError::VaultAssetsReturnedOverflow)?;
    vault_delegations_update_ticket.last_updated_index = vault_operator_ticket.index;

    Ok(())
}
