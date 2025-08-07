use crate::certora::utils::*;
use crate::{
    cooldown_vault_ncn_slasher_ticket::process_cooldown_vault_ncn_slasher_ticket,
    cooldown_vault_ncn_ticket::process_cooldown_vault_ncn_ticket,
    warmup_vault_ncn_slasher_ticket::process_warmup_vault_ncn_slasher_ticket,
    warmup_vault_ncn_ticket::process_warmup_vault_ncn_ticket,
};
use cvlr::prelude::*;
use cvlr_solana::{cvlr_deserialize_nondet_accounts, cvlr_nondet_pubkey};
use jito_bytemuck::AccountDeserialize;
use jito_jsm_core::slot_toggle::SlotToggleState;
use solana_program::{account_info::AccountInfo, clock::Clock, sysvar::Sysvar};

use jito_jsm_core::certora::utils::cvlr_advance_clock_slot;

// Rules to check that warmup and cooldown fucntions change the Slot Toggle state correctly

#[rule]
pub fn rule_integrity_process_cooldown_vault_ncn_ticket() {
    let program_id = cvlr_nondet_pubkey();

    let acc_infos: [AccountInfo; 16] = cvlr_deserialize_nondet_accounts();
    let used_acc_infos = &acc_infos[..5];
    let config_info = &used_acc_infos[0];
    let vault_ncn_ticket_info = &used_acc_infos[3];

    // initialize the clock
    cvlr_advance_clock_slot();

    let config = get_vault_config!(config_info);
    process_cooldown_vault_ncn_ticket(&program_id, &used_acc_infos).unwrap();

    // after a non-deterministic amount of time passes
    cvlr_advance_clock_slot();

    let vault_ncn_ticket = get_vault_ncn_ticket!(vault_ncn_ticket_info);
    let state = vault_ncn_ticket
        .state
        .state(Clock::get().unwrap().slot, config.epoch_length())
        .unwrap();
    cvlr_assert!(state == SlotToggleState::Cooldown || state == SlotToggleState::Inactive);
}

#[rule]
pub fn rule_integrity_process_cooldown_vault_ncn_slasher_ticket() {
    let program_id = cvlr_nondet_pubkey();

    let acc_infos: [AccountInfo; 16] = cvlr_deserialize_nondet_accounts();
    let used_acc_infos = &acc_infos[..6];
    let config_info = &used_acc_infos[0];
    let vault_ncn_slasher_ticket_info = &used_acc_infos[4];

    // initialize the clock
    cvlr_advance_clock_slot();

    let config = get_vault_config!(config_info);
    process_cooldown_vault_ncn_slasher_ticket(&program_id, &used_acc_infos).unwrap();

    // after a non-deterministic amount of time passes
    cvlr_advance_clock_slot();

    let vault_ncn_slasher_ticket = get_vault_ncn_slasher_ticket!(vault_ncn_slasher_ticket_info);
    let state = vault_ncn_slasher_ticket
        .state
        .state(Clock::get().unwrap().slot, config.epoch_length())
        .unwrap();
    cvlr_assert!(state == SlotToggleState::Cooldown || state == SlotToggleState::Inactive);
}

#[rule]
pub fn rule_integrity_process_warmup_vault_ncn_ticket() {
    let program_id = cvlr_nondet_pubkey();

    let acc_infos: [AccountInfo; 16] = cvlr_deserialize_nondet_accounts();
    let used_acc_infos = &acc_infos[..5];
    let config_info = &used_acc_infos[0];
    let vault_ncn_ticket_info = &used_acc_infos[3];

    // initialize the clock
    cvlr_advance_clock_slot();

    let config = get_vault_config!(config_info);
    let vault_ncn_ticket = get_vault_ncn_ticket!(vault_ncn_ticket_info);
    let state = vault_ncn_ticket.state;
    let current_slot = Clock::get().unwrap().slot;
    cvlr_assume!(state.slot_added() < current_slot);
    cvlr_assume!(state.slot_removed() < current_slot);

    process_warmup_vault_ncn_ticket(&program_id, &used_acc_infos).unwrap();

    // after a non-deterministic amount of time passes
    cvlr_advance_clock_slot();

    let vault_ncn_ticket = get_vault_ncn_ticket!(vault_ncn_ticket_info);
    let state = vault_ncn_ticket
        .state
        .state(Clock::get().unwrap().slot, config.epoch_length())
        .unwrap();
    cvlr_assert!(state == SlotToggleState::WarmUp || state == SlotToggleState::Active);
}

#[rule]
pub fn rule_integrity_process_warmup_vault_ncn_slasher_ticket() {
    let program_id = cvlr_nondet_pubkey();

    let acc_infos: [AccountInfo; 16] = cvlr_deserialize_nondet_accounts();
    let used_acc_infos = &acc_infos[..6];
    let config_info = &used_acc_infos[0];
    let vault_ncn_slasher_ticket_info = &used_acc_infos[4];

    // initialize the clock
    cvlr_advance_clock_slot();

    let config = get_vault_config!(config_info);
    let vault_ncn_slasher_ticket = get_vault_ncn_slasher_ticket!(vault_ncn_slasher_ticket_info);
    let state = vault_ncn_slasher_ticket.state;
    let current_slot = Clock::get().unwrap().slot;
    cvlr_assume!(state.slot_added() < current_slot);
    cvlr_assume!(state.slot_removed() < current_slot);

    process_warmup_vault_ncn_slasher_ticket(&program_id, &used_acc_infos).unwrap();

    // after a non-deterministic amount of time passes
    cvlr_advance_clock_slot();

    let vault_ncn_slasher_ticket = get_vault_ncn_slasher_ticket!(vault_ncn_slasher_ticket_info);
    let state = vault_ncn_slasher_ticket
        .state
        .state(Clock::get().unwrap().slot, config.epoch_length())
        .unwrap();
    cvlr_assert!(state == SlotToggleState::WarmUp || state == SlotToggleState::Active);
}
