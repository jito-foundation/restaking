use {
    crate::certora::utils::*,
    cvlr::{
        asserts::cvt::cvt_assert,
        rule,
        nondet,
    },
    cvlr_solana::cvlr_deserialize_nondet_accounts
};

use {
    crate::{*},
    solana_program::account_info::AccountInfo,
    jito_bytemuck::AccountDeserialize, // this is needed by get_vault!
};

/* 
#[rule]
/** This rule is expected to be verified **/
pub fn rule_pubkey_checks_burn() {
    let acc_infos: [AccountInfo; 16] = cvlr_deserialize_nondet_accounts();
    let used_acc_infos = &acc_infos[..11];
    let vault_info = &used_acc_infos[1];
    let vrt_mint_info = &used_acc_infos[3];

    let vault = get_vault!(vault_info);

    let amount_in_arg: u64 = nondet();
    let min_amount_out_arg: u64 = nondet();
    process_burn(&crate::id(), &used_acc_infos, amount_in_arg, min_amount_out_arg).unwrap();

    // vrt_mint should the vrt_mint from the vault
    cvt_assert_eq!(&vault.vrt_mint, vrt_mint_info.key);

}
*/

#[rule]
/** This rule is expected to be verified **/
pub fn rule_pubkey_checks_burn_withdrawal_ticket() {
    let acc_infos: [AccountInfo; 16] = cvlr_deserialize_nondet_accounts();
    let used_acc_infos = &acc_infos[..12];
    let vault_info = &used_acc_infos[1];
    let vrt_mint_info = &used_acc_infos[3];

    let vault = get_vault!(vault_info);

    process_burn_withdrawal_ticket(&crate::id(), &used_acc_infos).unwrap();

    // vrt_mint should the vrt_mint from the vault
    cvt_assert!(&vault.vrt_mint == vrt_mint_info.key);

}

#[rule]
/** This rule is expected to be verified **/
pub fn rule_pubkey_checks_mint() {
    let acc_infos: [AccountInfo; 16] = cvlr_deserialize_nondet_accounts();
    let used_acc_infos = &acc_infos[..10];
    let vault_info = &used_acc_infos[1];
    let vrt_mint_info = &used_acc_infos[2];

    let vault = get_vault!(vault_info);

    let amount_in_arg:u64 = nondet();
    let min_amount_out_arg: u64 = nondet();

    process_mint(&crate::id(), &used_acc_infos,  amount_in_arg, min_amount_out_arg).unwrap();

    // vrt_mint should the vrt_mint from the vault
    cvt_assert!(&vault.vrt_mint == vrt_mint_info.key);

}


#[rule]
/** This rule is expected to be verified **/
pub fn rule_pubkey_checks_update_vault_balance() {
    let acc_infos: [AccountInfo; 16] = cvlr_deserialize_nondet_accounts();
    let used_acc_infos = &acc_infos[..6];
    let vault_info = &used_acc_infos[1];
    let vrt_mint_info = &used_acc_infos[3];

    let vault = get_vault!(vault_info);

    process_update_vault_balance(&crate::id(), &used_acc_infos).unwrap();

    // vrt_mint should the vrt_mint from the vault
    cvt_assert!(&vault.vrt_mint == vrt_mint_info.key);

}

#[rule]
/**
  * This rule is expected to be violated.
  * This was a bug fixed by Jito team. See https://github.com/jito-foundation/restaking/pull/80/
 **/
pub fn rule_pubkey_checks_create_token_metadata() {
    let acc_infos: [AccountInfo; 16] = cvlr_deserialize_nondet_accounts();
    let used_acc_infos = &acc_infos[..7];
    let vault_info = &used_acc_infos[0];
    let vrt_mint_info = &used_acc_infos[2];

    let vault = get_vault!(vault_info);

    // JORGE: we don't generate non-deterministic String objects since they are not used
    // to make control decisions.
    let name_arg = String::default();
    let symbol_arg = String::default();
    let uri_arg = String::default();
    process_create_token_metadata(&crate::id(), &used_acc_infos,
                                  name_arg, symbol_arg, uri_arg).unwrap();

    // vrt_mint should the vrt_mint from the vault
    cvt_assert!(&vault.vrt_mint == vrt_mint_info.key);

}



#[rule]
/** This rule is expected to be verified **/
pub fn rule_pubkey_checks_cooldown_vault_ncn_ticket() {
    let acc_infos: [AccountInfo; 16] = cvlr_deserialize_nondet_accounts();
    let used_acc_infos = &acc_infos[..5];
    let vault_info = &used_acc_infos[1];
    let admin_info = &used_acc_infos[4];
    let vault = get_vault!(vault_info);

    process_cooldown_vault_ncn_ticket(&crate::id(), &used_acc_infos).unwrap();

    // Vault NCN admin should match the provided admin
    cvt_assert!(&vault.ncn_admin == admin_info.key);

}
