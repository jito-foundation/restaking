use jito_account_traits::AccountDeserialize;
use jito_jsm_core::loader::{
    load_associated_token_account, load_signer, load_system_program, load_token_mint,
    load_token_program,
};
use jito_vault_core::{
    config::Config,
    loader::{
        load_config, load_vault, load_vault_delegation_list, load_vault_staker_withdrawal_ticket,
    },
    vault::Vault,
    vault_delegation_list::VaultDelegationList,
    vault_staker_withdrawal_ticket::VaultStakerWithdrawalTicket,
};
use jito_vault_sdk::error::VaultError;
use solana_program::{
    account_info::AccountInfo, clock::Clock, entrypoint::ProgramResult, msg,
    program_error::ProgramError, pubkey::Pubkey, sysvar::Sysvar,
};
use std::cmp::min;

/// Burns the withdrawal ticket, transferring the assets to the staker and closing the withdrawal ticket.
///
/// One should call the [`crate::VaultInstruction::UpdateVault`] instruction before running this instruction
/// to ensure that any rewards that were accrued are accounted for.
pub fn process_burn_withdrawal_ticket(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    fail_if_not_fair_price: bool,
) -> ProgramResult {
    let [config, vault_info, vault_delegation_list, vault_token_account, vrt_mint, staker, staker_token_account, staker_vrt_token_account, vault_staker_withdrawal_ticket_info, vault_staker_withdrawal_ticket_token_account, token_program, system_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    load_config(program_id, config, false)?;
    load_vault(program_id, vault_info, true)?;
    load_vault_delegation_list(program_id, vault_delegation_list, vault_info, true)?;
    let mut vault_data = vault_info.data.borrow_mut();
    let vault = Vault::try_from_slice_mut(&mut vault_data)?;
    load_associated_token_account(vault_token_account, vault_info.key, &vault.supported_mint)?;
    load_token_mint(vrt_mint)?;
    load_signer(staker, false)?;
    load_associated_token_account(staker_token_account, staker.key, &vault.supported_mint)?;
    load_associated_token_account(staker_vrt_token_account, staker.key, &vault.vrt_mint)?;
    load_vault_staker_withdrawal_ticket(
        program_id,
        vault_staker_withdrawal_ticket_info,
        vault_info,
        staker,
        true,
    )?;
    load_associated_token_account(
        vault_staker_withdrawal_ticket_token_account,
        vault_staker_withdrawal_ticket_info.key,
        &vault.vrt_mint,
    )?;
    load_token_program(token_program)?;
    load_system_program(system_program)?;

    // The VRT mint provided as an account shall be the vault LRT mint
    if vault.vrt_mint.ne(vrt_mint.key) {
        msg!("Vault VRT mint mismatch");
        return Err(ProgramError::InvalidArgument);
    }

    let slot = Clock::get()?.slot;

    // The vault delegation list shall be up-to-date
    let config_data = config.data.borrow();
    let config = Config::try_from_slice(&config_data)?;
    let mut vault_delegation_list_data = vault_delegation_list.data.borrow_mut();
    let vault_delegation_list =
        VaultDelegationList::try_from_slice_mut(&mut vault_delegation_list_data)?;
    if vault_delegation_list.is_update_needed(slot, config.epoch_length) {
        msg!("Vault delegation list needs to be updated");
        return Err(VaultError::VaultDelegationListUpdateNeeded.into());
    }

    // The vault withdrawal ticket shall be withdrawable after one full epoch of cooling down
    let vault_staker_withdrawal_ticket_data = vault_staker_withdrawal_ticket_info.data.borrow();
    let vault_staker_withdrawal_ticket =
        VaultStakerWithdrawalTicket::try_from_slice(&vault_staker_withdrawal_ticket_data)?;
    if !vault_staker_withdrawal_ticket.is_withdrawable(slot, config.epoch_length)? {
        msg!("Vault staker withdrawal ticket is not withdrawable");
        return Err(VaultError::VaultStakerWithdrawalTicketNotWithdrawable.into());
    }

    // The total amount available to withdraw shall be equal to the total amount of tokens deposited
    // minus the total amount of delegated security, which includes staked and cooling down assets.
    let assets_delegated = vault_delegation_list.total_security()?;
    let assets_withdrawable = vault
        .tokens_deposited
        .checked_sub(assets_delegated)
        .ok_or(VaultError::VaultDelegationListUnderflow)?;

    let fair_price_redemption_amount =
        vault.calculate_assets_returned_amount(vault_staker_withdrawal_ticket.vrt_amount)?;

    // if the staker wants a fair price and the redemption amount exceeds the assets available to withdraw
    // the program shall return an error
    if fail_if_not_fair_price && fair_price_redemption_amount > assets_withdrawable {
        msg!("Redemption amount exceeds available assets. Check back later for a fair price.");
        return Err(VaultError::VaultInsufficientFunds.into());
    }

    let redemption_amount = min(assets_withdrawable, fair_price_redemption_amount);
    let

    // // burn the assets + close the token account + withdraw token account
    // {
    //     let (_, vault_staker_withdraw_bump, mut vault_staker_withdraw_seeds) =
    //         VaultStakerWithdrawalTicket::find_program_address(
    //             program_id,
    //             vault_info.key,
    //             staker.key,
    //             &vault_staker_withdrawal_ticket.base,
    //         );
    //     vault_staker_withdraw_seeds.push(vec![vault_staker_withdraw_bump]);
    //     let seed_slices: Vec<&[u8]> = vault_staker_withdraw_seeds
    //         .iter()
    //         .map(|seed| seed.as_slice())
    //         .collect();
    //
    //     drop(vault_staker_withdrawal_ticket_data);
    //
    //     // burn the VRT tokens
    //     invoke_signed(
    //         &burn(
    //             &spl_token::id(),
    //             vault_staker_withdrawal_ticket_token_account.key,
    //             vrt_mint.key,
    //             vault_staker_withdrawal_ticket_info.key,
    //             &[],
    //             vrt_to_burn,
    //         )?,
    //         &[
    //             vault_staker_withdrawal_ticket_token_account.clone(),
    //             vrt_mint.clone(),
    //             vault_staker_withdrawal_ticket_info.clone(),
    //         ],
    //         &[&seed_slices],
    //     )?;
    //
    //     // if there are any excess, transfer them to the staker
    //     let vrt_token_account =
    //         Account::unpack(&vault_staker_withdrawal_ticket_token_account.data.borrow())?;
    //     if vrt_token_account.amount > 0 {
    //         invoke_signed(
    //             &transfer(
    //                 &spl_token::id(),
    //                 vault_staker_withdrawal_ticket_token_account.key,
    //                 staker_vrt_token_account.key,
    //                 vault_staker_withdrawal_ticket_info.key,
    //                 &[],
    //                 vrt_token_account.amount,
    //             )?,
    //             &[
    //                 vault_staker_withdrawal_ticket_token_account.clone(),
    //                 staker_vrt_token_account.clone(),
    //                 vault_staker_withdrawal_ticket_info.clone(),
    //             ],
    //             &[&seed_slices],
    //         )?;
    //     }
    //
    //     // close token account
    //     invoke_signed(
    //         &close_account(
    //             &spl_token::id(),
    //             vault_staker_withdrawal_ticket_token_account.key,
    //             staker.key,
    //             vault_staker_withdrawal_ticket_info.key,
    //             &[],
    //         )?,
    //         &[
    //             vault_staker_withdrawal_ticket_token_account.clone(),
    //             staker.clone(),
    //             vault_staker_withdrawal_ticket_info.clone(),
    //         ],
    //         &[&seed_slices],
    //     )?;
    //
    //     close_program_account(program_id, vault_staker_withdrawal_ticket_info, staker)?;
    // }
    //
    // vault.vrt_supply = vault
    //     .vrt_supply
    //     .checked_sub(vrt_to_burn)
    //     .ok_or(ProgramError::ArithmeticOverflow)?;
    // // TODO (LB): https://github.com/jito-foundation/restaking/issues/24
    // //  If a withdraw ticket is created and there is a slashing event before the withdraw ticket
    // //  has fully matured, the program can end up in a situation where the original_redemption_amount
    // //  is greater than the total withdrawable_reserve_amount. This is a bug and needs to be fixed.
    // //  see test_burn_withdrawal_ticket_with_slashing_before_update
    // msg!(
    //     "vault.withdrawable_reserve_amount before: {:?}",
    //     vault.withdrawable_vrt_reserve_amount
    // );
    //
    // vault.withdrawable_vrt_reserve_amount = vault
    //     .withdrawable_vrt_reserve_amount
    //     .checked_sub(original_redemption_amount)
    //     .ok_or(ProgramError::ArithmeticOverflow)?;
    //
    // msg!(
    //     "vault.withdrawable_reserve_amount after: {:?}",
    //     vault.withdrawable_vrt_reserve_amount
    // );

    // vault.tokens_deposited = vault
    //     .tokens_deposited
    //     .checked_sub(actual_withdraw_amount)
    //     .ok_or(ProgramError::InsufficientFunds)?;

    // // transfer the assets to the staker
    // let (_, vault_bump, mut vault_seeds) = Vault::find_program_address(program_id, &vault.base);
    // vault_seeds.push(vec![vault_bump]);
    // let seed_slices: Vec<&[u8]> = vault_seeds.iter().map(|seed| seed.as_slice()).collect();
    // drop(vault_data); // avoid double borrow
    // invoke_signed(
    //     &transfer(
    //         &spl_token::id(),
    //         vault_token_account.key,
    //         staker_token_account.key,
    //         vault_info.key,
    //         &[],
    //         actual_withdraw_amount,
    //     )?,
    //     &[
    //         vault_token_account.clone(),
    //         staker_token_account.clone(),
    //         vault_info.clone(),
    //     ],
    //     &[&seed_slices],
    // )?;

    vault_delegation_list.withdrawable_vrt_reserve_amount = vault_delegation_list
        .withdrawable_vrt_reserve_amount
        .checked_sub(vault_staker_withdrawal_ticket.vrt_amount)
        .ok_or(ProgramError::ArithmeticOverflow)?;

    Ok(())
}
