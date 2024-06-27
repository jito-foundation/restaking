use jito_restaking_core::{
    avs::{Avs, SanitizedAvs},
    avs_vault_list::SanitizedAvsVaultList,
    config::{Config, SanitizedConfig},
};
use jito_restaking_sanitization::{
    assert_with_msg, signer::SanitizedSignerAccount, system_program::SanitizedSystemProgram,
};
use jito_vault_sdk::remove_avs;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::Clock,
    entrypoint::ProgramResult,
    program::invoke_signed,
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvar::Sysvar,
};

/// The AVS admin can remove support for receiving delegation from a vault.
/// The vault is removed at the end of epoch + 1.
/// This method is permissioned to the AVS admin.
///
/// [`crate::RestakingInstruction::AvsRemoveVault`]
pub fn process_avs_remove_vault(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    let config = SanitizedConfig::sanitize(program_id, next_account_info(accounts_iter)?, false)?;

    let avs = SanitizedAvs::sanitize(program_id, next_account_info(accounts_iter)?, true)?;

    let mut avs_vault_list = SanitizedAvsVaultList::sanitize(
        program_id,
        next_account_info(accounts_iter)?,
        true,
        avs.account().key,
    )?;

    let admin = SanitizedSignerAccount::sanitize(next_account_info(accounts_iter)?, true)?;

    let vault_program = next_account_info(accounts_iter)?;

    let vault = next_account_info(accounts_iter)?;
    let vault_config = next_account_info(accounts_iter)?;
    let vault_avs_list = next_account_info(accounts_iter)?;
    assert_with_msg(
        vault_avs_list.is_writable,
        ProgramError::InvalidAccountData,
        "Vault AVS list account must be writable",
    )?;
    let payer = SanitizedSignerAccount::sanitize(next_account_info(accounts_iter)?, true)?;
    let system_program = SanitizedSystemProgram::sanitize(next_account_info(accounts_iter)?)?;

    assert_with_msg(
        *vault_program.key == config.config().vault_program(),
        ProgramError::InvalidAccountData,
        "Vault program is not the correct program",
    )?;

    assert_with_msg(
        avs.avs().admin() == *admin.account().key,
        ProgramError::InvalidAccountData,
        "Admin is not the AVS admin",
    )?;

    let clock = Clock::get()?;

    assert_with_msg(
        avs_vault_list
            .avs_vault_list_mut()
            .remove_vault(*vault.key, clock.slot),
        ProgramError::InvalidAccountData,
        "Vault already exists in AVS vault list",
    )?;

    let mut config_seeds = Config::seeds();
    config_seeds.push(vec![config.config().bump()]);
    let config_seeds_slice = config_seeds
        .iter()
        .map(|seed| seed.as_slice())
        .collect::<Vec<&[u8]>>();

    let mut avs_seeds = Avs::seeds(&avs.avs().base());
    avs_seeds.push(vec![avs.avs().bump()]);

    let avs_seeds_slice = avs_seeds
        .iter()
        .map(|seed| seed.as_slice())
        .collect::<Vec<&[u8]>>();

    invoke_signed(
        &remove_avs(
            &config.config().vault_program(),
            config.account().key,
            avs.account().key,
            vault.key,
            vault_config.key,
            vault_avs_list.key,
            payer.account().key,
        ),
        &[
            config.account().clone(),
            avs.account().clone(),
            vault.clone(),
            vault_config.clone(),
            vault_avs_list.clone(),
            payer.account().clone(),
            system_program.account().clone(),
        ],
        &[config_seeds_slice.as_slice(), avs_seeds_slice.as_slice()],
    )?;

    avs.save()?;
    avs_vault_list.save()?;

    Ok(())
}
