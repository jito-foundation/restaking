use jito_restaking_core::{
    avs::SanitizedAvs, avs_slasher_list::SanitizedAvsSlasherList,
    avs_vault_list::SanitizedAvsVaultList, config::SanitizedConfig,
};
use jito_restaking_sanitization::{assert_with_msg, signer::SanitizedSignerAccount};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::Clock,
    entrypoint::ProgramResult,
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvar::Sysvar,
};

pub fn process_avs_deprecate_slasher(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    let _config = SanitizedConfig::sanitize(program_id, next_account_info(accounts_iter)?, false)?;
    let avs = SanitizedAvs::sanitize(program_id, next_account_info(accounts_iter)?, false)?;
    let avs_vault_list = SanitizedAvsVaultList::sanitize(
        program_id,
        next_account_info(accounts_iter)?,
        false,
        avs.account().key,
    )?;
    let mut avs_slasher_list = SanitizedAvsSlasherList::sanitize(
        program_id,
        next_account_info(accounts_iter)?,
        true,
        avs.account().key,
    )?;
    let vault = next_account_info(accounts_iter)?;
    let slasher = next_account_info(accounts_iter)?;
    let admin = SanitizedSignerAccount::sanitize(next_account_info(accounts_iter)?, false)?;

    assert_with_msg(
        avs.avs().admin() == *admin.account().key,
        ProgramError::InvalidAccountData,
        "Admin is not the AVS admin",
    )?;
    assert_with_msg(
        avs_vault_list.avs_vault_list().contains_vault(*vault.key),
        ProgramError::InvalidAccountData,
        "Vault does not exist in AVS vault list",
    )?;

    let clock = Clock::get()?;
    assert_with_msg(
        avs_slasher_list.avs_slasher_list_mut().deprecate_slasher(
            *vault.key,
            *slasher.key,
            clock.slot,
        ),
        ProgramError::InvalidAccountData,
        "Slasher, vault combination does not exist in AVS slasher list",
    )?;

    avs_slasher_list.save()?;

    Ok(())
}
