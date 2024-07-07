use jito_restaking_core::{
    avs::SanitizedAvs, avs_operator_list::SanitizedAvsOperatorList,
    avs_slasher_list::SanitizedAvsSlasherList, avs_vault_list::SanitizedAvsVaultList,
    operator::SanitizedOperator, operator_avs_list::SanitizedOperatorAvsList,
    operator_vault_list::SanitizedOperatorVaultList,
};
use jito_restaking_sanitization::{
    associated_token_account::SanitizedAssociatedTokenAccount, signer::SanitizedSignerAccount,
    token_program::SanitizedTokenProgram,
};
use jito_vault_core::{
    config::SanitizedConfig,
    vault::{SanitizedVault, Vault},
    vault_avs_list::SanitizedVaultAvsList,
    vault_operator_list::SanitizedVaultOperatorList,
    vault_slasher_list::SanitizedVaultSlasherList,
};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::Clock,
    entrypoint_deprecated::ProgramResult,
    program::invoke_signed,
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvar::Sysvar,
};
use spl_token::instruction::transfer;

/// Processes the vault slash instruction: [`crate::VaultInstruction::Slash`]
pub fn process_slash(program_id: &Pubkey, accounts: &[AccountInfo], amount: u64) -> ProgramResult {
    let SanitizedAccounts {
        mut vault,
        vault_slasher_list,
        vault_avs_list,
        mut vault_operator_list,
        mut vault_token_account,
        avs,
        avs_vault_list,
        avs_operator_list,
        avs_slasher_list,
        operator,
        operator_vault_list,
        operator_avs_list,
        slasher,
        slasher_token_account,
    } = SanitizedAccounts::sanitize(program_id, accounts)?;

    let slot = Clock::get()?.slot;
    // The vault shall be opted-in to the AVS and the AVS shall be opted-in to the vault
    _check_vault_avs_active(&vault, &avs, &vault_avs_list, &avs_vault_list, slot)?;
    // The operator shall be opted-in to vault and the vault shall be staked to the operator
    _check_vault_operator_active(
        &vault,
        &operator,
        &vault_operator_list,
        &operator_vault_list,
        slot,
    )?;
    // The operator shall be opted-in to the AVS and the AVS shall be opted-in to the operator
    _check_avs_operator_active(
        &avs,
        &operator,
        &avs_operator_list,
        &operator_avs_list,
        slot,
    )?;
    // The slasher shall be active for the AVS and the vault
    _check_slasher_avs_vault_active(
        &slasher,
        &avs,
        &vault,
        &vault_slasher_list,
        &avs_slasher_list,
        slot,
    )?;

    // TODO (LB): check to make sure didn't exceed max slashable for the epoch for the given node operator

    vault_operator_list
        .vault_operator_list_mut()
        .slash(operator.account().key, amount)?;

    _transfer_slashed_funds(&vault, &vault_token_account, &slasher_token_account, amount)?;

    vault_token_account.reload()?;
    vault
        .vault_mut()
        .set_tokens_deposited(vault_token_account.token_account().amount);

    vault.save()?;
    vault_operator_list.save()?;

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

fn _check_slasher_avs_vault_active(
    slasher: &SanitizedSignerAccount,
    avs: &SanitizedAvs,
    vault: &SanitizedVault,
    vault_slasher_list: &SanitizedVaultSlasherList,
    avs_slasher_list: &SanitizedAvsSlasherList,
    slot: u64,
) -> ProgramResult {
    vault_slasher_list
        .vault_slasher_list()
        .check_slasher_active(slasher.account().key, avs.account().key, slot)?;

    avs_slasher_list.avs_slasher_list().check_slasher_active(
        vault.account().key,
        slasher.account().key,
        slot,
    )?;

    Ok(())
}

fn _check_avs_operator_active(
    avs: &SanitizedAvs,
    operator: &SanitizedOperator,
    avs_operator_list: &SanitizedAvsOperatorList,
    operator_avs_list: &SanitizedOperatorAvsList,
    slot: u64,
) -> ProgramResult {
    avs_operator_list
        .avs_operator_list()
        .check_operator_active(operator.account().key, slot)?;
    operator_avs_list
        .operator_avs_list()
        .check_avs_active(avs.account().key, slot)?;
    Ok(())
}

fn _check_vault_operator_active(
    vault: &SanitizedVault,
    operator: &SanitizedOperator,
    vault_operator_list: &SanitizedVaultOperatorList,
    operator_vault_list: &SanitizedOperatorVaultList,
    slot: u64,
) -> ProgramResult {
    vault_operator_list
        .vault_operator_list()
        .check_operator_active(operator.account().key, slot)?;
    operator_vault_list
        .operator_vault_list()
        .check_vault_active(vault.account().key, slot)?;
    Ok(())
}

fn _check_vault_avs_active(
    vault: &SanitizedVault,
    avs: &SanitizedAvs,
    vault_avs_list: &SanitizedVaultAvsList,
    avs_vault_list: &SanitizedAvsVaultList,
    slot: u64,
) -> ProgramResult {
    vault_avs_list
        .vault_avs_list()
        .check_avs_active(avs.account().key, slot)?;
    avs_vault_list
        .avs_vault_list()
        .check_vault_active(vault.account().key, slot)?;
    Ok(())
}

struct SanitizedAccounts<'a, 'info> {
    vault: SanitizedVault<'a, 'info>,
    vault_slasher_list: SanitizedVaultSlasherList<'a, 'info>,
    vault_avs_list: SanitizedVaultAvsList<'a, 'info>,
    vault_operator_list: SanitizedVaultOperatorList<'a, 'info>,
    vault_token_account: SanitizedAssociatedTokenAccount<'a, 'info>,
    avs: SanitizedAvs<'a, 'info>,
    avs_vault_list: SanitizedAvsVaultList<'a, 'info>,
    avs_operator_list: SanitizedAvsOperatorList<'a, 'info>,
    avs_slasher_list: SanitizedAvsSlasherList<'a, 'info>,
    operator: SanitizedOperator<'a, 'info>,
    operator_vault_list: SanitizedOperatorVaultList<'a, 'info>,
    operator_avs_list: SanitizedOperatorAvsList<'a, 'info>,
    slasher: SanitizedSignerAccount<'a, 'info>,
    slasher_token_account: SanitizedAssociatedTokenAccount<'a, 'info>,
}

impl<'a, 'info> SanitizedAccounts<'a, 'info> {
    fn sanitize(
        program_id: &Pubkey,
        accounts: &'a [AccountInfo<'info>],
    ) -> Result<Self, ProgramError> {
        let mut accounts_iter = accounts.iter();

        let config =
            SanitizedConfig::sanitize(program_id, next_account_info(&mut accounts_iter)?, false)?;

        let vault =
            SanitizedVault::sanitize(program_id, next_account_info(&mut accounts_iter)?, false)?;
        let vault_slasher_list = SanitizedVaultSlasherList::sanitize(
            program_id,
            next_account_info(&mut accounts_iter)?,
            false,
            vault.account().key,
        )?;
        let vault_avs_list = SanitizedVaultAvsList::sanitize(
            program_id,
            next_account_info(&mut accounts_iter)?,
            false,
            vault.account().key,
        )?;
        let vault_operator_list = SanitizedVaultOperatorList::sanitize(
            program_id,
            next_account_info(&mut accounts_iter)?,
            true,
            vault.account().key,
        )?;
        let vault_token_account = SanitizedAssociatedTokenAccount::sanitize(
            next_account_info(&mut accounts_iter)?,
            &vault.vault().supported_mint(),
            vault.account().key,
        )?;
        let avs = SanitizedAvs::sanitize(
            &config.config().restaking_program(),
            next_account_info(&mut accounts_iter)?,
            false,
        )?;
        let avs_vault_list = SanitizedAvsVaultList::sanitize(
            &config.config().restaking_program(),
            next_account_info(&mut accounts_iter)?,
            false,
            avs.account().key,
        )?;
        let avs_operator_list = SanitizedAvsOperatorList::sanitize(
            &config.config().restaking_program(),
            next_account_info(&mut accounts_iter)?,
            false,
            avs.account().key,
        )?;
        let avs_slasher_list = SanitizedAvsSlasherList::sanitize(
            &config.config().restaking_program(),
            next_account_info(&mut accounts_iter)?,
            false,
            avs.account().key,
        )?;
        let operator = SanitizedOperator::sanitize(
            &config.config().restaking_program(),
            next_account_info(&mut accounts_iter)?,
            false,
        )?;
        let operator_vault_list = SanitizedOperatorVaultList::sanitize(
            &config.config().restaking_program(),
            next_account_info(&mut accounts_iter)?,
            true,
            operator.account().key,
        )?;
        let operator_avs_list = SanitizedOperatorAvsList::sanitize(
            &config.config().restaking_program(),
            next_account_info(&mut accounts_iter)?,
            false,
            operator.account().key,
        )?;
        let slasher =
            SanitizedSignerAccount::sanitize(next_account_info(&mut accounts_iter)?, false)?;
        let slasher_token_account = SanitizedAssociatedTokenAccount::sanitize(
            next_account_info(&mut accounts_iter)?,
            &vault.vault().supported_mint(),
            slasher.account().key,
        )?;

        let _token_program =
            SanitizedTokenProgram::sanitize(next_account_info(&mut accounts_iter)?)?;
        Ok(Self {
            vault,
            vault_slasher_list,
            vault_avs_list,
            vault_operator_list,
            vault_token_account,
            avs,
            avs_vault_list,
            avs_operator_list,
            avs_slasher_list,
            operator,
            operator_vault_list,
            operator_avs_list,
            slasher,
            slasher_token_account,
        })
    }
}
