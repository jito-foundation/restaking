use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, pubkey::Pubkey};

/// Burns the withdrawal ticket, transferring the assets to the staker and closing the withdrawal ticket.
///
/// One should call the [`crate::VaultInstruction::UpdateVault`] instruction before running this instruction
/// to ensure that any rewards that were accrued are accounted for.
pub const fn process_burn_withdrawal_ticket(
    _program_id: &Pubkey,
    _accounts: &[AccountInfo],
) -> ProgramResult {
    // let SanitizedAccounts {
    //     config,
    //     mut vault,
    //     vault_delegation_list,
    //     mut vault_token_account,
    //     mut lrt_mint,
    //     staker,
    //     mut staker_token_account,
    //     staker_lrt_token_account,
    //     vault_staker_withdrawal_ticket,
    //     mut vault_staker_withdrawal_ticket_token_account,
    // } = SanitizedAccounts::sanitize(program_id, accounts)?;
    //
    // let slot = Clock::get()?.slot;
    // let epoch_length = config.epoch_length;
    //
    // vault_delegation_list
    //     .vault_delegation_list()
    //     .check_update_needed(slot, epoch_length)?;
    //
    // assert_with_msg(
    //     vault.vault.lrt_mint == *lrt_mint.account().key,
    //     ProgramError::InvalidArgument,
    //     "LRT mint mismatch",
    // )?;
    //
    // vault_staker_withdrawal_ticket
    //     .vault_staker_withdrawal_ticket()
    //     .check_withdrawable(slot, epoch_length)?;
    //
    // // find the current redemption amount and the original redemption amount in the withdraw ticket
    // let redemption_amount = vault.vault.calculate_assets_returned_amount(
    //     vault_staker_withdrawal_ticket
    //         .vault_staker_withdrawal_ticket()
    //         .lrt_amount,
    // )?;
    //
    // let original_redemption_amount = vault_staker_withdrawal_ticket
    //     .vault_staker_withdrawal_ticket()
    //     .withdraw_allocation_amount;
    //
    // let actual_withdraw_amount = if redemption_amount > original_redemption_amount {
    //     // The program can guarantee the original redemption amount, but if the redemption amount
    //     // is greater than the original amount, there were rewards that accrued
    //     // to the LRT.
    //     // The program attempts to figure out how much more of the asset can be unstaked to fulfill
    //     // as much of the redemption amount as possible.
    //     // Available unstaked assets is equal to:
    //     // the amount of tokens deposited - any delegated security - the amount reserved for withdraw tickets
    //     let tokens_deposited_in_vault = vault.vault.tokens_deposited;
    //     let delegated_security_in_vault = vault_delegation_list
    //         .vault_delegation_list()
    //         .total_security()?;
    //     let assets_reserved_for_withdrawal_tickets = vault.vault.withdrawable_reserve_amount();
    //
    //     let available_unstaked_assets = tokens_deposited_in_vault
    //         .checked_sub(delegated_security_in_vault)
    //         .and_then(|x| x.checked_sub(assets_reserved_for_withdrawal_tickets))
    //         .ok_or(ProgramError::InsufficientFunds)?;
    //
    //     // Calculate the extra amount that can be withdrawn
    //     let extra_amount = redemption_amount
    //         .checked_sub(original_redemption_amount)
    //         .ok_or(ProgramError::ArithmeticOverflow)?;
    //
    //     // Determine the actual amount to withdraw
    //     original_redemption_amount
    //         .checked_add(extra_amount.min(available_unstaked_assets))
    //         .ok_or(ProgramError::ArithmeticOverflow)?
    // } else {
    //     redemption_amount
    // };
    //
    // let lrt_to_burn = vault
    //     .vault
    //     .calculate_lrt_mint_amount(actual_withdraw_amount)?;
    // let lrt_amount_to_burn = std::cmp::min(
    //     lrt_to_burn,
    //     vault_staker_withdrawal_ticket
    //         .vault_staker_withdrawal_ticket()
    //         .lrt_amount,
    // );
    //
    // _burn_lrt(
    //     program_id,
    //     &vault,
    //     &staker,
    //     &vault_staker_withdrawal_ticket,
    //     &vault_staker_withdrawal_ticket_token_account,
    //     &lrt_mint,
    //     lrt_amount_to_burn,
    // )?;
    // lrt_mint.reload()?;
    // vault_staker_withdrawal_ticket_token_account.reload()?;
    //
    // _transfer_vault_tokens_to_staker(
    //     program_id,
    //     &vault,
    //     &vault_token_account,
    //     &staker_token_account,
    //     actual_withdraw_amount,
    // )?;
    // vault_token_account.reload()?;
    // staker_token_account.reload()?;
    //
    // msg!(
    //     "decrementing reserve amount: {:?}, amount available: {:?}",
    //     original_redemption_amount,
    //     vault.vault.withdrawable_reserve_amount()
    // );
    //
    // // TODO (LB): https://github.com/jito-foundation/restaking/issues/24
    // //  If a withdraw ticket is created and there is a slashing event before the withdraw ticket
    // //  has fully matured, the program can end up in a situation where the original_redemption_amount
    // //  is greater than the total withdrawable_reserve_amount. This is a bug and needs to be fixed.
    // //  see test_burn_withdrawal_ticket_with_slashing_before_update
    // vault
    //     .vault_mut()
    //     .decrement_withdrawable_reserve_amount(original_redemption_amount)?;
    //
    // // refresh after burn
    // vault
    //     .vault_mut()
    //     .set_tokens_deposited(vault_token_account.token_account().amount);
    // vault.vault_mut().set_lrt_supply(lrt_mint.mint().supply);
    //
    // _close_token_account(
    //     program_id,
    //     &vault,
    //     &staker,
    //     &vault_staker_withdrawal_ticket,
    //     &vault_staker_withdrawal_ticket_token_account,
    //     &staker_lrt_token_account,
    // )?;
    //
    // close_program_account(
    //     program_id,
    //     vault_staker_withdrawal_ticket.account(),
    //     staker.account(),
    // )?;
    //
    // vault.save()?;
    // vault_delegation_list.save()?;

    Ok(())
}
//
// /// transfers all remaining assets to the staker + closes the account
// fn _close_token_account<'a, 'info>(
//     program_id: &Pubkey,
//     vault: &SanitizedVault<'a, 'info>,
//     staker: &SanitizedSignerAccount<'a, 'info>,
//     vault_staker_withdrawal_ticket: &SanitizedVaultStakerWithdrawalTicket<'a, 'info>,
//     vault_staker_withdrawal_ticket_token_account: &SanitizedAssociatedTokenAccount<'a, 'info>,
//     staker_lrt_token_account: &SanitizedAssociatedTokenAccount<'a, 'info>,
// ) -> ProgramResult {
//     let (_, bump, mut seeds) = VaultStakerWithdrawalTicket::find_program_address(
//         program_id,
//         vault.account().key,
//         staker.account().key,
//         &vault_staker_withdrawal_ticket
//             .vault_staker_withdrawal_ticket()
//             .base(),
//     );
//     seeds.push(vec![bump]);
//     let seed_slices: Vec<&[u8]> = seeds.iter().map(|seed| seed.as_slice()).collect();
//
//     if vault_staker_withdrawal_ticket_token_account
//         .token_account()
//         .amount
//         > 0
//     {
//         invoke_signed(
//             &transfer(
//                 &spl_token::id(),
//                 vault_staker_withdrawal_ticket_token_account.account().key,
//                 staker_lrt_token_account.account().key,
//                 vault_staker_withdrawal_ticket.account().key,
//                 &[],
//                 vault_staker_withdrawal_ticket_token_account
//                     .token_account()
//                     .amount,
//             )?,
//             &[
//                 vault_staker_withdrawal_ticket_token_account
//                     .account()
//                     .clone(),
//                 staker_lrt_token_account.account().clone(),
//                 vault_staker_withdrawal_ticket.account().clone(),
//             ],
//             &[&seed_slices],
//         )?;
//     }
//
//     invoke_signed(
//         &close_account(
//             &spl_token::id(),
//             vault_staker_withdrawal_ticket_token_account.account().key,
//             staker.account().key,
//             vault_staker_withdrawal_ticket.account().key,
//             &[],
//         )?,
//         &[
//             vault_staker_withdrawal_ticket_token_account
//                 .account()
//                 .clone(),
//             staker.account().clone(),
//             vault_staker_withdrawal_ticket.account().clone(),
//         ],
//         &[&seed_slices],
//     )?;
//     Ok(())
// }
//
// fn _transfer_vault_tokens_to_staker<'a, 'info>(
//     program_id: &Pubkey,
//     vault: &SanitizedVault<'a, 'info>,
//     vault_token_account: &SanitizedAssociatedTokenAccount<'a, 'info>,
//     staker_token_account: &SanitizedAssociatedTokenAccount<'a, 'info>,
//     amount: u64,
// ) -> ProgramResult {
//     let (_, bump, mut seeds) = Vault::find_program_address(program_id, &vault.vault.base());
//     seeds.push(vec![bump]);
//     let seed_slices: Vec<&[u8]> = seeds.iter().map(|seed| seed.as_slice()).collect();
//
//     invoke_signed(
//         &transfer(
//             &spl_token::id(),
//             vault_token_account.account().key,
//             staker_token_account.account().key,
//             vault.account().key,
//             &[],
//             amount,
//         )?,
//         &[
//             vault_token_account.account().clone(),
//             staker_token_account.account().clone(),
//             vault.account().clone(),
//         ],
//         &[&seed_slices],
//     )?;
//     Ok(())
// }
//
// fn _burn_lrt<'a, 'info>(
//     program_id: &Pubkey,
//     vault: &SanitizedVault<'a, 'info>,
//     staker: &SanitizedSignerAccount<'a, 'info>,
//     vault_staker_withdrawal_ticket: &SanitizedVaultStakerWithdrawalTicket<'a, 'info>,
//     vault_staker_withdrawal_ticket_token_account: &SanitizedAssociatedTokenAccount<'a, 'info>,
//     token_mint: &SanitizedTokenMint<'a, 'info>,
//     burn_amount: u64,
// ) -> ProgramResult {
//     let (_, bump, mut seeds) = VaultStakerWithdrawalTicket::find_program_address(
//         program_id,
//         vault.account().key,
//         staker.account().key,
//         &vault_staker_withdrawal_ticket
//             .vault_staker_withdrawal_ticket()
//             .base(),
//     );
//     seeds.push(vec![bump]);
//     let seed_slices: Vec<&[u8]> = seeds.iter().map(|seed| seed.as_slice()).collect();
//
//     invoke_signed(
//         &burn(
//             &spl_token::id(),
//             vault_staker_withdrawal_ticket_token_account.account().key,
//             token_mint.account().key,
//             vault_staker_withdrawal_ticket.account().key,
//             &[],
//             burn_amount,
//         )?,
//         &[
//             vault_staker_withdrawal_ticket_token_account
//                 .account()
//                 .clone(),
//             token_mint.account().clone(),
//             vault_staker_withdrawal_ticket.account().clone(),
//         ],
//         &[&seed_slices],
//     )
// }
