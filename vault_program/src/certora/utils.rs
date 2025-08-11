#![allow(unused_macros)]
#![allow(unused_imports)]

use cvlr::nondet;
use solana_program::account_info::AccountInfo;

use jito_bytemuck::{types::PodU64, AccountDeserialize, Discriminator};
use jito_vault_sdk::instruction::VaultAdminRole;

pub fn nondet_vault_admin_role() -> VaultAdminRole {
    let x: u8 = nondet();
    let res = match x {
        0 => VaultAdminRole::CapacityAdmin,
        1 => VaultAdminRole::DelegationAdmin,
        2 => VaultAdminRole::FeeAdmin,
        3 => VaultAdminRole::FeeWallet,
        4 => VaultAdminRole::MintBurnAdmin,
        5 => VaultAdminRole::NcnAdmin,
        6 => VaultAdminRole::OperatorAdmin,
        7 => VaultAdminRole::SlasherAdmin,
        8 => VaultAdminRole::DelegateAssetAdmin,
        _ => panic!(),
    };
    return res;
}

pub fn safe_mul_div(a: u64, b: u64, c: u64) -> u64 {
    cvlr::cvlr_assert_gt!(c, 0);
    if c > 0 {
        let r = (a as u128).wrapping_mul(b as u128).wrapping_div(c as u128);
        cvlr::cvlr_assert_le!(r, u64::MAX as u128);
        r as u64
    } else { 
        // -- return a specific sentinel value if divisor is 0
        // -- this matches hardware division
        u64::MAX 
    }
}

pub fn safe_mul_div_u128(a: u64, b: u64, c: u64) -> u128 {
    cvlr::cvlr_assume!(cvlr::is_u64(a));
    cvlr::cvlr_assume!(cvlr::is_u64(b));
    cvlr::cvlr_assume!(cvlr::is_u64(c));
    cvlr::cvlr_assert_gt!(c, 0);
    if c > 0 {
        let r = (a as u128).wrapping_mul(b as u128).wrapping_div(c as u128);
        r
    } else { 
        // -- return a specific sentinel value if divisor is 0
        // -- this matches hardware division
        u128::MAX 
    }
}



macro_rules! get_vault {
    ($acc_info: expr) => {{
        let data = $acc_info.data.borrow();
        let vault = jito_vault_core::vault::Vault::try_from_slice_unchecked(&data).unwrap();
        cvlr::cvlr_assume!(cvlr::is_u64(vault.vrt_supply()));
        cvlr::cvlr_assume!(cvlr::is_u64(vault.tokens_deposited()));
        *vault
    }};
}
pub(crate) use get_vault;

macro_rules! get_vault_config {
    ($acc_info: expr) => {{
        let data = $acc_info.data.borrow();
        let res = jito_vault_core::config::Config::try_from_slice_unchecked(&data).unwrap();
        *res
    }};
}
pub(crate) use get_vault_config;

macro_rules! get_vault_operator_delegation {
    ($acc_info: expr) => {{
        let data = $acc_info.data.borrow();
        let res = jito_vault_core::vault_operator_delegation::VaultOperatorDelegation::try_from_slice_unchecked(&data).unwrap();
        *res
    }};
}
pub(crate) use get_vault_operator_delegation;

macro_rules! get_vault_update_state_tracker {
    ($acc_info: expr) => {{
        let data = $acc_info.data.borrow();
        let res = jito_vault_core::vault_update_state_tracker::VaultUpdateStateTracker::try_from_slice_unchecked(&data).unwrap();
        *res
    }};
}

pub(crate) use get_vault_update_state_tracker;


macro_rules! get_vault_staker_withdrawal_ticket {
    ($acc_info: expr) => {{
        let data = $acc_info.data.borrow();
        let res = jito_vault_core::vault_staker_withdrawal_ticket::VaultStakerWithdrawalTicket::try_from_slice_unchecked(&data).unwrap();
        *res
    }};
}
pub(crate) use get_vault_staker_withdrawal_ticket;

macro_rules! get_vault_ncn_ticket {
    ($acc_info: expr) => {{
        let data = $acc_info.data.borrow();
        let res = jito_vault_core::vault_ncn_ticket::VaultNcnTicket::try_from_slice_unchecked(&data).unwrap();
        *res
    }};
}
pub(crate) use get_vault_ncn_ticket;

macro_rules! get_vault_ncn_slasher_ticket {
    ($acc_info: expr) => {{
        let data = $acc_info.data.borrow();
        let res = jito_vault_core::vault_ncn_slasher_ticket::VaultNcnSlasherTicket::try_from_slice_unchecked(&data).unwrap();
        *res
    }};
}
pub(crate) use get_vault_ncn_slasher_ticket;
