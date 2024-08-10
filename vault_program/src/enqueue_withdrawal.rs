use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, pubkey::Pubkey};

/// Enqueues a withdraw into the VaultStakerWithdrawalTicket account, transferring the amount from the
/// staker's LRT token account to the VaultStakerWithdrawalTicket LRT token account. It also queues
/// the withdrawal in the vault's delegation list.
///
/// The most obvious options for withdrawing are calculating the redemption ratio and withdrawing
/// the exact amount of collateral from operators. This may not be ideal in the case where the LRT:token
/// ratio increases due to rewards. However, if the vault has excess collateral that isn't staked, the vault
/// can withdraw that excess and return it to the staker. If there's no excess, they can withdraw the
/// amount that was set aside for withdraw.
///
/// One should call the [`crate::VaultInstruction::UpdateVault`] instruction before running this instruction
/// to ensure that any rewards that were accrued are accounted for.
pub const fn process_enqueue_withdrawal(
    _program_id: &Pubkey,
    _accounts: &[AccountInfo],
    _lrt_amount: u64,
) -> ProgramResult {
    // let SanitizedAccounts {
    //     config,
    //     vault,
    //     mut vault_delegation_list,
    //     vault_staker_withdrawal_ticket,
    //     vault_staker_withdrawal_ticket_token_account,
    //     vault_fee_token_account,
    //     staker,
    //     staker_lrt_token_account,
    //     base,
    //     system_program,
    //     burn_signer,
    // } = SanitizedAccounts::sanitize(program_id, accounts)?;
    //
    // // If a mint_signer is set, the signer shall be authorized by the vault to make deposits
    // if let Some(burn_signer) = burn_signer {
    //     assert_with_msg(
    //         *burn_signer.account().key == vault.vault.mint_burn_authority().unwrap(),
    //         ProgramError::InvalidAccountData,
    //         "Burn signer does not match vault mint-burn authority",
    //     )?;
    // }
    //
    // let slot = Clock::get()?.slot;
    // let epoch_length = config.epoch_length;
    // let rent = Rent::get()?;
    //
    // vault_delegation_list
    //     .vault_delegation_list_mut()
    //     .check_update_needed(slot, epoch_length)?;
    //
    // // The withdraw fee is subtracted here as opposed to when the withdraw ticket is processed
    // // so the amount representing the fee isn't unstaked.
    // let fee_amount = vault.vault.calculate_withdraw_fee(lrt_amount)?;
    // let amount_to_vault_staker_withdrawal_ticket = lrt_amount
    //     .checked_sub(fee_amount)
    //     .ok_or(ProgramError::ArithmeticOverflow)?;
    //
    // // Find the redemption ratio at this point in time.
    // // It may change in between this point in time and when the withdraw ticket is processed.
    // // Stakers may get back less than redemption if there were accrued rewards accrued in between
    // // this point and the redemption.
    // let amount_to_withdraw = vault
    //     .vault
    //     .calculate_assets_returned_amount(amount_to_vault_staker_withdrawal_ticket)?;
    // msg!(
    //     "lrt_supply: {} lrt_amount: {}, amount_to_withdraw: {}",
    //     vault.vault.lrt_supply(),
    //     amount_to_vault_staker_withdrawal_ticket,
    //     amount_to_withdraw
    // );
    //
    // vault_delegation_list
    //     .vault_delegation_list_mut()
    //     .undelegate_for_withdrawal(amount_to_withdraw, UndelegateForWithdrawMethod::ProRata)?;
    //
    // _create_vault_staker_withdrawal_ticket(
    //     program_id,
    //     &vault,
    //     &staker,
    //     &base,
    //     &vault_staker_withdrawal_ticket,
    //     &system_program,
    //     &rent,
    //     slot,
    //     amount_to_withdraw,
    //     amount_to_vault_staker_withdrawal_ticket,
    // )?;
    //
    // // Transfers the LRT tokens from the staker to their withdrawal account and the vault's fee account
    // _transfer_to(
    //     &staker_lrt_token_account,
    //     &vault_staker_withdrawal_ticket_token_account,
    //     &staker,
    //     amount_to_vault_staker_withdrawal_ticket,
    // )?;
    // _transfer_to(
    //     &staker_lrt_token_account,
    //     &vault_fee_token_account,
    //     &staker,
    //     fee_amount,
    // )?;
    //
    // vault_delegation_list.save()?;

    Ok(())
}
//
// fn _transfer_to<'a, 'info>(
//     from: &SanitizedTokenAccount<'a, 'info>,
//     to: &SanitizedAssociatedTokenAccount<'a, 'info>,
//     staker: &SanitizedSignerAccount<'a, 'info>,
//     amount: u64,
// ) -> ProgramResult {
//     invoke(
//         &transfer(
//             &spl_token::id(),
//             from.account().key,
//             to.account().key,
//             staker.account().key,
//             &[],
//             amount,
//         )?,
//         &[
//             from.account().clone(),
//             to.account().clone(),
//             staker.account().clone(),
//         ],
//     )
// }
//
// #[allow(clippy::too_many_arguments)]
// fn _create_vault_staker_withdrawal_ticket<'a, 'info>(
//     program_id: &Pubkey,
//     vault: &SanitizedVault<'a, 'info>,
//     staker: &SanitizedSignerAccount<'a, 'info>,
//     base: &SanitizedSignerAccount<'a, 'info>,
//     vault_staker_withdrawal_ticket_account: &EmptyAccount<'a, 'info>,
//     system_program: &SanitizedSystemProgram<'a, 'info>,
//     rent: &Rent,
//     slot: Slot,
//     amount_to_withdraw: u64,
//     amount_to_vault_staker_withdrawal_ticket: u64,
// ) -> ProgramResult {
//     let (address, bump, mut seeds) = VaultStakerWithdrawalTicket::find_program_address(
//         program_id,
//         vault.account().key,
//         staker.account().key,
//         base.account().key,
//     );
//     seeds.push(vec![bump]);
//
//     assert_with_msg(
//         address == *vault_staker_withdrawal_ticket_account.account().key,
//         ProgramError::InvalidAccountData,
//         "Vault staker withdraw ticket is not at the correct PDA",
//     )?;
//
//     let vault_staker_withdrawal_ticket = VaultStakerWithdrawalTicket::new(
//         *vault.account().key,
//         *staker.account().key,
//         *base.account().key,
//         amount_to_withdraw,
//         amount_to_vault_staker_withdrawal_ticket,
//         slot,
//         bump,
//     );
//
//     msg!(
//         "Creating vault staker withdraw ticket: {:?}",
//         vault_staker_withdrawal_ticket_account.account().key
//     );
//     let serialized = vault_staker_withdrawal_ticket.try_to_vec()?;
//     create_account(
//         staker.account(),
//         vault_staker_withdrawal_ticket_account.account(),
//         system_program.account(),
//         program_id,
//         rent,
//         serialized.len() as u64,
//         &seeds,
//     )?;
//     vault_staker_withdrawal_ticket_account
//         .account()
//         .data
//         .borrow_mut()[..serialized.len()]
//         .copy_from_slice(&serialized);
//     Ok(())
// }
//
// struct SanitizedAccounts<'a, 'info> {
//     config: SanitizedConfig<'a, 'info>,
//     vault: SanitizedVault<'a, 'info>,
//     vault_delegation_list: SanitizedVaultDelegationList<'a, 'info>,
//     vault_staker_withdrawal_ticket: EmptyAccount<'a, 'info>,
//     vault_staker_withdrawal_ticket_token_account: SanitizedAssociatedTokenAccount<'a, 'info>,
//     vault_fee_token_account: SanitizedAssociatedTokenAccount<'a, 'info>,
//     staker: SanitizedSignerAccount<'a, 'info>,
//     staker_lrt_token_account: SanitizedTokenAccount<'a, 'info>,
//     base: SanitizedSignerAccount<'a, 'info>,
//     system_program: SanitizedSystemProgram<'a, 'info>,
//     burn_signer: Option<SanitizedSignerAccount<'a, 'info>>,
// }
//
// impl<'a, 'info> SanitizedAccounts<'a, 'info> {
//     /// Loads accounts for [`crate::VaultInstruction::EnqueueWithdrawal`]
//     fn sanitize(
//         program_id: &Pubkey,
//         accounts: &'a [AccountInfo<'info>],
//     ) -> Result<SanitizedAccounts<'a, 'info>, ProgramError> {
//         let accounts_iter = &mut accounts.iter();
//
//         let config =
//             SanitizedConfig::sanitize(program_id, next_account_info(accounts_iter)?, false)?;
//         let vault = SanitizedVault::sanitize(program_id, next_account_info(accounts_iter)?, true)?;
//         let vault_delegation_list = SanitizedVaultDelegationList::sanitize(
//             program_id,
//             next_account_info(accounts_iter)?,
//             true,
//             vault.account().key,
//         )?;
//         let vault_staker_withdrawal_ticket =
//             EmptyAccount::sanitize(next_account_info(accounts_iter)?, true)?;
//         let vault_staker_withdrawal_ticket_token_account =
//             SanitizedAssociatedTokenAccount::sanitize(
//                 next_account_info(accounts_iter)?,
//                 &vault.vault.lrt_mint,
//                 vault_staker_withdrawal_ticket.account().key,
//             )?;
//         let vault_fee_token_account = SanitizedAssociatedTokenAccount::sanitize(
//             next_account_info(accounts_iter)?,
//             &vault.vault.lrt_mint,
//             &vault.vault.fee_wallet,
//         )?;
//         let staker = SanitizedSignerAccount::sanitize(next_account_info(accounts_iter)?, true)?;
//         let staker_lrt_token_account = SanitizedTokenAccount::sanitize(
//             next_account_info(accounts_iter)?,
//             &vault.vault.lrt_mint,
//             staker.account().key,
//         )?;
//         let base = SanitizedSignerAccount::sanitize(next_account_info(accounts_iter)?, false)?;
//         let _token_program = SanitizedTokenProgram::sanitize(next_account_info(accounts_iter)?)?;
//         let system_program = SanitizedSystemProgram::sanitize(next_account_info(accounts_iter)?)?;
//
//         let burn_signer = if vault.vault.mint_burn_authority().is_some() {
//             Some(SanitizedSignerAccount::sanitize(
//                 next_account_info(accounts_iter)?,
//                 false,
//             )?)
//         } else {
//             None
//         };
//
//         Ok(SanitizedAccounts {
//             config,
//             vault,
//             vault_delegation_list,
//             vault_staker_withdrawal_ticket,
//             vault_staker_withdrawal_ticket_token_account,
//             vault_fee_token_account,
//             staker,
//             staker_lrt_token_account,
//             base,
//             system_program,
//             burn_signer,
//         })
//     }
// }
