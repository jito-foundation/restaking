use jito_restaking_core::{
    avs::SanitizedAvs, avs_operator_ticket::SanitizedAvsOperatorTicket,
    avs_vault_slasher_ticket::SanitizedAvsVaultSlasherTicket,
    avs_vault_ticket::SanitizedAvsVaultTicket, operator::SanitizedOperator,
    operator_avs_ticket::SanitizedOperatorAvsTicket,
    operator_vault_ticket::SanitizedOperatorVaultTicket,
};
use jito_restaking_sanitization::{
    associated_token_account::SanitizedAssociatedTokenAccount, signer::SanitizedSignerAccount,
    token_program::SanitizedTokenProgram,
};
use jito_vault_core::{
    config::SanitizedConfig,
    vault::{SanitizedVault, Vault},
    vault_avs_slasher_operator_ticket::SanitizedVaultAvsSlasherOperatorTicket,
    vault_avs_slasher_ticket::SanitizedVaultAvsSlasherTicket,
    vault_avs_ticket::SanitizedVaultAvsTicket,
    vault_delegation_list::SanitizedVaultDelegationList,
    vault_operator_ticket::SanitizedVaultOperatorTicket,
};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::Clock,
    entrypoint::ProgramResult,
    msg,
    program::invoke_signed,
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvar::Sysvar,
};
use spl_token::instruction::transfer;

/// Processes the vault slash instruction: [`crate::VaultInstruction::Slash`]
pub fn process_slash(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    slash_amount: u64,
) -> ProgramResult {
    let slot = Clock::get()?.slot;
    let SanitizedAccounts {
        config,
        mut vault,
        operator,
        avs_operator_ticket,
        operator_avs_ticket,
        avs_vault_ticket,
        operator_vault_ticket,
        vault_avs_ticket,
        vault_operator_ticket,
        avs_vault_slasher_ticket,
        vault_avs_slasher_ticket,
        mut vault_delegation_list,
        mut vault_avs_slasher_operator_ticket,
        mut vault_token_account,
        slasher_token_account,
    } = SanitizedAccounts::sanitize(program_id, accounts, slot)?;

    msg!("checking update needed");
    vault_delegation_list
        .vault_delegation_list_mut()
        .check_update_needed(slot, config.config().epoch_length())?;

    // The vault shall be opted-in to the AVS and the AVS shall be opted-in to the vault
    msg!("vault <> avs check");
    vault_avs_ticket.vault_avs_ticket().check_active(slot)?;
    avs_vault_ticket.avs_vault_ticket().check_active(slot)?;

    // The operator shall be opted-in to vault and the vault shall be staked to the operator
    msg!("operator <> vault check");
    operator_vault_ticket
        .operator_vault_ticket()
        .check_active(slot)?;
    vault_operator_ticket
        .vault_operator_ticket()
        .check_active(slot)?;

    // The operator shall be opted-in to the AVS and the AVS shall be opted-in to the operator
    msg!("avs <> operator check");
    avs_operator_ticket
        .avs_operator_ticket()
        .check_active(slot)?;
    operator_avs_ticket
        .operator_avs_ticket()
        .check_active(slot)?;
    // The slasher shall be active for the AVS and the vault
    msg!("avs <> vault check");
    avs_vault_slasher_ticket
        .avs_vault_slasher_ticket()
        .check_active(slot)?;
    vault_avs_slasher_ticket
        .vault_avs_slasher_ticket()
        .check_active(slot)?;

    msg!("max exceeded check");
    let max_slashable_per_epoch = vault_avs_slasher_ticket
        .vault_avs_slasher_ticket()
        .max_slashable_per_epoch();
    vault_avs_slasher_operator_ticket
        .vault_avs_slasher_operator_ticket()
        .check_max_slashable_not_exceeded(slash_amount, max_slashable_per_epoch)?;

    msg!("slashing");
    vault_delegation_list
        .vault_delegation_list_mut()
        .slash(operator.account().key, slash_amount)?;
    vault_avs_slasher_operator_ticket
        .vault_avs_slasher_operator_ticket_mut()
        .increment_slashed_amount(slash_amount)?;
    msg!("sending slashed funds");
    _transfer_slashed_funds(
        &vault,
        &vault_token_account,
        &slasher_token_account,
        slash_amount,
    )?;

    vault_token_account.reload()?;
    vault
        .vault_mut()
        .set_tokens_deposited(vault_token_account.token_account().amount);

    vault.save()?;
    vault_delegation_list.save()?;
    vault_avs_slasher_operator_ticket.save()?;

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn _transfer_slashed_funds<'a, 'info>(
    vault: &SanitizedVault<'a, 'info>,
    vault_token_account: &SanitizedAssociatedTokenAccount<'a, 'info>,
    slasher_token_account: &SanitizedAssociatedTokenAccount<'a, 'info>,
    amount: u64,
) -> ProgramResult {
    let mut vault_seeds = Vault::seeds(&vault.vault().base());
    vault_seeds.push(vec![vault.vault().bump()]);
    let vault_seeds_slice = vault_seeds
        .iter()
        .map(|seed| seed.as_slice())
        .collect::<Vec<&[u8]>>();

    invoke_signed(
        &transfer(
            &spl_token::id(),
            vault_token_account.account().key,
            slasher_token_account.account().key,
            vault.account().key,
            &[],
            amount,
        )?,
        &[
            vault_token_account.account().clone(),
            slasher_token_account.account().clone(),
            vault.account().clone(),
        ],
        &[vault_seeds_slice.as_slice()],
    )?;

    Ok(())
}

struct SanitizedAccounts<'a, 'info> {
    config: SanitizedConfig<'a, 'info>,
    vault: SanitizedVault<'a, 'info>,
    operator: SanitizedOperator<'a, 'info>,
    avs_operator_ticket: SanitizedAvsOperatorTicket<'a, 'info>,
    operator_avs_ticket: SanitizedOperatorAvsTicket<'a, 'info>,
    avs_vault_ticket: SanitizedAvsVaultTicket<'a, 'info>,
    operator_vault_ticket: SanitizedOperatorVaultTicket<'a, 'info>,
    vault_avs_ticket: SanitizedVaultAvsTicket<'a, 'info>,
    vault_operator_ticket: SanitizedVaultOperatorTicket<'a, 'info>,
    avs_vault_slasher_ticket: SanitizedAvsVaultSlasherTicket<'a, 'info>,
    vault_avs_slasher_ticket: SanitizedVaultAvsSlasherTicket<'a, 'info>,
    vault_delegation_list: SanitizedVaultDelegationList<'a, 'info>,
    vault_avs_slasher_operator_ticket: SanitizedVaultAvsSlasherOperatorTicket<'a, 'info>,
    vault_token_account: SanitizedAssociatedTokenAccount<'a, 'info>,
    slasher_token_account: SanitizedAssociatedTokenAccount<'a, 'info>,
}

impl<'a, 'info> SanitizedAccounts<'a, 'info> {
    /// Sanitizes the accounts for the slash instruction: [`crate::VaultInstruction::Slash`]
    fn sanitize(
        program_id: &Pubkey,
        accounts: &'a [AccountInfo<'info>],
        slot: u64,
    ) -> Result<Self, ProgramError> {
        let mut accounts_iter = accounts.iter();

        let config =
            SanitizedConfig::sanitize(program_id, next_account_info(&mut accounts_iter)?, false)?;
        msg!("a");
        let vault =
            SanitizedVault::sanitize(program_id, next_account_info(&mut accounts_iter)?, true)?;
        msg!("b");
        let avs = SanitizedAvs::sanitize(
            &config.config().restaking_program(),
            next_account_info(&mut accounts_iter)?,
            false,
        )?;
        msg!("c");
        let operator = SanitizedOperator::sanitize(
            &config.config().restaking_program(),
            next_account_info(&mut accounts_iter)?,
            false,
        )?;
        msg!("d");
        let slasher =
            SanitizedSignerAccount::sanitize(next_account_info(&mut accounts_iter)?, false)?;
        msg!("e");
        let avs_operator_ticket = SanitizedAvsOperatorTicket::sanitize(
            &config.config().restaking_program(),
            next_account_info(&mut accounts_iter)?,
            false,
            avs.account().key,
            operator.account().key,
        )?;
        msg!("f");
        let operator_avs_ticket = SanitizedOperatorAvsTicket::sanitize(
            &config.config().restaking_program(),
            next_account_info(&mut accounts_iter)?,
            false,
            operator.account().key,
            avs.account().key,
        )?;
        msg!("g");
        let avs_vault_ticket = SanitizedAvsVaultTicket::sanitize(
            &config.config().restaking_program(),
            next_account_info(&mut accounts_iter)?,
            false,
            avs.account().key,
            vault.account().key,
        )?;
        msg!("h");
        let operator_vault_ticket = SanitizedOperatorVaultTicket::sanitize(
            &config.config().restaking_program(),
            next_account_info(&mut accounts_iter)?,
            false,
            operator.account().key,
            vault.account().key,
        )?;
        msg!("i");
        let vault_avs_ticket = SanitizedVaultAvsTicket::sanitize(
            program_id,
            next_account_info(&mut accounts_iter)?,
            false,
            vault.account().key,
            avs.account().key,
        )?;
        msg!("j");
        let vault_operator_ticket = SanitizedVaultOperatorTicket::sanitize(
            program_id,
            next_account_info(&mut accounts_iter)?,
            false,
            vault.account().key,
            operator.account().key,
        )?;
        msg!("k");
        let avs_vault_slasher_ticket = SanitizedAvsVaultSlasherTicket::sanitize(
            &config.config().restaking_program(),
            next_account_info(&mut accounts_iter)?,
            false,
            avs.account().key,
            vault.account().key,
            slasher.account().key,
        )?;
        msg!("l");
        let vault_avs_slasher_ticket = SanitizedVaultAvsSlasherTicket::sanitize(
            program_id,
            next_account_info(&mut accounts_iter)?,
            false,
            vault.account().key,
            avs.account().key,
            slasher.account().key,
        )?;
        msg!("m");
        let vault_delegation_list = SanitizedVaultDelegationList::sanitize(
            program_id,
            next_account_info(&mut accounts_iter)?,
            true,
            vault.account().key,
        )?;
        msg!("n");
        let epoch = slot.checked_div(config.config().epoch_length()).unwrap();
        let vault_avs_slasher_operator_ticket = SanitizedVaultAvsSlasherOperatorTicket::sanitize(
            program_id,
            next_account_info(&mut accounts_iter)?,
            true,
            vault.account().key,
            avs.account().key,
            slasher.account().key,
            operator.account().key,
            epoch,
        )?;

        msg!("o");
        let vault_token_account = SanitizedAssociatedTokenAccount::sanitize(
            next_account_info(&mut accounts_iter)?,
            &vault.vault().supported_mint(),
            vault.account().key,
        )?;
        msg!("p");
        let slasher_token_account = SanitizedAssociatedTokenAccount::sanitize(
            next_account_info(&mut accounts_iter)?,
            &vault.vault().supported_mint(),
            slasher.account().key,
        )?;
        let _token_program =
            SanitizedTokenProgram::sanitize(next_account_info(&mut accounts_iter)?)?;
        Ok(Self {
            config,
            vault,
            operator,
            avs_operator_ticket,
            operator_avs_ticket,
            avs_vault_ticket,
            operator_vault_ticket,
            vault_avs_ticket,
            vault_operator_ticket,
            avs_vault_slasher_ticket,
            vault_avs_slasher_ticket,
            vault_delegation_list,
            vault_avs_slasher_operator_ticket,
            vault_token_account,
            slasher_token_account,
        })
    }
}
