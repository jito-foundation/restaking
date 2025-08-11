//! Simple rules to check basic functionality

use cvlr::{mathint::NativeInt, prelude::*};
use jito_vault_core::vault::Vault;

#[rule]
/// This rule is expected to be verified
pub fn rule_additivity_of_calculate_deposit_fee() {
    let vault: Vault = nondet();

    let lr_amount_x: u64 = nondet();
    let lr_amount_y: u64 = nondet();
    let lr_amount_xy: u64 = lr_amount_x.checked_add(lr_amount_y).unwrap();

    let fee_xy = vault.calculate_deposit_fee(lr_amount_xy).unwrap();
    let fee_x = vault.calculate_deposit_fee(lr_amount_x).unwrap();
    let fee_y = vault.calculate_deposit_fee(lr_amount_y).unwrap();

    cvlr_assert_ge!(
        NativeInt::from(fee_x) + NativeInt::from(fee_y),
        NativeInt::from(fee_xy)
    );
}

#[rule]
/// This rule is expected to be verified
pub fn rule_calculate_fee_non_zero() {
    let vault: Vault = nondet();
    let lr_amount: u64 = nondet();
    cvlr_assume!(lr_amount > 0);
    cvlr_assume!(vault.deposit_fee_bps() > 0);
    let fee = vault.calculate_deposit_fee(lr_amount).unwrap();
    cvlr_assert!(fee > 0);
}
