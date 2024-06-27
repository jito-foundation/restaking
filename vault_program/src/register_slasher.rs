use borsh::BorshDeserialize;
use jito_restaking_sanitization::{assert_with_msg, signer::SanitizedSignerAccount};
use jito_restaking_sdk::{get_max_slashable_per_epoch, GetMaxSlashablePerEpochResponse};
use jito_vault_core::{
    config::SanitizedConfig, vault::SanitizedVault, vault_slasher_list::SanitizedVaultSlasherList,
};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::Clock,
    entrypoint::ProgramResult,
    program::{get_return_data, invoke},
    program_error::ProgramError,
    pubkey::Pubkey,
    rent::Rent,
    sysvar::Sysvar,
};

/// Processes the register slasher instruction: [`crate::VaultInstruction::AddSlasher`]
pub fn process_register_slasher(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let account_iter = &mut accounts.iter();

    let config = SanitizedConfig::sanitize(program_id, next_account_info(account_iter)?, false)?;
    let vault = SanitizedVault::sanitize(program_id, next_account_info(account_iter)?, false)?;
    let mut vault_slasher_list = SanitizedVaultSlasherList::sanitize(
        program_id,
        next_account_info(account_iter)?,
        true,
        vault.account().key,
    )?;
    let slasher = next_account_info(account_iter)?;
    let admin = SanitizedSignerAccount::sanitize(next_account_info(account_iter)?, false)?;
    let payer = SanitizedSignerAccount::sanitize(next_account_info(account_iter)?, true)?;

    let restaking_program = next_account_info(account_iter)?;
    let avs = next_account_info(account_iter)?;
    let avs_slasher_list = next_account_info(account_iter)?;

    assert_with_msg(
        *admin.account().key == vault.vault().admin(),
        ProgramError::InvalidArgument,
        "Admin account does not match vault admin",
    )?;

    assert_with_msg(
        config.config().restaking_program() == *restaking_program.key,
        ProgramError::InvalidArgument,
        "Restaking program account does not match config",
    )?;

    // TODO (LB): any pre-checks we want to do here?
    //  check to make sure AVS in the AVS list + active?

    invoke(
        &get_max_slashable_per_epoch(
            &config.config().restaking_program(),
            avs.key,
            avs_slasher_list.key,
            slasher.key,
            vault.account().key,
        ),
        &[avs.clone(), avs_slasher_list.clone()],
    )?;

    let response = get_return_data();
    assert_with_msg(
        response.is_some(),
        ProgramError::InvalidArgument,
        "No response from get_max_slashable_per_epoch",
    )?;
    let (returning_program, data) = response.unwrap();
    assert_with_msg(
        returning_program == config.config().restaking_program(),
        ProgramError::InvalidArgument,
        "Returned program does not match restaking program",
    )?;
    let response = GetMaxSlashablePerEpochResponse::try_from_slice(&data)?;

    let slot = Clock::get()?.slot;

    let slasher_added = vault_slasher_list.vault_slasher_list_mut().add_slasher(
        avs.key,
        slasher.key,
        response.max_slashable_per_epoch,
        slot,
    );
    assert_with_msg(
        slasher_added,
        ProgramError::InvalidArgument,
        "Slasher already exists",
    )?;

    vault_slasher_list.save(&Rent::get()?, payer.account())?;

    Ok(())
}
