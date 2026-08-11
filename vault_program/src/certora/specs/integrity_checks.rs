use cvlr::prelude::*;
use cvlr_solana::{cvlr_deserialize_nondet_accounts, token::spl_token_account_get_amount};
use jito_vault_core::{delegation_state::DelegationState, vault::Vault};
use {crate::certora::utils::*, jito_bytemuck::AccountDeserialize};

use crate::{
    burn_withdrawal_ticket::process_burn_withdrawal_ticket, process_mint,
    process_update_vault_balance,
};
use jito_vault_sdk::instruction::WithdrawalAllocationMethod;
use solana_program::{account_info::AccountInfo, clock::Clock, sysvar::Sysvar};

use jito_jsm_core::get_epoch;

#[rule]
// This rule is expected to be verified.
pub fn rule_integrity_mint() {
    let acc_infos: [AccountInfo; 16] = cvlr_deserialize_nondet_accounts();

    let used_acc_infos = &acc_infos[..10];

    let vault_info = &used_acc_infos[1];
    let depositor_token_account = &used_acc_infos[4];
    let vault_token_account = &used_acc_infos[5];
    let depositor_vrt_token_account = &used_acc_infos[6];
    let vault_fee_token_account = &used_acc_infos[7];

    // These three accounts are SPL token accounts
    let depositor_token_old = spl_token_account_get_amount(depositor_token_account);
    let depositor_vrt_old = spl_token_account_get_amount(depositor_vrt_token_account);
    let vault_fee_vrt_old = spl_token_account_get_amount(vault_fee_token_account);

    let vault_old: Vault = get_vault!(vault_info);
    let vrt_supply_old = vault_old.vrt_supply();
    let vault_tokens_old = vault_old.tokens_deposited();

    // 1. transfer amount_in from depositor_token_account to vault_token_account
    // 2. mint "vrt_to_depositor" to depositor_vrt_token_account
    // 3. mint "vrt_to_fees" to vault_fee_token_account
    //
    // where (vrt_to_depositor, vrt_to_fees) = vault.mint_with_fee(amount_in, min_amount_out)
    let amount_token_in_arg: u64 = nondet();
    let min_amount_token_out_arg: u64 = nondet();
    process_mint(
        &crate::id(),
        &used_acc_infos,
        amount_token_in_arg,
        min_amount_token_out_arg,
    )
    .unwrap();

    // -- all tokens send to mint are used
    let depositor_token = spl_token_account_get_amount(depositor_token_account);
    let vault: Vault = get_vault!(vault_info);
    let vrt_supply = vault.vrt_supply();
    let vault_tokens = vault.tokens_deposited();

    if depositor_token_account.key != vault_token_account.key {
        // The following assertions should pass without any precondition
        cvlr_assert!(depositor_token == depositor_token_old - amount_token_in_arg);
        cvlr_assert!(vault_tokens - vault_tokens_old == depositor_token_old - depositor_token);
    } else {
        cvlr_assert!(depositor_token == depositor_token_old);
    }

    // vault token supply always updated after mint
    cvlr_assert!(vault_tokens == vault_tokens_old + amount_token_in_arg);

    let depositor_vrt = spl_token_account_get_amount(depositor_vrt_token_account);
    let vault_fee_vrt = spl_token_account_get_amount(vault_fee_token_account);

    // -- vrt_supply increases
    cvlr_assert!(vrt_supply >= vrt_supply_old);

    // -- vault_fee_vrt increases
    cvlr_assert!(vault_fee_vrt >= vault_fee_vrt_old);

    let minted_vrt_supply = vrt_supply - vrt_supply_old;
    let delta_vault_fee_vrt = vault_fee_vrt - vault_fee_vrt_old;

    let delta_depositor_vrt = depositor_vrt - depositor_vrt_old;
    cvlr_assert!(minted_vrt_supply == delta_depositor_vrt + delta_vault_fee_vrt);
}

#[rule]
// This rule is expected to be violated.
pub fn rule_integrity_mint_should_fail() {
    let acc_infos: [AccountInfo; 16] = cvlr_deserialize_nondet_accounts();

    let used_acc_infos = &acc_infos[..10];

    let vault_info = &used_acc_infos[1];
    let depositor_token_account = &used_acc_infos[4];
    let depositor_vrt_token_account = &used_acc_infos[6];
    let vault_fee_token_account = &used_acc_infos[7];

    // These three accounts are SPL token accounts
    let depositor_token_old = spl_token_account_get_amount(depositor_token_account);
    let depositor_vrt_old = spl_token_account_get_amount(depositor_vrt_token_account);
    let vault_fee_vrt_old = spl_token_account_get_amount(vault_fee_token_account);

    let vault_old: Vault = get_vault!(vault_info);
    let vrt_supply_old = vault_old.vrt_supply();
    let vault_tokens_old = vault_old.tokens_deposited();

    // 1. transfer amount_in from depositor_token_account to vault_token_account
    // 2. mint "vrt_to_depositor" to depositor_vrt_token_account
    // 3. mint "vrt_to_fees" to vault_fee_token_account
    //
    // where (vrt_to_depositor, vrt_to_fees) = vault.mint_with_fee(amount_in, min_amount_out)
    let amount_token_in_arg: u64 = nondet();
    let min_amount_token_out_arg: u64 = nondet();
    process_mint(
        &crate::id(),
        &used_acc_infos,
        amount_token_in_arg,
        min_amount_token_out_arg,
    )
    .unwrap();

    // -- all tokens send to mint are used
    let depositor_token = spl_token_account_get_amount(depositor_token_account);
    let vault: Vault = get_vault!(vault_info);
    let vrt_supply = vault.vrt_supply();
    let vault_tokens = vault.tokens_deposited();

    // Depositor token amount always update after mint
    cvlr_assert!(depositor_token == depositor_token_old - amount_token_in_arg);
    // Token preservation invariant
    cvlr_assert!(vault_tokens - vault_tokens_old == depositor_token_old - depositor_token);

    // vault token supply always updated after mint
    cvlr_assert!(vault_tokens == vault_tokens_old + amount_token_in_arg);

    let depositor_vrt = spl_token_account_get_amount(depositor_vrt_token_account);
    let vault_fee_vrt = spl_token_account_get_amount(vault_fee_token_account);

    // -- vrt_supply increases
    cvlr_assert!(vrt_supply >= vrt_supply_old);

    // -- vault_fee_vrt increases
    cvlr_assert!(vault_fee_vrt >= vault_fee_vrt_old);

    let minted_vrt_supply = vrt_supply - vrt_supply_old;
    let delta_vault_fee_vrt = vault_fee_vrt - vault_fee_vrt_old;

    let delta_depositor_vrt = depositor_vrt - depositor_vrt_old;
    cvlr_assert!(minted_vrt_supply == delta_depositor_vrt + delta_vault_fee_vrt);
}

fn cvt_vault_inv(
    vault: &Vault,
    vault_token_info: Option<&AccountInfo>,
    vrt_mint_info: Option<&AccountInfo>,
) -> bool {
    // vault is ok
    let mut res = cvt_vault_is_ok(vault);

    // minted tokens is same as vault knows
    if let Some(vrt_mint) = vrt_mint_info {
        let vrt_supply = vault.vrt_supply();
        let mint_vrt_supply = cvlr_solana::token::spl_mint_get_supply(vrt_mint);
        res &= vrt_supply == mint_vrt_supply;
    }

    // vault token account has at least as much funds as the vault expects
    if let Some(vault_token) = vault_token_info {
        let vault_token_balance = spl_token_account_get_amount(vault_token);
        let vault_tokens_deposited = vault.tokens_deposited();
        res &= vault_tokens_deposited <= vault_token_balance;
    }

    res
}

/// Representation invariants of Vault
fn cvt_vault_is_ok(vault: &Vault) -> bool {
    let total_security = vault.delegation_state.total_security();

    // the stake is covered by deposited tokens
    if let Ok(total_security) = total_security {
        return total_security <= vault.tokens_deposited();
    }
    return false;
}

#[rule]
/// the (SPL) vrt mint account and the vault vrt supply must be equal
/// Expected to be verified
pub fn rule_integrity_update_vault_balance_1() {
    let acc_infos: [AccountInfo; 16] = cvlr_deserialize_nondet_accounts();
    let used_acc_infos = &acc_infos[..6];
    let vault_info = &used_acc_infos[1];
    let vault_token_account = &used_acc_infos[2];
    let vrt_mint_account = &used_acc_infos[3];
    let _vault_fee_token_account = &used_acc_infos[4];

    // let minted_tokens_old = spl_mint_get_supply(vrt_mint_account);
    let vault_old: Vault = get_vault!(vault_info);
    cvlr_assume!(cvt_vault_inv(
        &vault_old,
        vault_token_account.into(),
        vrt_mint_account.into()
    ));

    process_update_vault_balance(&crate::id(), &used_acc_infos).unwrap();

    let vault: Vault = get_vault!(vault_info);
    cvlr_assert!(cvt_vault_inv(
        &vault,
        vault_token_account.into(),
        vrt_mint_account.into()
    ));
}

#[rule]
/// The vault fee token account is increased by the same amount that minted vrt tokens.
/// Expected to be verified
pub fn rule_integrity_update_vault_balance_2() {
    let acc_infos: [AccountInfo; 16] = cvlr_deserialize_nondet_accounts();
    let used_acc_infos = &acc_infos[..6];
    let vault_info = &used_acc_infos[1];
    let vault_fee_token_account = &used_acc_infos[4];

    let vault_old: Vault = get_vault!(vault_info);
    let vault_vrt_supply_old = vault_old.vrt_supply();
    let vault_fees_old: u64 = spl_token_account_get_amount(vault_fee_token_account);

    process_update_vault_balance(&crate::id(), &used_acc_infos).unwrap();

    let vault: Vault = get_vault!(vault_info);
    let vault_vrt_supply = vault.vrt_supply();
    let vault_fees: u64 = spl_token_account_get_amount(vault_fee_token_account);

    cvlr_assert!(vault_vrt_supply >= vault_vrt_supply_old);
    let delta_vault_vrt_supply = vault_vrt_supply - vault_vrt_supply_old;

    cvlr_assert!(vault_fees >= vault_fees_old);
    let delta_vault_fees = vault_fees - vault_fees_old;

    cvlr_assert!(delta_vault_fees == delta_vault_vrt_supply);
}

#[rule]
/// If there are fees and rewards and `process_update_vault_balance` does not revert then
/// vault's vrt supply should strictly increase.
///
/// Expected to be verified but it is violated. This is probably a bug.
/// See cex here https://prover.certora.com/output/26873/4639797ada55425ba3286be6880e4c30?anonymousKey=ef9e7ea51d31b042ee0c11665a42eb976be846a6
///
/// UPDATE: the code was fixed after 2nd audit. Thus, the rule it is expected to be verified.
pub fn rule_update_vault_balance_assess_fees() {
    let acc_infos: [AccountInfo; 16] = cvlr_deserialize_nondet_accounts();
    let used_acc_infos = &acc_infos[..6];
    let vault_info = &used_acc_infos[1];
    let vault_token_account = &used_acc_infos[2];

    let vault_old: Vault = get_vault!(vault_info);
    let vrt_supply_old = vault_old.vrt_supply();
    let token_deposited_old = vault_old.tokens_deposited();

    let balance: u64 = spl_token_account_get_amount(vault_token_account);

    // Assumption #1: there is some fee
    cvlr_assume!(vault_old.reward_fee_bps() > 0);
    cvlr_assume!(vault_old.reward_fee_bps() <= 10_000);

    // Assumption #2: there are some vrt tokens
    cvlr_assume!(vrt_supply_old > 0);

    // these assumptions only used to generate small values in the counterexample
    cvlr_assume!(vrt_supply_old <= 1_000_000);
    cvlr_assume!(token_deposited_old <= 1_000_000);

    process_update_vault_balance(&crate::id(), &used_acc_infos).unwrap();

    let vault: Vault = get_vault!(vault_info);
    let vrt_supply = vault.vrt_supply();
    let token_deposited = vault.tokens_deposited();

    // Assumption #3: there is some reward
    cvlr_assume!(balance > token_deposited_old);
    // Assumption #4: vault's tokens deposited is synchronized with the vault_token_account
    cvlr_assume!(balance == token_deposited);

    // this indirectly checks that fee was > 0 since the fee requires minting new vrt
    cvlr_assert!(vrt_supply > vrt_supply_old);
}

#[rule]
// This rule is expected to be verified.
pub fn rule_mint_assess_fees() {
    let acc_infos: [AccountInfo; 16] = cvlr_deserialize_nondet_accounts();
    let used_acc_infos = &acc_infos[..10];
    let vault_info = &used_acc_infos[1];
    let vault_fee_token_account = &used_acc_infos[7];

    let fees_old: u64 = spl_token_account_get_amount(vault_fee_token_account);
    let vault_old: Vault = get_vault!(vault_info);
    let vrt_supply_old = vault_old.vrt_supply();

    // Assumption #1: there is some fee
    cvlr_assume!(vault_old.deposit_fee_bps() > 0);
    cvlr_assume!(vault_old.deposit_fee_bps() <= 10_000);

    let amount_token_in_arg: u64 = nondet();
    let min_amount_token_out_arg: u64 = nondet();

    process_mint(
        &crate::id(),
        &used_acc_infos,
        amount_token_in_arg,
        min_amount_token_out_arg,
    )
    .unwrap();

    let fees: u64 = spl_token_account_get_amount(vault_fee_token_account);
    let vault: Vault = get_vault!(vault_info);
    let vrt_supply = vault.vrt_supply();
    if vrt_supply > vrt_supply_old {
        cvlr_assert!(fees > fees_old); // some VRT fees have been collected
    }
}

#[rule]
/// "No dilution"
/// Expected to be verified
pub fn rule_update_vault_balance_no_dilution() {
    let acc_infos: [AccountInfo; 16] = cvlr_deserialize_nondet_accounts();
    let used_acc_infos = &acc_infos[..6];
    let vault_info = &used_acc_infos[1];
    let vault_token_account = &used_acc_infos[2];

    let vault_old: Vault = get_vault!(vault_info);
    let vrt_supply_old = vault_old.vrt_supply();
    let token_deposited_old = vault_old.tokens_deposited();

    cvlr_assume!(token_deposited_old > 0);
    cvlr_assume!(vrt_supply_old > 0);
    cvlr::cvlr_assume!(cvlr::is_u64(vrt_supply_old));
    cvlr::cvlr_assume!(cvlr::is_u64(token_deposited_old));

    let balance: u64 = spl_token_account_get_amount(vault_token_account);

    cvlr_assume!(vault_old.reward_fee_bps() <= ONE_IN_BPS);

    process_update_vault_balance(&crate::id(), &used_acc_infos).unwrap();

    let vault: Vault = get_vault!(vault_info);
    cvlr_assert!(!vault.is_paused());
    let vrt_supply = vault.vrt_supply();
    let token_deposited = vault.tokens_deposited();

    cvlr_assume!(balance >= token_deposited_old);

    cvlr_assert!(vrt_supply >= vrt_supply_old);
    let delta_vrt_supply = vrt_supply - vrt_supply_old;
    let delta_token = token_deposited - token_deposited_old;
    cvlr::cvlr_assume!(cvlr::is_u64(delta_token));

    clog!(
        delta_vrt_supply,
        delta_token,
        vrt_supply_old,
        token_deposited_old
    );

    // minted vrt cannot be greater than the deposited rewards (converted to vrt)
    // computation in u128 because it might overflow u64 even if there is no overflow in function under verification
    let max_vrt_increase = safe_mul_div_u128(delta_token, vrt_supply_old, token_deposited_old);
    cvlr_assert!(u128::from(delta_vrt_supply) <= max_vrt_increase);
}

#[rule]
/// "No dilution"
/// Expected to be verified
pub fn rule_mint_no_dilution() {
    let acc_infos: [AccountInfo; 16] = cvlr_deserialize_nondet_accounts();

    let used_acc_infos = &acc_infos[..10];
    let vault_info = &used_acc_infos[1];

    let vault_old: Vault = get_vault!(vault_info);
    let vrt_supply_old = vault_old.vrt_supply();
    let token_deposited_old = vault_old.tokens_deposited();

    // -- vault should always have some tokens deposited
    // -- enable this assumption for this rule to pass
    cvlr_assume!(token_deposited_old > 0);

    cvlr_assume!(vault_old.reward_fee_bps() <= 10_000);

    let amount_token_in_arg: u64 = nondet();
    let min_amount_token_out_arg: u64 = nondet();
    process_mint(
        &crate::id(),
        &used_acc_infos,
        amount_token_in_arg,
        min_amount_token_out_arg,
    )
    .unwrap();

    let vault: Vault = get_vault!(vault_info);
    let vrt_supply = vault.vrt_supply();
    let token_deposited = vault.tokens_deposited();

    cvlr_assert!(vrt_supply >= vrt_supply_old);
    let delta_vrt_supply = vrt_supply - vrt_supply_old;
    let delta_token = token_deposited - token_deposited_old;

    // minted vrt cannot be greater than amount_token_in_arg  (converted to vrt)
    cvlr_assert!(
        delta_vrt_supply <= safe_mul_div(delta_token, vrt_supply_old, token_deposited_old)
    );
}

const ONE_IN_BPS: u16 = 10_000;

#[rule]
/// Expected to be verified
pub fn rule_integrity_burn_withdrawal() {
    let acc_infos: [AccountInfo; 16] = cvlr_deserialize_nondet_accounts();

    let used_acc_infos = &acc_infos[..12];
    let vault_info = &used_acc_infos[1];
    let vault_token_account_info = &used_acc_infos[2];
    let vrt_mint_account_info = &used_acc_infos[3];

    let vault_old: Vault = get_vault!(vault_info);
    cvlr_assume!(cvt_vault_inv(
        &vault_old,
        vault_token_account_info.into(),
        vrt_mint_account_info.into()
    ));

    cvlr_assume!(vault_old.withdrawal_fee_bps() <= ONE_IN_BPS);

    process_burn_withdrawal_ticket(&crate::id(), &used_acc_infos).unwrap();

    let vault: Vault = get_vault!(vault_info);
    cvlr_assert!(cvt_vault_inv(
        &vault,
        vault_token_account_info.into(),
        vrt_mint_account_info.into()
    ));
}

#[rule]
/// Expected to be verified
pub fn rule_burn_withdrawal_no_dilution() {
    let acc_infos: [AccountInfo; 16] = cvlr_deserialize_nondet_accounts();

    /*
    [
     0   config,
     1   vault_info,
     2   vault_token_account,
     3   vrt_mint,
     4   staker,
     5   staker_token_account,
     6   vault_staker_withdrawal_ticket_info,
     7   vault_staker_withdrawal_ticket_token_account,
     8   vault_fee_token_account,
     9   program_fee_token_account,
     10   token_program,
     11   system_program]
     */

    let used_acc_infos = &acc_infos[..12];
    let vault_info = &used_acc_infos[1];
    let vault_token_account_info = &used_acc_infos[2];
    let vrt_mint_account_info = &used_acc_infos[3];
    let staker_info = &used_acc_infos[4];
    let staker_token_account_info = &used_acc_infos[5];
    let vault_staker_withdrawal_ticket_info = &used_acc_infos[6];
    let vault_stacker_withdrawal_ticket_token_account_info = &used_acc_infos[7];
    let vault_fee_token_account_info = &used_acc_infos[8];
    let program_fee_token_account_info = &used_acc_infos[9];

    let staker_token_account_balance_old = spl_token_account_get_amount(staker_token_account_info);
    let vault_fee_vrt_old = spl_token_account_get_amount(vault_fee_token_account_info);
    let program_fee_vrt_old = spl_token_account_get_amount(program_fee_token_account_info);

    let withdrawal_ticket_old =
        get_vault_staker_withdrawal_ticket!(vault_staker_withdrawal_ticket_info);

    let withdrawal_ticket_token_amount_old =
        spl_token_account_get_amount(vault_stacker_withdrawal_ticket_token_account_info);

    let vault_old: Vault = get_vault!(vault_info);
    cvlr_assume!(cvt_vault_inv(
        &vault_old,
        vault_token_account_info.into(),
        vrt_mint_account_info.into()
    ));

    cvlr_assume!(vault_old.withdrawal_fee_bps() <= ONE_IN_BPS);
    cvlr_assume!(vault_old.program_fee_bps() <= ONE_IN_BPS);

    // XXX SUM of fees should be <= ONE_IN_BPS

    let vrt_supply_old = vault_old.vrt_supply();
    let token_deposited_old = vault_old.tokens_deposited();

    process_burn_withdrawal_ticket(&crate::id(), &used_acc_infos).unwrap();

    let vault: Vault = get_vault!(vault_info);
    cvlr_assert!(!vault.is_paused());

    let vrt_supply = vault.vrt_supply();
    let token_deposited = vault.tokens_deposited();

    // -- token supply decreased
    cvlr_assert!(token_deposited_old >= token_deposited);
    // -- vrt supply decreased
    cvlr_assert!(vrt_supply_old >= vrt_supply);

    let delta_token = token_deposited_old - token_deposited;
    let delta_vrt_supply = vrt_supply_old - vrt_supply;

    // -- VRT decrease is limitted by ticket amount
    cvlr_assert!(delta_vrt_supply <= withdrawal_ticket_token_amount_old);

    // -- VRT:TKN ratio is preserved
    // cvlr_assert!(delta_token <= (delta_vrt_supply * token_deposited_old) / vrt_supply_old);
    cvlr_assert!(
        delta_token <= safe_mul_div(delta_vrt_supply, token_deposited_old, vrt_supply_old)
    );

    let program_fee_vrt = spl_token_account_get_amount(program_fee_token_account_info);
    cvlr_assert!(program_fee_vrt >= program_fee_vrt_old);

    // -- BUG: instruction should dissallow vault to be a staker
    cvlr_assume!(staker_token_account_info.key != vault_token_account_info.key);

    let withdrawal_ticket_token_amount =
        spl_token_account_get_amount(vault_stacker_withdrawal_ticket_token_account_info);
    cvlr_assert!(withdrawal_ticket_token_amount == 0);

    // -- staker does not get more tokens then send out
    let staker_token_account_balance = spl_token_account_get_amount(staker_token_account_info);
    let staker_token_delta = staker_token_account_balance - staker_token_account_balance_old;
    cvlr_assert!(staker_token_delta == delta_token);

    let vault_fee_vrt = spl_token_account_get_amount(vault_fee_token_account_info);
    let program_fee_vrt = spl_token_account_get_amount(program_fee_token_account_info);

    cvlr_assert!(vault_fee_vrt_old <= vault_fee_vrt);
    cvlr_assert!(program_fee_vrt_old <= program_fee_vrt);
    let delta_vault_fee_vrt = vault_fee_vrt - vault_fee_vrt_old;
    let delta_program_fee_vrt = program_fee_vrt - program_fee_vrt_old;

    clog!(
        delta_vrt_supply,
        delta_vault_fee_vrt,
        delta_program_fee_vrt,
        withdrawal_ticket_token_amount_old
    );

    cvlr_assert!(
        delta_vrt_supply + delta_vault_fee_vrt + delta_program_fee_vrt
            == withdrawal_ticket_token_amount_old
    );

    cvlr_assert!(&withdrawal_ticket_old.staker == staker_info.key);
}

#[rule]
/// Expected to be verified
pub fn rule_capacity_respected_by_mint_to() {
    let acc_infos: [AccountInfo; 16] = cvlr_deserialize_nondet_accounts();
    let used_acc_infos = &acc_infos[..10];
    let vault_info = &used_acc_infos[1];

    let vault_old: Vault = get_vault!(vault_info);

    let amount_token_in_arg: u64 = nondet();
    let min_amount_token_out_arg: u64 = nondet();

    process_mint(
        &crate::id(),
        &used_acc_infos,
        amount_token_in_arg,
        min_amount_token_out_arg,
    )
    .unwrap();

    let vault: Vault = get_vault!(vault_info);
    if vault.tokens_deposited() > vault_old.tokens_deposited() {
        // if TKN increased, there was capacity for new TKN
        cvlr_assert!(vault_old.tokens_deposited() < vault_old.deposit_capacity());
    }

    // if new VRT are minted, vault had capacity
    if vault_old.vrt_supply() < vault.vrt_supply() {
        cvlr_assert!(vault_old.tokens_deposited() < vault_old.deposit_capacity());
    }
}

#[rule]
/// This rule was written assuming that if capacity has been reached then
/// no VRT can be minted. However, after talking to customers the spec was different.
/// Therefore, this rule is expected to be a satisfy rule.
pub fn rule_capacity_respected_by_update_vault_balance_1() {
    let acc_infos: [AccountInfo; 16] = cvlr_deserialize_nondet_accounts();
    let used_acc_infos = &acc_infos[..6];
    let vault_info = &used_acc_infos[1];

    // let minted_tokens_old = spl_mint_get_supply(vrt_mint_account);
    let vault_old: Vault = get_vault!(vault_info);
    process_update_vault_balance(&crate::id(), &used_acc_infos).unwrap();

    let vault: Vault = get_vault!(vault_info);
    if vault.tokens_deposited() > vault_old.tokens_deposited() {
        // if TKN increased, there was capacity for new TKN
        cvlr_satisfy!(vault_old.tokens_deposited() < vault_old.deposit_capacity());
    }
}

#[rule]
/// This rule was written assuming that if capacity has been reached then
/// no VRT can be minted. However, after talking to customers the spec was different.
/// Therefore, this rule is expected to be a satisfy rule.
pub fn rule_capacity_respected_by_update_vault_balance_2() {
    let acc_infos: [AccountInfo; 16] = cvlr_deserialize_nondet_accounts();
    let used_acc_infos = &acc_infos[..6];
    let vault_info = &used_acc_infos[1];

    // let minted_tokens_old = spl_mint_get_supply(vrt_mint_account);
    let vault_old: Vault = get_vault!(vault_info);
    process_update_vault_balance(&crate::id(), &used_acc_infos).unwrap();

    let vault: Vault = get_vault!(vault_info);

    // if new VRT are minted, vault had capacity
    if vault_old.vrt_supply() < vault.vrt_supply() {
        cvlr_satisfy!(vault_old.tokens_deposited() < vault_old.deposit_capacity());
    }
}

#[rule]
/// Expected to be verified
pub fn rule_integrity_mint_2() {
    let acc_infos: [AccountInfo; 16] = cvlr_deserialize_nondet_accounts();
    let used_acc_infos = &acc_infos[..10];
    let vault_info = &used_acc_infos[1];
    let vrt_mint_account_info = &used_acc_infos[2];
    let depositor_token_account_info = &used_acc_infos[4];
    let vault_token_account_info = &used_acc_infos[5];

    // This assumption is important, otherwise the vault_token_account is not updated if the
    // depositor is the same.
    cvlr_assume!(depositor_token_account_info.key != vault_token_account_info.key);

    let vault_old: Vault = get_vault!(vault_info);
    cvlr_assume!(cvt_vault_inv(
        &vault_old,
        vault_token_account_info.into(),
        vrt_mint_account_info.into()
    ));

    cvlr_assume!(vault_old.withdrawal_fee_bps() <= ONE_IN_BPS);

    let amount_in: u64 = nondet();
    let min_amount_out: u64 = nondet();

    process_mint(&crate::id(), &used_acc_infos, amount_in, min_amount_out).unwrap();

    let vault: Vault = get_vault!(vault_info);
    cvlr_assert!(!vault.is_paused());
    cvlr_assert!(cvt_vault_inv(
        &vault,
        vault_token_account_info.into(),
        vrt_mint_account_info.into()
    ));
}

fn cvt_assert_delegation_accumulated(
    dst: &DelegationState,
    src1: &DelegationState,
    src2: &DelegationState,
) {
    cvlr_assert!(dst.staked_amount() == src1.staked_amount() + src2.staked_amount());
    cvlr_assert!(
        dst.cooling_down_amount() == src1.cooling_down_amount() + src2.cooling_down_amount()
    );
    cvlr_assert!(
        dst.enqueued_for_cooldown_amount()
            == src1.enqueued_for_cooldown_amount() + src2.enqueued_for_cooldown_amount()
    );
}

#[rule]
/// Expected to be verified
pub fn rule_crank_vault_update_state_tracker_integrity() {
    // initialize mock clock
    let now_slot = Clock::get().unwrap().slot;

    let acc_infos: [AccountInfo; 16] = cvlr_deserialize_nondet_accounts();
    let used_acc_infos = &acc_infos[..5];

    let config = &used_acc_infos[0];
    let vault_info = &used_acc_infos[1];
    let _operator = &used_acc_infos[2];
    let vault_operator_delegation_info = &used_acc_infos[3];
    let vault_update_state_tracker_info = &used_acc_infos[4];

    let config_old: jito_vault_core::config::Config = get_vault_config!(config);
    let vault_old: Vault = get_vault!(vault_info);

    // -- assume fixed epoch length
    cvlr_assume!(config_old.epoch_length() == solana_program::clock::DEFAULT_SLOTS_PER_EPOCH);

    let epoch_length = config_old.epoch_length();
    let now_epoch = get_epoch(now_slot, epoch_length).unwrap();
    let vault_last_update_epoch =
        get_epoch(vault_old.last_full_state_update_slot(), epoch_length).unwrap();
    // -- last update was in some previous epoch
    cvlr_assume!(vault_last_update_epoch < now_epoch);

    let vault_operator_delegation_old =
        get_vault_operator_delegation!(vault_operator_delegation_info);

    let operator_last_update_epoch = get_epoch(
        vault_operator_delegation_old.last_update_slot(),
        epoch_length,
    )
    .unwrap();

    // -- operator might have been updated after the vault, but not before
    cvlr_assume!(vault_last_update_epoch <= operator_last_update_epoch);

    let vault_update_tracker_old = get_vault_update_state_tracker!(vault_update_state_tracker_info);

    crate::process_crank_vault_update_state_tracker(&crate::id(), &used_acc_infos).unwrap();

    // -- vault is not paused
    cvlr_assert!(!vault_old.is_paused());

    // -- on success, operator has not yet been updated in this epoch
    cvlr_assert!(operator_last_update_epoch < now_epoch);

    let vault_operator_delegation = get_vault_operator_delegation!(vault_operator_delegation_info);
    // -- on success, it is recorded that the operator has been updated now
    cvlr_assert!(vault_operator_delegation.last_update_slot() == now_slot);

    let vault_update_tracker = get_vault_update_state_tracker!(vault_update_state_tracker_info);

    // -- on success, greedy allocation method was used
    let allocation_method =
        WithdrawalAllocationMethod::try_from(vault_update_tracker.withdrawal_allocation_method);
    match allocation_method {
        Ok(WithdrawalAllocationMethod::Greedy) => {}
        _ => {
            cvlr_assert!(false);
        }
    }

    // -- check that operator delegation is updated properly given the number of
    // -- epochs that have passed since the last update
    let operator_delegation_state_old = &vault_operator_delegation_old.delegation_state;
    let operator_delegation_state = &vault_operator_delegation.delegation_state;

    // -- maximum amount that was virtually enqueued in the previous epoch by
    // -- creation of the withdrawal ticket. This should be treated as though it was
    // -- set to cooldown in the last epoch the operator was updated
    // -- this is also maximum amount that might be unstaked in this operation
    let virtual_enqued_for_cooldown = if operator_last_update_epoch <= vault_last_update_epoch {
        std::cmp::min(
            vault_old.additional_assets_need_unstaking(),
            operator_delegation_state_old.staked_amount(),
        )
    } else {
        0
    };

    // -- nothing is cooling down because it was enqued in the previous epoch
    cvlr_assert!(operator_delegation_state.enqueued_for_cooldown_amount() == 0);

    // -- staked amount cannot increase, no matter what
    cvlr_assert!(
        operator_delegation_state.staked_amount() <= operator_delegation_state_old.staked_amount()
    );

    // -- whatever is virtually cooled after an update must come from staked amount
    cvlr_assert!(
        operator_delegation_state.staked_amount()
            == operator_delegation_state_old.staked_amount() - virtual_enqued_for_cooldown
    );

    // -- cooling down amount is as expected
    let operator_delta_epoch = now_epoch - operator_last_update_epoch;
    match operator_delta_epoch {
        0 => {
            cvlr_assert!(false);
        }
        1 => {
            cvlr_assert!(
                operator_delegation_state.cooling_down_amount()
                    == operator_delegation_state_old.enqueued_for_cooldown_amount()
                        + virtual_enqued_for_cooldown
            );
        }
        _ => {
            cvlr_assert!(operator_delegation_state.cooling_down_amount() == 0);
        }
    }

    // -- need unstaking decreased as expected
    let vault = get_vault!(vault_info);
    cvlr_assert!(
        vault.additional_assets_need_unstaking()
            == vault_old.additional_assets_need_unstaking() - virtual_enqued_for_cooldown
    );

    // -- vault accumulated delegation state
    cvt_assert_delegation_accumulated(
        &vault_update_tracker.delegation_state,
        &operator_delegation_state,
        &vault_update_tracker_old.delegation_state,
    );
}
