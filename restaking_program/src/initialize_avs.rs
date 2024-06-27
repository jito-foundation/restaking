use borsh::BorshSerialize;
use jito_restaking_core::{
    avs::Avs, avs_operator_list::AvsOperatorList, avs_slasher_list::AvsSlasherList,
    avs_vault_list::AvsVaultList, config::SanitizedConfig,
};
use jito_restaking_sanitization::{
    assert_with_msg, create_account, signer::SanitizedSignerAccount,
    system_program::SanitizedSystemProgram,
};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
    rent::Rent,
    sysvar::Sysvar,
};

/// Initializes an AVS and associated accounts
/// [`crate::RestakingInstruction::InitializeAvs`]
pub fn process_initialize_avs(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    let mut config =
        SanitizedConfig::sanitize(program_id, next_account_info(accounts_iter)?, true)?;

    let avs_account = next_account_info(accounts_iter)?;
    assert_with_msg(
        avs_account.is_writable,
        ProgramError::InvalidAccountData,
        "AVS account must be writable",
    )?;
    assert_with_msg(
        avs_account.data_is_empty(),
        ProgramError::InvalidAccountData,
        "AVS account must be empty",
    )?;

    let avs_operator_list_account = next_account_info(accounts_iter)?;
    assert_with_msg(
        avs_operator_list_account.is_writable,
        ProgramError::InvalidAccountData,
        "AVS operator list account must be writable",
    )?;
    assert_with_msg(
        avs_operator_list_account.data_is_empty(),
        ProgramError::InvalidAccountData,
        "AVS operator list account must be empty",
    )?;

    let avs_vault_list_account = next_account_info(accounts_iter)?;
    assert_with_msg(
        avs_vault_list_account.is_writable,
        ProgramError::InvalidAccountData,
        "AVS vault list account must be writable",
    )?;
    assert_with_msg(
        avs_vault_list_account.data_is_empty(),
        ProgramError::InvalidAccountData,
        "AVS vault list account must be empty",
    )?;

    let avs_slasher_list_account = next_account_info(accounts_iter)?;
    assert_with_msg(
        avs_slasher_list_account.is_writable,
        ProgramError::InvalidAccountData,
        "AVS slasher list account must be writable",
    )?;
    assert_with_msg(
        avs_slasher_list_account.data_is_empty(),
        ProgramError::InvalidAccountData,
        "AVS slasher list account must be empty",
    )?;

    let admin = SanitizedSignerAccount::sanitize(next_account_info(accounts_iter)?, true)?;
    let base = SanitizedSignerAccount::sanitize(next_account_info(accounts_iter)?, true)?;

    let system_program = SanitizedSystemProgram::sanitize(next_account_info(accounts_iter)?)?;

    let (expected_avs_pubkey, avs_bump, mut avs_seeds) =
        Avs::find_program_address(program_id, base.account().key);
    avs_seeds.push(vec![avs_bump]);
    assert_with_msg(
        expected_avs_pubkey == *avs_account.key,
        ProgramError::InvalidAccountData,
        "AVS account is not at the correct PDA",
    )?;

    let (expected_avs_operator_list_pubkey, avs_operator_list_bump, mut avs_operator_list_seeds) =
        AvsOperatorList::find_program_address(program_id, avs_account.key);
    avs_operator_list_seeds.push(vec![avs_operator_list_bump]);
    assert_with_msg(
        expected_avs_operator_list_pubkey == *avs_operator_list_account.key,
        ProgramError::InvalidAccountData,
        "AVS operator list account is not at the correct PDA",
    )?;

    let (expected_avs_vault_list_pubkey, avs_vault_list_bump, mut avs_vault_list_seeds) =
        AvsVaultList::find_program_address(program_id, avs_account.key);
    avs_vault_list_seeds.push(vec![avs_vault_list_bump]);
    assert_with_msg(
        expected_avs_vault_list_pubkey == *avs_vault_list_account.key,
        ProgramError::InvalidAccountData,
        "AVS vault list account is not at the correct PDA",
    )?;

    let (expected_avs_slasher_list_pubkey, avs_slasher_list_bump, mut avs_slasher_list_seeds) =
        AvsSlasherList::find_program_address(program_id, avs_account.key);
    avs_slasher_list_seeds.push(vec![avs_slasher_list_bump]);
    assert_with_msg(
        expected_avs_slasher_list_pubkey == *avs_slasher_list_account.key,
        ProgramError::InvalidAccountData,
        "AVS slasher list account is not at the correct PDA",
    )?;

    let avs = Avs::new(
        *base.account().key,
        *admin.account().key,
        config.config().avs_count(),
        avs_bump,
    );
    let avs_operator_list = AvsOperatorList::new(*avs_account.key, avs_operator_list_bump);
    let avs_vault_list = AvsVaultList::new(*avs_account.key, avs_vault_list_bump);
    let avs_slasher_list = AvsSlasherList::new(*avs_account.key, avs_slasher_list_bump);

    let num_avs = config.config_mut().increment_avs();
    assert_with_msg(
        num_avs.is_some(),
        ProgramError::InvalidAccountData,
        "Number of AVS accounts has reached the maximum",
    )?;

    let rent = Rent::get()?;

    msg!("Initializing AVS @ address {}", avs_account.key);
    let serialized_avs = avs.try_to_vec()?;
    create_account(
        admin.account(),
        avs_account,
        system_program.account(),
        program_id,
        &rent,
        serialized_avs.len() as u64,
        &avs_seeds,
    )?;
    avs_account.data.borrow_mut()[..serialized_avs.len()].copy_from_slice(&serialized_avs);

    msg!(
        "Initializing AVS operator list @ address {}",
        avs_operator_list_account.key
    );
    let serialized_avs_operator_list = avs_operator_list.try_to_vec()?;
    create_account(
        admin.account(),
        avs_operator_list_account,
        system_program.account(),
        program_id,
        &rent,
        serialized_avs_operator_list.len() as u64,
        &avs_operator_list_seeds,
    )?;
    avs_operator_list_account.data.borrow_mut()[..serialized_avs_operator_list.len()]
        .copy_from_slice(&serialized_avs_operator_list);

    msg!(
        "Initializing AVS vault list @ address {}",
        avs_vault_list_account.key
    );
    let serialized_avs_vault_list = avs_vault_list.try_to_vec()?;
    create_account(
        admin.account(),
        avs_vault_list_account,
        system_program.account(),
        program_id,
        &rent,
        serialized_avs_vault_list.len() as u64,
        &avs_vault_list_seeds,
    )?;
    avs_vault_list_account.data.borrow_mut()[..serialized_avs_vault_list.len()]
        .copy_from_slice(&serialized_avs_vault_list);

    msg!(
        "Initializing AVS slasher list @ address {}",
        avs_slasher_list_account.key
    );
    let serialized_avs_slasher_list = avs_slasher_list.try_to_vec()?;
    create_account(
        admin.account(),
        avs_slasher_list_account,
        system_program.account(),
        program_id,
        &rent,
        serialized_avs_slasher_list.len() as u64,
        &avs_slasher_list_seeds,
    )?;
    avs_slasher_list_account.data.borrow_mut()[..serialized_avs_slasher_list.len()]
        .copy_from_slice(&serialized_avs_slasher_list);

    config.save()?;

    Ok(())
}
