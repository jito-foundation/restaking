use jito_restaking_core::{
    avs::SanitizedAvs, avs_operator_list::SanitizedAvsOperatorList, operator::SanitizedOperator,
};
use jito_restaking_sanitization::{
    assert_with_msg, associated_token_account::SanitizedAssociatedTokenAccount,
    signer::SanitizedSignerAccount, token_program::SanitizedTokenProgram,
};
use jito_vault_core::{
    config::SanitizedConfig,
    vault::{SanitizedVault, Vault},
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

pub fn process_slash(program_id: &Pubkey, accounts: &[AccountInfo], amount: u64) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    let config = SanitizedConfig::sanitize(program_id, next_account_info(accounts_iter)?, false)?;
    let mut vault = SanitizedVault::sanitize(program_id, next_account_info(accounts_iter)?, false)?;
    let vault_slasher_list = SanitizedVaultSlasherList::sanitize(
        program_id,
        next_account_info(accounts_iter)?,
        false,
        vault.account().key,
    )?;
    let mut vault_operator_list = SanitizedVaultOperatorList::sanitize(
        program_id,
        next_account_info(accounts_iter)?,
        true,
        vault.account().key,
    )?;
    let mut vault_token_account = SanitizedAssociatedTokenAccount::sanitize(
        next_account_info(accounts_iter)?,
        &vault.vault().supported_mint(),
        vault.account().key,
    )?;
    let avs = SanitizedAvs::sanitize(
        &config.config().restaking_program(),
        next_account_info(accounts_iter)?,
        false,
    )?;
    let avs_operator_list = SanitizedAvsOperatorList::sanitize(
        &config.config().restaking_program(),
        next_account_info(accounts_iter)?,
        false,
        avs.account().key,
    )?;
    let operator = SanitizedOperator::sanitize(
        &config.config().restaking_program(),
        next_account_info(accounts_iter)?,
        false,
    )?;
    let slasher = SanitizedSignerAccount::sanitize(next_account_info(accounts_iter)?, false)?;
    let slasher_token_account = SanitizedAssociatedTokenAccount::sanitize(
        next_account_info(accounts_iter)?,
        &vault.vault().supported_mint(),
        slasher.account().key,
    )?;
    let _token_program = SanitizedTokenProgram::sanitize(next_account_info(accounts_iter)?)?;

    let slot = Clock::get()?.slot;

    slash_vault_operator(
        &mut vault,
        &mut vault_operator_list,
        &vault_slasher_list,
        &avs,
        &avs_operator_list,
        &operator,
        &slasher,
        &mut vault_token_account,
        &slasher_token_account,
        slot,
        amount,
    )?;

    vault.save()?;
    vault_operator_list.save()?;

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn slash_vault_operator<'a, 'info>(
    vault: &mut SanitizedVault<'a, 'info>,
    vault_operator_list: &mut SanitizedVaultOperatorList,
    vault_slasher_list: &SanitizedVaultSlasherList,
    avs: &SanitizedAvs,
    avs_operator_list: &SanitizedAvsOperatorList,
    operator: &SanitizedOperator,
    slasher: &SanitizedSignerAccount,
    vault_token_account: &mut SanitizedAssociatedTokenAccount<'a, 'info>,
    slasher_token_account: &SanitizedAssociatedTokenAccount<'a, 'info>,
    slot: u64,
    amount: u64,
) -> ProgramResult {
    // The slasher for the given AVS on this vault shall be registered with the vault
    let slasher_info = vault_slasher_list.vault_slasher_list().get_active_slasher(
        slasher.account().key,
        avs.account().key,
        slot,
    );
    assert_with_msg(
        slasher_info.is_some(),
        ProgramError::InvalidArgument,
        "Slasher is not in the vault slasher list",
    )?;

    // The AVS shall have the operator as active
    let operator_info = avs_operator_list
        .avs_operator_list()
        .get_active_operator(operator.account().key, slot);
    assert_with_msg(
        operator_info.is_some(),
        ProgramError::InvalidArgument,
        "Operator is not in the AVS operator list",
    )?;

    // TODO The operator shall be staked by the vault and the max slashable
    //  shall be the delegated funds vault_operator_list
    let slashable_amount = vault_operator_list
        .vault_operator_list_mut()
        .slash(operator.account().key, amount);
    assert_with_msg(
        slashable_amount.is_some(),
        ProgramError::InvalidArgument,
        "Operator has no funds to slash",
    )?;
    let slashable_amount = slashable_amount.unwrap();

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
            slashable_amount,
        )?,
        &[
            vault_token_account.account().clone(),
            slasher_token_account.account().clone(),
            vault.account().clone(),
        ],
        &[vault_seeds_slice.as_slice()],
    )?;

    vault_token_account.reload()?;
    vault
        .vault_mut()
        .set_tokens_deposited(vault_token_account.token_account().amount);

    Ok(())
}
