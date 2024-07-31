use jito_restaking_sanitization::{
    assert_with_msg, associated_token_account::SanitizedAssociatedTokenAccount,
    close_program_account, signer::SanitizedSignerAccount, system_program::SanitizedSystemProgram,
    token_mint::SanitizedTokenMint, token_program::SanitizedTokenProgram,
};
use jito_vault_core::{
    config::SanitizedConfig,
    vault::{SanitizedVault, Vault},
    vault_delegation_list::SanitizedVaultDelegationList,
    vault_staker_withdraw_ticket::{SanitizedVaultStakerWithdrawTicket, VaultStakerWithdrawTicket},
};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::Clock,
    entrypoint::ProgramResult,
    program::invoke_signed,
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvar::Sysvar,
};
use spl_token::instruction::{burn, close_account, transfer};

pub fn process_burn_withdraw_ticket(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let SanitizedAccounts {
        config,
        mut vault,
        mut vault_delegation_list,
        mut vault_token_account,
        mut lrt_mint,
        staker,
        mut staker_token_account,
        staker_lrt_token_account,
        vault_staker_withdraw_ticket,
        mut vault_staker_withdraw_ticket_token_account,
    } = SanitizedAccounts::sanitize(program_id, accounts)?;

    let slot = Clock::get()?.slot;
    let epoch_length = config.config().epoch_length();

    assert_with_msg(
        vault.vault().lrt_mint() == *lrt_mint.account().key,
        ProgramError::InvalidArgument,
        "LRT mint mismatch",
    )?;
    vault_staker_withdraw_ticket
        .vault_staker_withdraw_ticket()
        .check_withdrawable(slot, epoch_length)?;
    vault_delegation_list
        .vault_delegation_list_mut()
        .check_update_needed(slot, epoch_length)?;

    // find the current redemption amount and the original redemption amount in the withdraw ticket
    let redemption_amount = vault.vault().calculate_assets_returned_amount(
        vault_staker_withdraw_ticket
            .vault_staker_withdraw_ticket()
            .lrt_amount(),
    )?;
    let original_redemption_amount = vault_staker_withdraw_ticket
        .vault_staker_withdraw_ticket()
        .withdraw_allocation_amount();

    let actual_withdraw_amount = if redemption_amount > original_redemption_amount {
        // The program can guarantee the original redemption amount, but if the redemption amount
        // is greater than the original amount, there were rewards that accrued
        // to the LRT.
        // The program attempts to figure out how much more of the asset can be unstaked to fulfill
        // as much of the redemption amount as possible.
        // Available unstaked assets is equal to:
        // the amount of tokens deposited - any delegated security - the amount reserved for withdraw tickets
        let tokens_deposited_in_vault = vault.vault().tokens_deposited();
        let delegated_security_in_vault = vault_delegation_list
            .vault_delegation_list()
            .delegated_security()?;
        let assets_reserved_for_withdraw_tickets = vault_delegation_list
            .vault_delegation_list()
            .amount_withdrawable_by_tickets();

        let available_unstaked_assets = tokens_deposited_in_vault
            .checked_sub(delegated_security_in_vault)
            .ok_or(ProgramError::InsufficientFunds)?
            .checked_sub(assets_reserved_for_withdraw_tickets)
            .ok_or(ProgramError::InsufficientFunds)?;

        // Calculate the extra amount that can be withdrawn
        let extra_amount = redemption_amount
            .checked_sub(original_redemption_amount)
            .ok_or(ProgramError::ArithmeticOverflow)?;

        // Determine the actual amount to withdraw
        original_redemption_amount
            .checked_add(extra_amount.min(available_unstaked_assets))
            .ok_or(ProgramError::ArithmeticOverflow)?
    } else {
        redemption_amount
    };

    let lrt_to_burn = vault
        .vault()
        .calculate_lrt_mint_amount(actual_withdraw_amount)?;
    let lrt_amount_to_burn = std::cmp::min(
        lrt_to_burn,
        vault_staker_withdraw_ticket
            .vault_staker_withdraw_ticket()
            .lrt_amount(),
    );

    _burn_lrt(
        program_id,
        &vault,
        &staker,
        &vault_staker_withdraw_ticket,
        &vault_staker_withdraw_ticket_token_account,
        &lrt_mint,
        lrt_amount_to_burn,
    )?;
    lrt_mint.reload()?;
    vault_staker_withdraw_ticket_token_account.reload()?;

    _transfer_vault_tokens_to_staker(
        program_id,
        &vault,
        &vault_token_account,
        &staker_token_account,
        actual_withdraw_amount,
    )?;
    vault_token_account.reload()?;
    staker_token_account.reload()?;

    // Decrement the amount reserved for withdraw tickets because it's been claimed now
    vault_delegation_list
        .vault_delegation_list_mut()
        .decrement_amount_withdrawable_by_tickets(original_redemption_amount)?;

    // refresh after burn
    vault
        .vault_mut()
        .set_tokens_deposited(vault_token_account.token_account().amount);
    vault.vault_mut().set_lrt_supply(lrt_mint.mint().supply);

    vault.save()?;
    vault_delegation_list.save()?;

    close_program_account(
        program_id,
        vault_staker_withdraw_ticket.account(),
        staker.account(),
    )?;
    _close_token_account(
        program_id,
        &vault,
        &staker,
        &vault_staker_withdraw_ticket,
        &vault_staker_withdraw_ticket_token_account,
        &staker_lrt_token_account,
    )?;

    Ok(())
}

/// transfers all remaining assets to the staker + closes the account
fn _close_token_account<'a, 'info>(
    program_id: &Pubkey,
    vault: &SanitizedVault<'a, 'info>,
    staker: &SanitizedSignerAccount<'a, 'info>,
    vault_staker_withdraw_ticket: &SanitizedVaultStakerWithdrawTicket<'a, 'info>,
    vault_staker_withdraw_ticket_token_account: &SanitizedAssociatedTokenAccount<'a, 'info>,
    staker_lrt_token_account: &SanitizedAssociatedTokenAccount<'a, 'info>,
) -> ProgramResult {
    // TODO: combine with burn lrt method
    let (_, bump, mut seeds) = VaultStakerWithdrawTicket::find_program_address(
        program_id,
        vault.account().key,
        staker.account().key,
        &vault_staker_withdraw_ticket
            .vault_staker_withdraw_ticket()
            .base(),
    );
    seeds.push(vec![bump]);
    let seed_slices: Vec<&[u8]> = seeds.iter().map(|seed| seed.as_slice()).collect();

    invoke_signed(
        &transfer(
            &spl_token::id(),
            vault_staker_withdraw_ticket_token_account.account().key,
            staker_lrt_token_account.account().key,
            vault.account().key,
            &[],
            vault_staker_withdraw_ticket_token_account
                .token_account()
                .amount,
        )?,
        &[
            vault_staker_withdraw_ticket_token_account.account().clone(),
            staker_lrt_token_account.account().clone(),
            vault_staker_withdraw_ticket.account().clone(),
        ],
        &[&seed_slices],
    )?;

    invoke_signed(
        &close_account(
            &spl_token::id(),
            vault_staker_withdraw_ticket_token_account.account().key,
            staker.account().key,
            staker.account().key,
            &[],
        )?,
        &[
            vault_staker_withdraw_ticket_token_account.account().clone(),
            staker.account().clone(),
            staker.account().clone(),
        ],
        &[&seed_slices],
    )?;
    Ok(())
}

fn _transfer_vault_tokens_to_staker<'a, 'info>(
    program_id: &Pubkey,
    vault: &SanitizedVault<'a, 'info>,
    vault_token_account: &SanitizedAssociatedTokenAccount<'a, 'info>,
    staker_token_account: &SanitizedAssociatedTokenAccount<'a, 'info>,
    amount: u64,
) -> ProgramResult {
    let (_, bump, mut seeds) = Vault::find_program_address(program_id, &vault.vault().base());
    seeds.push(vec![bump]);
    let seed_slices: Vec<&[u8]> = seeds.iter().map(|seed| seed.as_slice()).collect();

    invoke_signed(
        &transfer(
            &spl_token::id(),
            vault_token_account.account().key,
            staker_token_account.account().key,
            vault.account().key,
            &[],
            amount,
        )?,
        &[
            vault_token_account.account().clone(),
            staker_token_account.account().clone(),
            vault.account().clone(),
        ],
        &[&seed_slices],
    )?;
    Ok(())
}

fn _burn_lrt<'a, 'info>(
    program_id: &Pubkey,
    vault: &SanitizedVault<'a, 'info>,
    staker: &SanitizedSignerAccount<'a, 'info>,
    vault_staker_withdraw_ticket: &SanitizedVaultStakerWithdrawTicket<'a, 'info>,
    vault_staker_withdraw_ticket_token_account: &SanitizedAssociatedTokenAccount<'a, 'info>,
    token_mint: &SanitizedTokenMint<'a, 'info>,
    burn_amount: u64,
) -> ProgramResult {
    let (_, bump, mut seeds) = VaultStakerWithdrawTicket::find_program_address(
        program_id,
        vault.account().key,
        staker.account().key,
        &vault_staker_withdraw_ticket
            .vault_staker_withdraw_ticket()
            .base(),
    );
    seeds.push(vec![bump]);
    let seed_slices: Vec<&[u8]> = seeds.iter().map(|seed| seed.as_slice()).collect();

    invoke_signed(
        &burn(
            &spl_token::id(),
            vault_staker_withdraw_ticket_token_account.account().key,
            token_mint.account().key,
            vault.account().key,
            &[],
            burn_amount,
        )?,
        &[
            vault_staker_withdraw_ticket_token_account.account().clone(),
            token_mint.account().clone(),
            vault_staker_withdraw_ticket.account().clone(),
        ],
        &[&seed_slices],
    )
}

pub struct SanitizedAccounts<'a, 'info> {
    config: SanitizedConfig<'a, 'info>,
    vault: SanitizedVault<'a, 'info>,
    vault_delegation_list: SanitizedVaultDelegationList<'a, 'info>,
    vault_token_account: SanitizedAssociatedTokenAccount<'a, 'info>,
    lrt_mint: SanitizedTokenMint<'a, 'info>,
    staker: SanitizedSignerAccount<'a, 'info>,
    staker_token_account: SanitizedAssociatedTokenAccount<'a, 'info>,
    staker_lrt_token_account: SanitizedAssociatedTokenAccount<'a, 'info>,
    vault_staker_withdraw_ticket: SanitizedVaultStakerWithdrawTicket<'a, 'info>,
    vault_staker_withdraw_ticket_token_account: SanitizedAssociatedTokenAccount<'a, 'info>,
}

impl<'a, 'info> SanitizedAccounts<'a, 'info> {
    fn sanitize(
        program_id: &Pubkey,
        accounts: &'a [AccountInfo<'info>],
    ) -> Result<SanitizedAccounts<'a, 'info>, ProgramError> {
        let accounts_iter = &mut accounts.iter();

        let config =
            SanitizedConfig::sanitize(program_id, next_account_info(accounts_iter)?, false)?;
        let vault = SanitizedVault::sanitize(program_id, next_account_info(accounts_iter)?, true)?;
        let vault_delegation_list = SanitizedVaultDelegationList::sanitize(
            program_id,
            next_account_info(accounts_iter)?,
            true,
            vault.account().key,
        )?;
        let vault_token_account = SanitizedAssociatedTokenAccount::sanitize(
            next_account_info(accounts_iter)?,
            &vault.vault().supported_mint(),
            vault.account().key,
        )?;
        let lrt_mint = SanitizedTokenMint::sanitize(next_account_info(accounts_iter)?, true)?;
        let staker = SanitizedSignerAccount::sanitize(next_account_info(accounts_iter)?, true)?;
        let staker_token_account = SanitizedAssociatedTokenAccount::sanitize(
            next_account_info(accounts_iter)?,
            &vault.vault().supported_mint(),
            staker.account().key,
        )?;
        let staker_lrt_token_account = SanitizedAssociatedTokenAccount::sanitize(
            next_account_info(accounts_iter)?,
            &vault.vault().lrt_mint(),
            staker.account().key,
        )?;
        let vault_staker_withdraw_ticket = SanitizedVaultStakerWithdrawTicket::sanitize(
            program_id,
            next_account_info(accounts_iter)?,
            vault.account().key,
            staker.account().key,
            true,
        )?;
        let vault_staker_withdraw_ticket_token_account = SanitizedAssociatedTokenAccount::sanitize(
            next_account_info(accounts_iter)?,
            &vault.vault().supported_mint(),
            vault_staker_withdraw_ticket.account().key,
        )?;
        let _token_program = SanitizedTokenProgram::sanitize(next_account_info(accounts_iter)?)?;
        let _system_program = SanitizedSystemProgram::sanitize(next_account_info(accounts_iter)?)?;

        Ok(SanitizedAccounts {
            config,
            vault,
            vault_delegation_list,
            vault_token_account,
            lrt_mint,
            staker,
            staker_token_account,
            staker_lrt_token_account,
            vault_staker_withdraw_ticket,
            vault_staker_withdraw_ticket_token_account,
        })
    }
}