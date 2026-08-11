//! Simple rules to check basic functionality

use {
    cvlr::{prelude::*, mathint::NativeInt},
    cvlr_solana::cvlr_nondet_pubkey,
    jito_restaking_core::ncn::Ncn,
};

#[rule]
pub fn rule_ncn_increment() {
    let mut ncn = Ncn::new(
        cvlr_nondet_pubkey(),
        cvlr_nondet_pubkey(),
        nondet::<u64>(),
        nondet::<u8>(),
    );

    let vault_count_before = NativeInt::from(ncn.vault_count());
    ncn.increment_vault_count().unwrap();
    let vault_count_after = ncn.vault_count();
    cvlr_assert_eq!(vault_count_before + 1, NativeInt::from(vault_count_after));
}
