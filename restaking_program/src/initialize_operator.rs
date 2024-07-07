use borsh::BorshSerialize;
use jito_restaking_core::{
    config::SanitizedConfig, operator::Operator, operator_avs_list::OperatorAvsList,
    operator_vault_list::OperatorVaultList,
};
use jito_restaking_sanitization::{
    assert_with_msg, create_account, empty_account::EmptyAccount, signer::SanitizedSignerAccount,
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
pub fn process_initialize_operator(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let SanitizedAccounts {
        mut config,
        operator_account,
        operator_avs_list_account,
        operator_vault_list_account,
        admin,
        base,
        system_program,
    } = SanitizedAccounts::sanitize(program_id, accounts)?;

    let rent = Rent::get()?;

    _create_operator(
        program_id,
        &config,
        &operator_account,
        &base,
        &admin,
        &system_program,
        &rent,
    )?;
    _create_operator_avs_list(
        program_id,
        &operator_account,
        &operator_avs_list_account,
        &admin,
        &system_program,
        &rent,
    )?;
    _create_operator_vault_list(
        program_id,
        &operator_account,
        &operator_vault_list_account,
        &admin,
        &system_program,
        &rent,
    )?;

    let num_operators = config.config_mut().increment_operators();
    assert_with_msg(
        num_operators.is_some(),
        ProgramError::InvalidAccountData,
        "Number of node operators has reached the maximum",
    )?;

    config.save()?;

    Ok(())
}

fn _create_operator_vault_list<'a, 'info>(
    program_id: &Pubkey,
    operator_account: &EmptyAccount<'a, 'info>,
    operator_vault_list_account: &EmptyAccount<'a, 'info>,
    admin: &SanitizedSignerAccount<'a, 'info>,
    system_program: &SanitizedSystemProgram<'a, 'info>,
    rent: &Rent,
) -> ProgramResult {
    let (
        expected_node_operator_vault_list_pubkey,
        node_operator_vault_list_bump,
        mut node_operator_vault_list_seeds,
    ) = OperatorVaultList::find_program_address(program_id, operator_account.account().key);
    node_operator_vault_list_seeds.push(vec![node_operator_vault_list_bump]);
    assert_with_msg(
        expected_node_operator_vault_list_pubkey == *operator_vault_list_account.account().key,
        ProgramError::InvalidAccountData,
        "Operator vault list account is not at the correct PDA",
    )?;

    let node_operator_vault_list = OperatorVaultList::new(
        *operator_account.account().key,
        node_operator_vault_list_bump,
    );

    let serialized_node_operator_vault_list = node_operator_vault_list.try_to_vec()?;
    create_account(
        admin.account(),
        operator_vault_list_account.account(),
        system_program.account(),
        program_id,
        rent,
        serialized_node_operator_vault_list.len() as u64,
        &node_operator_vault_list_seeds,
    )?;
    operator_vault_list_account.account().data.borrow_mut()
        [..serialized_node_operator_vault_list.len()]
        .copy_from_slice(&serialized_node_operator_vault_list);

    Ok(())
}

fn _create_operator_avs_list<'a, 'info>(
    program_id: &Pubkey,
    operator_account: &EmptyAccount<'a, 'info>,
    operator_avs_list_account: &EmptyAccount<'a, 'info>,
    admin: &SanitizedSignerAccount<'a, 'info>,
    system_program: &SanitizedSystemProgram<'a, 'info>,
    rent: &Rent,
) -> ProgramResult {
    let (
        expected_node_operator_avs_list_pubkey,
        node_operator_avs_list_bump,
        mut node_operator_avs_list_seeds,
    ) = OperatorAvsList::find_program_address(program_id, operator_account.account().key);
    node_operator_avs_list_seeds.push(vec![node_operator_avs_list_bump]);
    assert_with_msg(
        expected_node_operator_avs_list_pubkey == *operator_avs_list_account.account().key,
        ProgramError::InvalidAccountData,
        "Operator AVS list account is not at the correct PDA",
    )?;

    let operator_avs_list =
        OperatorAvsList::new(*operator_account.account().key, node_operator_avs_list_bump);

    let serialized_node_operator_avs_list = operator_avs_list.try_to_vec()?;
    create_account(
        admin.account(),
        operator_avs_list_account.account(),
        system_program.account(),
        program_id,
        rent,
        serialized_node_operator_avs_list.len() as u64,
        &node_operator_avs_list_seeds,
    )?;
    operator_avs_list_account.account().data.borrow_mut()
        [..serialized_node_operator_avs_list.len()]
        .copy_from_slice(&serialized_node_operator_avs_list);

    Ok(())
}

fn _create_operator<'a, 'info>(
    program_id: &Pubkey,
    config: &SanitizedConfig,
    operator_account: &EmptyAccount<'a, 'info>,
    base: &SanitizedSignerAccount<'a, 'info>,
    admin: &SanitizedSignerAccount<'a, 'info>,
    system_program: &SanitizedSystemProgram<'a, 'info>,
    rent: &Rent,
) -> ProgramResult {
    let (expected_operator_pubkey, operator_bump, mut operator_seeds) =
        Operator::find_program_address(program_id, base.account().key);
    operator_seeds.push(vec![operator_bump]);
    assert_with_msg(
        expected_operator_pubkey == *operator_account.account().key,
        ProgramError::InvalidAccountData,
        "Operator account is not at the correct PDA",
    )?;

    let operator = Operator::new(
        *base.account().key,
        *admin.account().key,
        *admin.account().key,
        config.config().operators_count(),
        operator_bump,
    );

    let serialized_operator = operator.try_to_vec()?;
    create_account(
        admin.account(),
        operator_account.account(),
        system_program.account(),
        program_id,
        rent,
        serialized_operator.len() as u64,
        &operator_seeds,
    )?;
    operator_account.account().data.borrow_mut()[..serialized_operator.len()]
        .copy_from_slice(&serialized_operator);

    Ok(())
}

struct SanitizedAccounts<'a, 'info> {
    config: SanitizedConfig<'a, 'info>,
    operator_account: EmptyAccount<'a, 'info>,
    operator_avs_list_account: EmptyAccount<'a, 'info>,
    operator_vault_list_account: EmptyAccount<'a, 'info>,
    admin: SanitizedSignerAccount<'a, 'info>,
    base: SanitizedSignerAccount<'a, 'info>,
    system_program: SanitizedSystemProgram<'a, 'info>,
}

impl<'a, 'info> SanitizedAccounts<'a, 'info> {
    fn sanitize(
        program_id: &Pubkey,
        accounts: &'a [AccountInfo<'info>],
    ) -> Result<SanitizedAccounts<'a, 'info>, ProgramError> {
        let accounts_iter = &mut accounts.iter();

        let config =
            SanitizedConfig::sanitize(program_id, next_account_info(accounts_iter)?, true)?;
        let operator_account = EmptyAccount::sanitize(next_account_info(accounts_iter)?, true)?;
        let operator_avs_list_account =
            EmptyAccount::sanitize(next_account_info(accounts_iter)?, true)?;
        let operator_vault_list_account =
            EmptyAccount::sanitize(next_account_info(accounts_iter)?, true)?;
        let admin = SanitizedSignerAccount::sanitize(next_account_info(accounts_iter)?, true)?;
        let base = SanitizedSignerAccount::sanitize(next_account_info(accounts_iter)?, false)?;
        let system_program = SanitizedSystemProgram::sanitize(next_account_info(accounts_iter)?)?;

        Ok(SanitizedAccounts {
            config,
            operator_account,
            operator_avs_list_account,
            operator_vault_list_account,
            admin,
            base,
            system_program,
        })
    }
}
