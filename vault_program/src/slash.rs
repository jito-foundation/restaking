use jito_restaking_core::{
    ncn::SanitizedNcn, ncn_operator_ticket::SanitizedNcnOperatorTicket,
    ncn_vault_slasher_ticket::SanitizedNcnVaultSlasherTicket,
    ncn_vault_ticket::SanitizedNcnVaultTicket, operator::SanitizedOperator,
    operator_ncn_ticket::SanitizedOperatorNcnTicket,
    operator_vault_ticket::SanitizedOperatorVaultTicket,
};
use jito_restaking_sanitization::{
    associated_token_account::SanitizedAssociatedTokenAccount, signer::SanitizedSignerAccount,
    token_program::SanitizedTokenProgram,
};
use jito_vault_core::{
    config::SanitizedConfig,
    vault::{SanitizedVault, Vault},
    vault_delegation_list::SanitizedVaultDelegationList,
    vault_ncn_slasher_operator_ticket::SanitizedVaultNcnSlasherOperatorTicket,
    vault_ncn_slasher_ticket::SanitizedVaultNcnSlasherTicket,
    vault_ncn_ticket::SanitizedVaultNcnTicket,
    vault_operator_ticket::SanitizedVaultOperatorTicket,
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
        ncn_operator_ticket,
        operator_ncn_ticket,
        ncn_vault_ticket,
        operator_vault_ticket,
        vault_ncn_ticket,
        vault_operator_ticket,
        ncn_vault_slasher_ticket,
        vault_ncn_slasher_ticket,
        mut vault_delegation_list,
        mut vault_ncn_slasher_operator_ticket,
        mut vault_token_account,
        slasher_token_account,
    } = SanitizedAccounts::sanitize(program_id, accounts, slot)?;

    vault_delegation_list
        .vault_delegation_list_mut()
        .check_update_needed(slot, config.config().epoch_length())?;

    // The vault shall be opted-in to the NCN and the NCN shall be opted-in to the vault
    vault_ncn_ticket
        .vault_ncn_ticket()
        .check_active_or_cooldown(slot, config.config().epoch_length())?;
    ncn_vault_ticket
        .ncn_vault_ticket()
        .check_active_or_cooldown(slot, config.config().epoch_length())?;

    // The operator shall be opted-in to vault and the vault shall be staked to the operator
    operator_vault_ticket
        .operator_vault_ticket()
        .check_active_or_cooldown(slot, config.config().epoch_length())?;
    vault_operator_ticket
        .vault_operator_ticket()
        .check_active_or_cooldown(slot, config.config().epoch_length())?;

    // The operator shall be opted-in to the NCN and the NCN shall be opted-in to the operator
    ncn_operator_ticket
        .ncn_operator_ticket()
        .check_active_or_cooldown(slot, config.config().epoch_length())?;
    operator_ncn_ticket
        .operator_ncn_ticket()
        .check_active_or_cooldown(slot, config.config().epoch_length())?;
    // The slasher shall be active for the NCN and the vault
    ncn_vault_slasher_ticket
        .ncn_vault_slasher_ticket()
        .check_active_or_cooldown(slot, config.config().epoch_length())?;
    vault_ncn_slasher_ticket
        .vault_ncn_slasher_ticket()
        .check_active_or_cooldown(slot, config.config().epoch_length())?;

    let max_slashable_per_epoch = vault_ncn_slasher_ticket
        .vault_ncn_slasher_ticket()
        .max_slashable_per_epoch();
    vault_ncn_slasher_operator_ticket
        .vault_ncn_slasher_operator_ticket()
        .check_max_slashable_not_exceeded(slash_amount, max_slashable_per_epoch)?;

    vault_delegation_list
        .vault_delegation_list_mut()
        .slash(operator.account().key, slash_amount)?;
    vault_ncn_slasher_operator_ticket
        .vault_ncn_slasher_operator_ticket_mut()
        .increment_slashed_amount(slash_amount)?;
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
    vault_ncn_slasher_operator_ticket.save()?;

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
    ncn_operator_ticket: SanitizedNcnOperatorTicket<'a, 'info>,
    operator_ncn_ticket: SanitizedOperatorNcnTicket<'a, 'info>,
    ncn_vault_ticket: SanitizedNcnVaultTicket<'a, 'info>,
    operator_vault_ticket: SanitizedOperatorVaultTicket<'a, 'info>,
    vault_ncn_ticket: SanitizedVaultNcnTicket<'a, 'info>,
    vault_operator_ticket: SanitizedVaultOperatorTicket<'a, 'info>,
    ncn_vault_slasher_ticket: SanitizedNcnVaultSlasherTicket<'a, 'info>,
    vault_ncn_slasher_ticket: SanitizedVaultNcnSlasherTicket<'a, 'info>,
    vault_delegation_list: SanitizedVaultDelegationList<'a, 'info>,
    vault_ncn_slasher_operator_ticket: SanitizedVaultNcnSlasherOperatorTicket<'a, 'info>,
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
        let vault =
            SanitizedVault::sanitize(program_id, next_account_info(&mut accounts_iter)?, true)?;
        let ncn = SanitizedNcn::sanitize(
            &config.config().restaking_program(),
            next_account_info(&mut accounts_iter)?,
            false,
        )?;
        let operator = SanitizedOperator::sanitize(
            &config.config().restaking_program(),
            next_account_info(&mut accounts_iter)?,
            false,
        )?;
        let slasher =
            SanitizedSignerAccount::sanitize(next_account_info(&mut accounts_iter)?, false)?;
        let ncn_operator_ticket = SanitizedNcnOperatorTicket::sanitize(
            &config.config().restaking_program(),
            next_account_info(&mut accounts_iter)?,
            false,
            ncn.account().key,
            operator.account().key,
        )?;
        let operator_ncn_ticket = SanitizedOperatorNcnTicket::sanitize(
            &config.config().restaking_program(),
            next_account_info(&mut accounts_iter)?,
            false,
            operator.account().key,
            ncn.account().key,
        )?;
        let ncn_vault_ticket = SanitizedNcnVaultTicket::sanitize(
            &config.config().restaking_program(),
            next_account_info(&mut accounts_iter)?,
            false,
            ncn.account().key,
            vault.account().key,
        )?;
        let operator_vault_ticket = SanitizedOperatorVaultTicket::sanitize(
            &config.config().restaking_program(),
            next_account_info(&mut accounts_iter)?,
            false,
            operator.account().key,
            vault.account().key,
        )?;
        let vault_ncn_ticket = SanitizedVaultNcnTicket::sanitize(
            program_id,
            next_account_info(&mut accounts_iter)?,
            false,
            vault.account().key,
            ncn.account().key,
        )?;
        let vault_operator_ticket = SanitizedVaultOperatorTicket::sanitize(
            program_id,
            next_account_info(&mut accounts_iter)?,
            false,
            vault.account().key,
            operator.account().key,
        )?;
        let ncn_vault_slasher_ticket = SanitizedNcnVaultSlasherTicket::sanitize(
            &config.config().restaking_program(),
            next_account_info(&mut accounts_iter)?,
            false,
            ncn.account().key,
            vault.account().key,
            slasher.account().key,
        )?;
        let vault_ncn_slasher_ticket = SanitizedVaultNcnSlasherTicket::sanitize(
            program_id,
            next_account_info(&mut accounts_iter)?,
            false,
            vault.account().key,
            ncn.account().key,
            slasher.account().key,
        )?;
        let vault_delegation_list = SanitizedVaultDelegationList::sanitize(
            program_id,
            next_account_info(&mut accounts_iter)?,
            true,
            vault.account().key,
        )?;
        let epoch = slot.checked_div(config.config().epoch_length()).unwrap();
        let vault_ncn_slasher_operator_ticket = SanitizedVaultNcnSlasherOperatorTicket::sanitize(
            program_id,
            next_account_info(&mut accounts_iter)?,
            true,
            vault.account().key,
            ncn.account().key,
            slasher.account().key,
            operator.account().key,
            epoch,
        )?;

        let vault_token_account = SanitizedAssociatedTokenAccount::sanitize(
            next_account_info(&mut accounts_iter)?,
            &vault.vault().supported_mint(),
            vault.account().key,
        )?;
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
            ncn_operator_ticket,
            operator_ncn_ticket,
            ncn_vault_ticket,
            operator_vault_ticket,
            vault_ncn_ticket,
            vault_operator_ticket,
            ncn_vault_slasher_ticket,
            vault_ncn_slasher_ticket,
            vault_delegation_list,
            vault_ncn_slasher_operator_ticket,
            vault_token_account,
            slasher_token_account,
        })
    }
}
