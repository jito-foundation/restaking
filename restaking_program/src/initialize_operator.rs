use borsh::BorshSerialize;
use jito_restaking_core::{
    config::SanitizedConfig,
    node_operator::{NodeOperator, NodeOperatorAvsList, OperatorVaultList},
};
use jito_restaking_sanitization::{
    assert_with_msg, create_account, signer::SanitizedSignerAccount,
    system_program::SanitizedSystemProgram,
};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program_error::ProgramError,
    pubkey::Pubkey,
    rent::Rent,
    sysvar::Sysvar,
};

/// Initializes a node operator and associated accounts.
///
/// [`crate::RestakingInstruction::InitializeOperator`]
pub fn process_initialize_node_operator(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    let config_account = next_account_info(accounts_iter)?;
    let mut config = SanitizedConfig::sanitize(program_id, config_account, true)?;

    let node_operator_account = next_account_info(accounts_iter)?;
    assert_with_msg(
        node_operator_account.is_writable,
        ProgramError::InvalidAccountData,
        "Node operator account must be writable",
    )?;
    assert_with_msg(
        node_operator_account.data_is_empty(),
        ProgramError::InvalidAccountData,
        "Node operator account must be empty",
    )?;

    let node_operator_avs_list_account = next_account_info(accounts_iter)?;
    assert_with_msg(
        node_operator_avs_list_account.is_writable,
        ProgramError::InvalidAccountData,
        "Node operator AVS list account must be writable",
    )?;
    assert_with_msg(
        node_operator_avs_list_account.data_is_empty(),
        ProgramError::InvalidAccountData,
        "Node operator AVS list account must be empty",
    )?;

    let node_operator_vault_list_account = next_account_info(accounts_iter)?;
    assert_with_msg(
        node_operator_vault_list_account.is_writable,
        ProgramError::InvalidAccountData,
        "Node operator vault list account must be writable",
    )?;
    assert_with_msg(
        node_operator_vault_list_account.data_is_empty(),
        ProgramError::InvalidAccountData,
        "Node operator vault list account must be empty",
    )?;

    let admin = SanitizedSignerAccount::sanitize(next_account_info(accounts_iter)?, true)?;
    let base = SanitizedSignerAccount::sanitize(next_account_info(accounts_iter)?, false)?;

    let system_program = SanitizedSystemProgram::sanitize(next_account_info(accounts_iter)?)?;

    let (expected_node_operator_pubkey, node_operator_bump, mut node_operator_seeds) =
        NodeOperator::find_program_address(program_id, base.account().key);
    node_operator_seeds.push(vec![node_operator_bump]);
    assert_with_msg(
        expected_node_operator_pubkey == *node_operator_account.key,
        ProgramError::InvalidAccountData,
        "Node operator account is not at the correct PDA",
    )?;

    let (
        expected_node_operator_avs_list_pubkey,
        node_operator_avs_list_bump,
        mut node_operator_avs_list_seeds,
    ) = NodeOperatorAvsList::find_program_address(program_id, node_operator_account.key);
    node_operator_avs_list_seeds.push(vec![node_operator_avs_list_bump]);
    assert_with_msg(
        expected_node_operator_avs_list_pubkey == *node_operator_avs_list_account.key,
        ProgramError::InvalidAccountData,
        "Node operator AVS list account is not at the correct PDA",
    )?;

    let (
        expected_node_operator_vault_list_pubkey,
        node_operator_vault_list_bump,
        mut node_operator_vault_list_seeds,
    ) = OperatorVaultList::find_program_address(program_id, node_operator_account.key);
    node_operator_vault_list_seeds.push(vec![node_operator_vault_list_bump]);
    assert_with_msg(
        expected_node_operator_vault_list_pubkey == *node_operator_vault_list_account.key,
        ProgramError::InvalidAccountData,
        "Node operator vault list account is not at the correct PDA",
    )?;

    let node_operator = NodeOperator::new(
        *base.account().key,
        *admin.account().key,
        *admin.account().key,
        config.config().operators_count(),
        node_operator_bump,
    );

    let node_operator_avs_list =
        NodeOperatorAvsList::new(*node_operator_account.key, node_operator_avs_list_bump);

    let node_operator_vault_list =
        OperatorVaultList::new(*node_operator_account.key, node_operator_vault_list_bump);

    let num_operators = config.config_mut().increment_operators();
    assert_with_msg(
        num_operators.is_some(),
        ProgramError::InvalidAccountData,
        "Number of node operators has reached the maximum",
    )?;

    config.save()?;

    let rent = Rent::get()?;

    let serialized_node_operator = node_operator.try_to_vec()?;
    create_account(
        admin.account(),
        node_operator_account,
        system_program.account(),
        program_id,
        &rent,
        serialized_node_operator.len() as u64,
        &node_operator_seeds,
    )?;
    node_operator_account.data.borrow_mut()[..serialized_node_operator.len()]
        .copy_from_slice(&serialized_node_operator);

    let serialized_node_operator_avs_list = node_operator_avs_list.try_to_vec()?;
    create_account(
        admin.account(),
        node_operator_avs_list_account,
        system_program.account(),
        program_id,
        &rent,
        serialized_node_operator_avs_list.len() as u64,
        &node_operator_avs_list_seeds,
    )?;
    node_operator_avs_list_account.data.borrow_mut()[..serialized_node_operator_avs_list.len()]
        .copy_from_slice(&serialized_node_operator_avs_list);

    let serialized_node_operator_vault_list = node_operator_vault_list.try_to_vec()?;
    create_account(
        admin.account(),
        node_operator_vault_list_account,
        system_program.account(),
        program_id,
        &rent,
        serialized_node_operator_vault_list.len() as u64,
        &node_operator_vault_list_seeds,
    )?;
    node_operator_vault_list_account.data.borrow_mut()[..serialized_node_operator_vault_list.len()]
        .copy_from_slice(&serialized_node_operator_vault_list);

    Ok(())
}
