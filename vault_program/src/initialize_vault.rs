use borsh::BorshSerialize;
use jito_restaking_sanitization::{
    assert_with_msg, create_account, empty_account::EmptyAccount, signer::SanitizedSignerAccount,
    system_program::SanitizedSystemProgram, token_mint::SanitizedTokenMint,
    token_program::SanitizedTokenProgram,
};
use jito_vault_core::{
    config::SanitizedConfig, vault::Vault, vault_avs_list::VaultAvsList,
    vault_operator_list::VaultOperatorList, vault_slasher_list::VaultSlasherList,
};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::invoke,
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction,
    sysvar::Sysvar,
};
use spl_token::state::Mint;

/// Processes the create instruction: [`crate::VaultInstruction::InitializeVault`]
pub fn process_initialize_vault(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    deposit_fee_bps: u16,
    withdrawal_fee_bps: u16,
) -> ProgramResult {
    let mut accounts_iter = accounts.iter();

    let mut config_account =
        SanitizedConfig::sanitize(program_id, next_account_info(&mut accounts_iter)?, true)?;
    let vault_account = EmptyAccount::sanitize(next_account_info(&mut accounts_iter)?, true)?;
    let vault_avs_list_account =
        EmptyAccount::sanitize(next_account_info(&mut accounts_iter)?, true)?;
    let vault_operator_list_account =
        EmptyAccount::sanitize(next_account_info(&mut accounts_iter)?, true)?;
    let vault_slasher_list_account =
        EmptyAccount::sanitize(next_account_info(&mut accounts_iter)?, true)?;
    let lrt_mint = EmptyAccount::sanitize(next_account_info(&mut accounts_iter)?, true)?;
    let mint = SanitizedTokenMint::sanitize(next_account_info(&mut accounts_iter)?, false)?;
    let admin = SanitizedSignerAccount::sanitize(next_account_info(&mut accounts_iter)?, true)?;
    let base = SanitizedSignerAccount::sanitize(next_account_info(&mut accounts_iter)?, false)?;
    let system_program = SanitizedSystemProgram::sanitize(next_account_info(&mut accounts_iter)?)?;
    let token_program = SanitizedTokenProgram::sanitize(next_account_info(&mut accounts_iter)?)?;

    assert_with_msg(
        lrt_mint.account().is_signer,
        ProgramError::InvalidAccountData,
        "Mint account is not a signer",
    )?;

    let rent = Rent::get()?;

    msg!("Initializing mint @ address {}", lrt_mint.account().key);
    invoke(
        &system_instruction::create_account(
            admin.account().key,
            lrt_mint.account().key,
            rent.minimum_balance(Mint::get_packed_len()),
            Mint::get_packed_len() as u64,
            token_program.account().key,
        ),
        &[
            admin.account().clone(),
            lrt_mint.account().clone(),
            system_program.account().clone(),
        ],
    )?;

    invoke(
        &spl_token::instruction::initialize_mint2(
            &spl_token::id(),
            lrt_mint.account().key,
            vault_account.account().key,
            None,
            9,
        )?,
        &[lrt_mint.account().clone()],
    )?;

    let (vault_address, bump, mut vault_seeds) =
        Vault::find_program_address(program_id, base.account().key);
    vault_seeds.push(vec![bump]);
    assert_with_msg(
        vault_address == *vault_account.account().key,
        ProgramError::InvalidAccountData,
        "Vault account is not at the correct PDA",
    )?;
    let vault = Vault::new(
        *lrt_mint.account().key,
        *mint.account().key,
        *admin.account().key,
        config_account.config().vaults_count(),
        *base.account().key,
        deposit_fee_bps,
        withdrawal_fee_bps,
        bump,
    );

    let (vault_avs_list_address, bump, mut vault_avs_list_seeds) =
        VaultAvsList::find_program_address(program_id, vault_account.account().key);
    vault_avs_list_seeds.push(vec![bump]);
    assert_with_msg(
        vault_avs_list_address == *vault_avs_list_account.account().key,
        ProgramError::InvalidAccountData,
        "Vault AVS list account is not at the correct PDA",
    )?;
    let vault_avs_list = VaultAvsList::new(vault_address, bump);

    let (vault_operator_list_address, bump, mut vault_operator_list_seeds) =
        VaultOperatorList::find_program_address(program_id, vault_account.account().key);
    vault_operator_list_seeds.push(vec![bump]);
    assert_with_msg(
        vault_operator_list_address == *vault_operator_list_account.account().key,
        ProgramError::InvalidAccountData,
        "Vault operator list account is not at the correct PDA",
    )?;
    let vault_operator_list = VaultOperatorList::new(vault_address, bump);

    let (vault_slasher_list_address, bump, mut vault_slasher_list_seeds) =
        VaultSlasherList::find_program_address(program_id, vault_account.account().key);
    vault_slasher_list_seeds.push(vec![bump]);
    assert_with_msg(
        vault_slasher_list_address == *vault_slasher_list_account.account().key,
        ProgramError::InvalidAccountData,
        "Vault slasher list account is not at the correct PDA",
    )?;
    let vault_slasher_list = VaultSlasherList::new(vault_address, bump);

    let num_vaults = config_account.config_mut().increment_vaults();
    assert_with_msg(
        num_vaults.is_some(),
        ProgramError::InvalidAccountData,
        "Overflow when incrementing number of vaults",
    )?;

    config_account.save()?;

    let rent = Rent::get()?;

    msg!(
        "Initializing vault @ address {}",
        vault_account.account().key
    );
    let vault_serialized = vault.try_to_vec()?;
    create_account(
        admin.account(),
        vault_account.account(),
        system_program.account(),
        program_id,
        &rent,
        vault_serialized.len() as u64,
        &vault_seeds,
    )?;
    vault_account.account().try_borrow_mut_data()?[..vault_serialized.len()]
        .copy_from_slice(&vault_serialized);

    msg!(
        "Initializing vault AVS list @ address {}",
        vault_avs_list_account.account().key
    );
    let vault_avs_list_serialized = vault_avs_list.try_to_vec()?;
    create_account(
        admin.account(),
        vault_avs_list_account.account(),
        system_program.account(),
        program_id,
        &rent,
        vault_avs_list_serialized.len() as u64,
        &vault_avs_list_seeds,
    )?;
    vault_avs_list_account.account().try_borrow_mut_data()?[..vault_avs_list_serialized.len()]
        .copy_from_slice(&vault_avs_list_serialized);

    msg!(
        "Initializing vault operator list @ address {}",
        vault_operator_list_account.account().key
    );
    let vault_operator_list_serialized = vault_operator_list.try_to_vec()?;
    create_account(
        admin.account(),
        vault_operator_list_account.account(),
        system_program.account(),
        program_id,
        &rent,
        vault_operator_list_serialized.len() as u64,
        &vault_operator_list_seeds,
    )?;
    vault_operator_list_account
        .account()
        .try_borrow_mut_data()?[..vault_operator_list_serialized.len()]
        .copy_from_slice(&vault_operator_list_serialized);

    msg!(
        "Initializing vault slasher list @ address {}",
        vault_slasher_list_account.account().key
    );
    let vault_slasher_list_serialized = vault_slasher_list.try_to_vec()?;
    create_account(
        admin.account(),
        vault_slasher_list_account.account(),
        system_program.account(),
        program_id,
        &rent,
        vault_slasher_list_serialized.len() as u64,
        &vault_slasher_list_seeds,
    )?;
    vault_slasher_list_account.account().data.borrow_mut()[..vault_slasher_list_serialized.len()]
        .copy_from_slice(&vault_slasher_list_serialized);

    Ok(())
}
