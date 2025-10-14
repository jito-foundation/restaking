//! Rules for loading operations

use cvlr::prelude::*;
use cvlr_solana::{cvlr_deserialize_nondet_accounts, cvlr_nondet_pubkey};

use {
    jito_restaking_core::{config::Config, ncn::Ncn, operator::Operator},
    solana_program::{account_info::AccountInfo, pubkey::Pubkey},
};

#[rule]
// A config account can be loaded twice
pub fn rule_load_config_twice_witness() {
    let program_id: Pubkey = cvlr_nondet_pubkey();
    let acc_infos: [AccountInfo; 16] = cvlr_deserialize_nondet_accounts();
    let used_acc_infos = &acc_infos[..1];
    let config_acc = &used_acc_infos[0];
    let expected_writable: bool = nondet();

    Config::load(&program_id, config_acc, expected_writable).unwrap();
    Config::load(&program_id, config_acc, expected_writable).unwrap();
    cvlr_satisfy!(true);
}

#[rule]
// A config account cannot be loaded as a ncn account: expected verified
// Sanity is expected to fail here. The end of the rule body cannot be reached as Ncn::load
// will throw and report a ProgramError, thus the end of the rule body is not reachable.
// Therefore, sanity checks are disabled in loader-assert-false-rules.conf
pub fn rule_load_config_and_ncn() {
    let program_id1 = cvlr_nondet_pubkey();
    let program_id2 = cvlr_nondet_pubkey();
    let acc_infos: [AccountInfo; 16] = cvlr_deserialize_nondet_accounts();
    let used_acc_infos = &acc_infos[..1];
    let config_acc = &used_acc_infos[0];

    Config::load(&program_id1, config_acc, nondet()).unwrap();
    Ncn::load(&program_id2, config_acc, nondet()).unwrap();
    cvlr_assert!(false);
}

#[rule]
// An operator account cannot be loaded as a ncn account: expected verified
// CVLR_based Sanity is expected to fail here. The end of the rule body cannot be reached as Ncn::load
// will throw and report a ProgramError, thus the end of the rule body is not reachable.
// Therefore, sanity checks are disabled in loader-assert-false-rules.conf
pub fn rule_load_operator_and_ncn() {
    let program_id = cvlr_nondet_pubkey();
    let acc_infos: [AccountInfo; 16] = cvlr_deserialize_nondet_accounts();
    let used_acc_infos = &acc_infos[..1];
    let acc_info = &used_acc_infos[0];
    let expected_writable: bool = nondet();

    Operator::load(&program_id, acc_info, expected_writable).unwrap();
    Ncn::load(&program_id, acc_info, expected_writable).unwrap();
    cvlr_assert!(false);
}
