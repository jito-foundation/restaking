//! Simple rules to check basic functionality

use {crate::certora::utils::*, cvlr::prelude::*, cvlr_solana::cvlr_deserialize_nondet_accounts};

use {
    crate::*,
    jito_bytemuck::AccountDeserialize,
    jito_jsm_core::slot_toggle::SlotToggleState,
    jito_restaking_core::config::Config,
    jito_restaking_core::ncn::Ncn,
    jito_restaking_core::operator::Operator,
    jito_restaking_core::operator_vault_ticket::OperatorVaultTicket,
    jito_restaking_sdk::instruction::NcnAdminRole,
    solana_program::{account_info::AccountInfo, clock::Clock, sysvar::Sysvar},
};

#[rule]
/// This is expected to be verified
pub fn rule_integrity_ncn_set_admin() {
    let acc_infos: [AccountInfo; 16] = cvlr_deserialize_nondet_accounts();
    let used_acc_infos = &acc_infos[..3];
    let ncn_info = &used_acc_infos[0];
    let new_admin_info = &used_acc_infos[2];

    process_ncn_set_admin(&crate::id(), &used_acc_infos).unwrap();

    let ncn: Ncn = get_ncn!(ncn_info);
    cvlr_assert!(ncn.admin == *new_admin_info.key);
}

#[rule]
/// This is expected to be verified
pub fn rule_integrity_ncn_set_admin_secondary_admins() {
    let acc_infos: [AccountInfo; 16] = cvlr_deserialize_nondet_accounts();
    let used_acc_infos = &acc_infos[..3];
    let ncn_info = &used_acc_infos[0];
    let old_admin_info = &used_acc_infos[1];
    let new_admin_info = &used_acc_infos[2];

    let old_ncn: Ncn = get_ncn!(ncn_info);
    let old_operator_admin = old_ncn.operator_admin;
    let old_vault_admin = old_ncn.vault_admin;
    let old_slasher_admin = old_ncn.slasher_admin;
    let old_delegate_admin = old_ncn.delegate_admin;
    let old_metadata_admin = old_ncn.metadata_admin;

    process_ncn_set_admin(&crate::id(), &used_acc_infos).unwrap();

    let ncn: Ncn = get_ncn!(ncn_info);

    // TOOD: review newly available admins
    if old_operator_admin == *old_admin_info.key {
        cvlr_assert!(ncn.operator_admin == *new_admin_info.key);
    }
    if old_vault_admin == *old_admin_info.key {
        cvlr_assert!(ncn.vault_admin == *new_admin_info.key);
    }
    if old_slasher_admin == *old_admin_info.key {
        cvlr_assert!(ncn.slasher_admin == *new_admin_info.key);
    }
    if old_delegate_admin == *old_admin_info.key {
        cvlr_assert!(ncn.delegate_admin == *new_admin_info.key);
    }
    if old_metadata_admin == *old_admin_info.key {
        cvlr_assert!(ncn.ncn_program_admin == *new_admin_info.key);
    }
}

fn nondet_ncn_admin_role() -> NcnAdminRole {
    let x: u8 = nondet();
    let res = match x {
        0 => NcnAdminRole::OperatorAdmin,
        1 => NcnAdminRole::VaultAdmin,
        2 => NcnAdminRole::SlasherAdmin,
        3 => NcnAdminRole::DelegateAdmin,
        4 => NcnAdminRole::MetadataAdmin,
        5 => NcnAdminRole::WeightTableAdmin,
        6 => NcnAdminRole::NcnProgramAdmin,
        _ => panic!(),
    };
    return res;
}

#[rule]
/// This is expected to be verified
pub fn rule_integrity_ncn_set_secondary_admin() {
    let acc_infos: [AccountInfo; 16] = cvlr_deserialize_nondet_accounts();
    let used_acc_infos = &acc_infos[..3];
    let ncn_info = &used_acc_infos[0];
    let new_admin_info = &used_acc_infos[2];

    let old_ncn: Ncn = get_ncn!(ncn_info);
    let role_arg = nondet_ncn_admin_role();
    let cloned_role_arg = nondet_ncn_admin_role();
    // This is needed because NcnAdminRole does not have the Clone/Copy traits.
    cvlr_assume!(role_arg == cloned_role_arg);
    process_ncn_set_secondary_admin(&crate::id(), &used_acc_infos, cloned_role_arg).unwrap();

    let ncn: Ncn = get_ncn!(ncn_info);

    // The main admin cannot change
    cvlr_assert!(ncn.admin == old_ncn.admin);

    match role_arg {
        NcnAdminRole::OperatorAdmin => {
            cvlr_assert!(ncn.operator_admin.eq(new_admin_info.key));
        }
        NcnAdminRole::VaultAdmin => {
            cvlr_assert!(ncn.vault_admin.eq(new_admin_info.key));
        }
        NcnAdminRole::SlasherAdmin => {
            cvlr_assert!(ncn.slasher_admin.eq(new_admin_info.key));
        }
        NcnAdminRole::DelegateAdmin => {
            cvlr_assert!(ncn.delegate_admin.eq(new_admin_info.key));
        }
        NcnAdminRole::MetadataAdmin => {
            cvlr_assert!(ncn.metadata_admin.eq(new_admin_info.key));
        }
        _ => {
            cvlr_assert!(false);
        }
    }
}

#[rule]
/// This is expected to be verified
pub fn rule_integrity_operator_set_admin() {
    let acc_infos: [AccountInfo; 16] = cvlr_deserialize_nondet_accounts();
    let used_acc_infos = &acc_infos[..3];
    let operator_info = &used_acc_infos[0];
    let new_admin_info = &used_acc_infos[2];

    process_set_node_operator_admin(&crate::id(), &used_acc_infos).unwrap();

    let operator: Operator = get_operator!(operator_info);
    cvlr_assert!(operator.admin == *new_admin_info.key);
}

#[rule]
/// This is expected to be verified
pub fn rule_integrity_process_cooldown_operator_vault_ticket() {
    let acc_infos: [AccountInfo; 16] = cvlr_deserialize_nondet_accounts();
    let used_acc_infos = &acc_infos[..5];
    let config_info = &used_acc_infos[0];
    let operator_vault_ticket_info = &used_acc_infos[3];

    let config = get_config!(config_info);
    process_cooldown_operator_vault_ticket(&crate::id(), &used_acc_infos).unwrap();

    let operator_vault_ticket = get_operator_vault_ticket!(operator_vault_ticket_info);
    let state = operator_vault_ticket
        .state
        .state(Clock::get().unwrap().slot, config.epoch_length())
        .unwrap();
    cvlr_assert!(state == SlotToggleState::Inactive || state == SlotToggleState::Cooldown);
}
