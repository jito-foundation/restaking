use jito_restaking_core::{
    config::{Config, SanitizedConfig},
    operator::{Operator, SanitizedOperator, SanitizedOperatorVaultList},
};
use jito_restaking_sanitization::{
    assert_with_msg, signer::SanitizedSignerAccount, system_program::SanitizedSystemProgram,
};
use jito_vault_sdk::remove_operator;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::Clock,
    entrypoint::ProgramResult,
    program::invoke_signed,
    program_error::ProgramError,
    pubkey::Pubkey,
    rent::Rent,
    sysvar::Sysvar,
};

/// The node operator admin can remove support for receiving delegation from a vault.
/// The vault can be used at the end of epoch + 1.
/// This method is permissioned to the node operator admin.
/// [`crate::RestakingInstruction::OperatorRemoveVault`]
pub fn process_node_operator_remove_vault(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    let config = SanitizedConfig::sanitize(program_id, next_account_info(accounts_iter)?, false)?;

    let node_operator =
        SanitizedOperator::sanitize(program_id, next_account_info(accounts_iter)?, true)?;

    let mut node_operator_vault_list = SanitizedOperatorVaultList::sanitize(
        program_id,
        next_account_info(accounts_iter)?,
        true,
        node_operator.account().key,
    )?;

    let admin = SanitizedSignerAccount::sanitize(next_account_info(accounts_iter)?, true)?;

    let vault_program = next_account_info(accounts_iter)?;

    let vault = next_account_info(accounts_iter)?;
    let vault_config = next_account_info(accounts_iter)?;
    let vault_operator_list = next_account_info(accounts_iter)?;
    let payer = SanitizedSignerAccount::sanitize(next_account_info(accounts_iter)?, true)?;
    let system_program = SanitizedSystemProgram::sanitize(next_account_info(accounts_iter)?)?;

    assert_with_msg(
        *vault_program.key == config.config().vault_program(),
        ProgramError::InvalidAccountData,
        "Vault program is not the correct program",
    )?;

    assert_with_msg(
        node_operator.operator().admin() == *admin.account().key,
        ProgramError::InvalidAccountData,
        "Admin is not the node operator admin",
    )?;

    let mut config_seeds = Config::seeds();
    config_seeds.push(vec![config.config().bump()]);
    let config_seeds_slice = config_seeds
        .iter()
        .map(|seed| seed.as_slice())
        .collect::<Vec<&[u8]>>();

    let mut node_operator_seeds = Operator::seeds(&node_operator.operator().base());
    node_operator_seeds.push(vec![node_operator.operator().bump()]);
    let node_operator_seeds_slice = node_operator_seeds
        .iter()
        .map(|seed| seed.as_slice())
        .collect::<Vec<&[u8]>>();

    invoke_signed(
        &remove_operator(
            &config.config().vault_program(),
            config.account().key,
            node_operator.account().key,
            vault.key,
            vault_config.key,
            vault_operator_list.key,
            payer.account().key,
        ),
        &[
            config.account().clone(),
            node_operator.account().clone(),
            vault.clone(),
            vault_config.clone(),
            vault_operator_list.clone(),
            payer.account().clone(),
            system_program.account().clone(),
        ],
        &[
            config_seeds_slice.as_slice(),
            node_operator_seeds_slice.as_slice(),
        ],
    )?;

    let clock = Clock::get()?;

    assert_with_msg(
        node_operator_vault_list
            .operator_vault_list_mut()
            .remove_vault(*vault.key, clock.slot),
        ProgramError::InvalidAccountData,
        "Vault already exists in operator vault list",
    )?;

    node_operator.save()?;
    node_operator_vault_list.save(&Rent::get()?, admin.account())?;

    Ok(())
}