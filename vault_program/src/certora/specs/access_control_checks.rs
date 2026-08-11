use {
    crate::certora::utils::*,
    cvlr::{
        rule, cvlr_assert
    },
    cvlr_solana::acc_infos_with_mem_layout,
};
use cvlr::nondet;
use cvlr_solana::cvlr_nondet_pubkey;
use jito_bytemuck::AccountDeserialize;
use jito_vault_sdk::instruction::VaultAdminRole;
use solana_program::account_info::AccountInfo;
use crate::{
    burn_withdrawal_ticket::process_burn_withdrawal_ticket, enqueue_withdrawal::process_enqueue_withdrawal, process_mint, process_update_vault_balance, set_admin::process_set_admin, set_capacity::process_set_deposit_capacity, set_config_admin::process_set_config_admin, set_fees::process_set_fees, set_is_paused::process_set_is_paused, set_program_fee::process_set_program_fee, set_program_fee_wallet::process_set_program_fee_wallet, set_secondary_admin::process_set_secondary_admin 
};
use solana_program::pubkey::Pubkey;

// Rules to check that accounts are passed with correct owners and pubkeys

pub fn check_config_and_vault_owner(
    program_id: Pubkey,
    config_info: &AccountInfo<'static>,
    vault_info: &AccountInfo<'static>,
) {
    cvlr_assert!(*config_info.owner == program_id);
    cvlr_assert!(*vault_info.owner == program_id);
}

#[rule]
pub fn rule_burn_withdrawal_perms() {
    
    let program_id = cvlr_nondet_pubkey();

    // 12 required accounts
    // 1 optional account
    let acc_infos: [AccountInfo; 16] = acc_infos_with_mem_layout!();
    let used_acc_infos = &acc_infos[..13];
    
    let config_info = &acc_infos[0];
    let vault_info = &used_acc_infos[1];
    let vrt_mint_info = &used_acc_infos[3];
    let staker_info = &used_acc_infos[4];
    let vault_staker_withdrawal_ticket_info = &used_acc_infos[6];
    let mint_burn_admin_info = &used_acc_infos[12];

    let vault = get_vault!(vault_info);
    let withdrawal_ticket = get_vault_staker_withdrawal_ticket!(vault_staker_withdrawal_ticket_info);
    let vault_staker_withdrawal_ticket_owner = *vault_staker_withdrawal_ticket_info.owner;

    process_burn_withdrawal_ticket(&program_id, &used_acc_infos).unwrap();

    check_config_and_vault_owner(program_id, config_info, vault_info);
    cvlr_assert!(vault_staker_withdrawal_ticket_owner == program_id);
    cvlr_assert!(vault_info.key == &withdrawal_ticket.vault);
    cvlr_assert!(vault.vrt_mint == *vrt_mint_info.key);
    cvlr_assert!(withdrawal_ticket.staker == *staker_info.key);
    if vault.mint_burn_admin != Pubkey::default() {
        cvlr_assert!(vault.mint_burn_admin == *mint_burn_admin_info.key);
    }
}

#[rule]
pub fn rule_mint_perms() {
    let program_id = cvlr_nondet_pubkey();

    let acc_infos: [AccountInfo; 16] = acc_infos_with_mem_layout!();
    let used_acc_infos = &acc_infos[..10];

    let config_info = &acc_infos[0];
    let vault_info =  &used_acc_infos[1];
    let vrt_mint_info = &used_acc_infos[2];
    let depositor = &used_acc_infos[3];
    let depositor_token_account = &used_acc_infos[4];
    let vault_token_account = &used_acc_infos[5];
    let mint_burn_admin_info = &used_acc_infos[9];
    
    let vault = get_vault!(vault_info);

    let amount_token_in_arg:u64 = nondet();
    let min_amount_token_out_arg:u64 = nondet();
    
    process_mint(
        &program_id,
        &used_acc_infos,
        amount_token_in_arg,
        min_amount_token_out_arg
    ).unwrap();

    check_config_and_vault_owner(program_id, config_info, vault_info);
    cvlr_assert!(vault.vrt_mint == *vrt_mint_info.key);
    cvlr_assert!(*depositor.key != *vault_info.key);
    cvlr_assert!(*depositor_token_account.key != *vault_token_account.key);
    if vault.mint_burn_admin != Pubkey::default() {
        cvlr_assert!(vault.mint_burn_admin == *mint_burn_admin_info.key);
    }
}

#[rule]
pub fn rule_update_vault_balance_perms() {
    let program_id = cvlr_nondet_pubkey();

    let acc_infos: [AccountInfo; 16] = acc_infos_with_mem_layout!();
    let used_acc_infos = &acc_infos[..6];

    let config_info = &acc_infos[0];
    let vault_info =  &used_acc_infos[1];
    let vrt_mint_info = &used_acc_infos[3];
    
    process_update_vault_balance(&program_id, &used_acc_infos).unwrap();

    let vault = get_vault!(vault_info);

    check_config_and_vault_owner(program_id, config_info, vault_info);
    cvlr_assert!(vault.vrt_mint == *vrt_mint_info.key);
}

#[rule]
pub fn rule_enqueue_withdrawal_perms() {
    let program_id = cvlr_nondet_pubkey();

    let acc_infos: [AccountInfo; 16] = acc_infos_with_mem_layout!();
    let used_acc_infos = &acc_infos[..10];
    
    let config_info = &acc_infos[0];
    let vault_info =  &used_acc_infos[1];
    let vault_staker_withdrawal_ticket_info = &used_acc_infos[2];
    let staker_info = &used_acc_infos[4];
    let mint_burn_admin_info = &used_acc_infos[9];
    
    let vrt_amount = nondet();
    process_enqueue_withdrawal(&program_id, &used_acc_infos, vrt_amount).unwrap();

    let withdrawal_ticket = get_vault_staker_withdrawal_ticket!(vault_staker_withdrawal_ticket_info);

    let vault = get_vault!(vault_info);

    check_config_and_vault_owner(program_id, config_info, vault_info);
    cvlr_assert!(withdrawal_ticket.vault == *vault_info.key);
    cvlr_assert!(withdrawal_ticket.staker == *staker_info.key);
    if vault.mint_burn_admin != Pubkey::default() {
        cvlr_assert!(vault.mint_burn_admin == *mint_burn_admin_info.key);
    }
}


#[rule]
pub fn rule_set_admin_perms() {
    let program_id = cvlr_nondet_pubkey();
    
    let acc_infos: [AccountInfo; 16] = acc_infos_with_mem_layout!();
    let used_acc_infos = &acc_infos[..4];

    let config_info = &acc_infos[0];
    let vault_info =  &used_acc_infos[1];
    let old_admin_info = &used_acc_infos[2];
    let new_admin_info = &used_acc_infos[3];
    
    let vault = get_vault!(vault_info);
    let vault_admin_old = vault.admin;
    
    process_set_admin(&program_id, used_acc_infos).unwrap();

    let vault = get_vault!(vault_info);
    let vault_admin_new = vault.admin;

    check_config_and_vault_owner(program_id, config_info, vault_info);
    cvlr_assert!(vault_admin_old == *old_admin_info.key);
    cvlr_assert!(vault_admin_new == *new_admin_info.key);
}

#[rule]
pub fn rule_set_deposit_capacity_perms() {
    let program_id = cvlr_nondet_pubkey();
    
    let acc_infos: [AccountInfo; 16] = acc_infos_with_mem_layout!();
    let used_acc_infos = &acc_infos[..3];

    let config_info = &acc_infos[0];
    let vault_info =  &used_acc_infos[1];
    let vault = get_vault!(vault_info);
    let vault_capacity_admin = &used_acc_infos[2];

    let capacity: u64 = nondet();
    process_set_deposit_capacity(&program_id, used_acc_infos, capacity).unwrap();

    check_config_and_vault_owner(program_id, config_info, vault_info);
    cvlr_assert!(vault.capacity_admin == *vault_capacity_admin.key);
}

#[rule]
pub fn rule_set_config_admin_perms() {
    let program_id = cvlr_nondet_pubkey();
    
    let acc_infos: [AccountInfo; 16] = acc_infos_with_mem_layout!();
    let used_acc_infos = &acc_infos[..3];

    let config_info = &acc_infos[0];
    let old_admin_info = &used_acc_infos[1];
    let new_admin_info = &used_acc_infos[2];
    
    let config = get_vault_config!(config_info);
    let config_admin_old = config.admin;

    process_set_config_admin(&program_id, used_acc_infos).unwrap();

    let config = get_vault_config!(config_info);
    let config_admin_new = config.admin;

    cvlr_assert!(*config_info.owner == program_id);
    cvlr_assert!(config_admin_old == *old_admin_info.key);
    cvlr_assert!(config_admin_new == *new_admin_info.key);
}

#[rule]
pub fn rule_set_fees_perms() {
    let program_id = cvlr_nondet_pubkey();
    
    let acc_infos: [AccountInfo; 16] = acc_infos_with_mem_layout!();
    let used_acc_infos = &acc_infos[..3];

    let config_info = &acc_infos[0];
    let vault_info =  &used_acc_infos[1];
    let vault_fee_admin_info = &used_acc_infos[2];

    let deposit_fee_bps: Option<u16> = nondet();

    // nisarg: if I set withdrawal_fee_bps and reward_fee_bps as below, 
    // I get a prover error
    // "Error: Cannot analyze rule rule_not_vacuous_cvlr:
    // sbf.support.UnknownStackContentError: [3002] stack location is not accessible
    // source: vault_program/src/set_fees.rs:46"

    // let withdrawal_fee_bps: Option<u16> = nondet();
    // let reward_fee_bps: Option<u16> = nondet();

    let withdrawal_fee_bps: Option<u16> = Some(nondet());
    let reward_fee_bps: Option<u16> = Some(nondet());

    process_set_fees(
        &program_id, 
        used_acc_infos,
        deposit_fee_bps,
        withdrawal_fee_bps,
        reward_fee_bps
    ).unwrap();

    let vault = get_vault!(vault_info);

    check_config_and_vault_owner(program_id, config_info, vault_info);
    cvlr_assert!(vault.fee_admin == *vault_fee_admin_info.key);
}

#[rule]
pub fn rule_set_is_paused_perms() {
    let program_id = cvlr_nondet_pubkey();
    
    let acc_infos: [AccountInfo; 16] = acc_infos_with_mem_layout!();
    let used_acc_infos = &acc_infos[..3];

    let config_info = &acc_infos[0];
    let vault_info =  &used_acc_infos[1];
    let admin_info = &used_acc_infos[2];

    let is_paused: bool = nondet();

    process_set_is_paused(
        &program_id, 
        used_acc_infos,
        is_paused
    ).unwrap();

    let vault = get_vault!(vault_info);

    check_config_and_vault_owner(program_id, config_info, vault_info);
    cvlr_assert!(vault.admin == *admin_info.key);
}

#[rule]
pub fn rule_set_program_fee_wallet_perms() {
    let program_id = cvlr_nondet_pubkey();
    
    let acc_infos: [AccountInfo; 16] = acc_infos_with_mem_layout!();
    let used_acc_infos = &acc_infos[..3];

    let config_info = &acc_infos[0];
    let config_fee_admin_info =  &used_acc_infos[1];
    let new_fee_wallet_info = &used_acc_infos[2];

    process_set_program_fee_wallet(&program_id, used_acc_infos).unwrap();

    let config = get_vault_config!(config_info);

    cvlr_assert!(*config_info.owner == program_id);
    cvlr_assert!(*config_fee_admin_info.key == config.fee_admin);
    cvlr_assert!(*new_fee_wallet_info.key == config.program_fee_wallet);
}

#[rule]
pub fn rule_set_program_fee_perms() {
    let program_id = cvlr_nondet_pubkey();
    
    let acc_infos: [AccountInfo; 16] = acc_infos_with_mem_layout!();
    let used_acc_infos = &acc_infos[..2];

    let config_info = &acc_infos[0];
    let config_admin_info =  &used_acc_infos[1];

    let new_fee_bps: u16 = nondet();

    process_set_program_fee(
        &program_id, 
        used_acc_infos,
        new_fee_bps
    ).unwrap();

    let config = get_vault_config!(config_info);

    cvlr_assert!(*config_info.owner == program_id);
    cvlr_assert!(*config_admin_info.key == config.admin);
}

#[rule]
pub fn rule_set_secondary_admin_perms() {
    let program_id = cvlr_nondet_pubkey();
    
    let acc_infos: [AccountInfo; 16] = acc_infos_with_mem_layout!();
    let used_acc_infos = &acc_infos[..4];

    let config_info = &acc_infos[0];
    let vault_info = &acc_infos[1];
    let admin_info =  &used_acc_infos[2];

    let role: VaultAdminRole = nondet_vault_admin_role();

    process_set_secondary_admin(
        &program_id, 
        used_acc_infos,
        role
    ).unwrap();

    let vault = get_vault!(vault_info);

    check_config_and_vault_owner(program_id, config_info, vault_info);
    cvlr_assert!(*admin_info.key == vault.admin);
}
