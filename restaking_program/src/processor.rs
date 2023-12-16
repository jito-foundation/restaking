use borsh::BorshSerialize;
use jito_lrt_sdk::{add_avs, add_operator, remove_avs, remove_operator};
use jito_restaking_core::{
    avs::{
        Avs, AvsOperatorList, AvsSlasherList, AvsVaultList, SanitizedAvs, SanitizedAvsOperatorList,
        SanitizedAvsSlasherList, SanitizedAvsVaultList,
    },
    config::{Config, SanitizedConfig},
    node_operator::{
        NodeOperator, NodeOperatorAvsList, OperatorVaultList, SanitizedNodeOperator,
        SanitizedNodeOperatorAvsList, SanitizedNodeOperatorVaultList,
    },
};
use jito_restaking_sanitization::{
    assert_with_msg, create_account, signer::SanitizedSignerAccount,
    system_program::SanitizedSystemProgram,
};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::Clock,
    entrypoint::ProgramResult,
    msg,
    program::invoke_signed,
    program_error::ProgramError,
    pubkey::Pubkey,
    rent::Rent,
    sysvar::Sysvar,
};

pub struct RestakingProcessor;

impl RestakingProcessor {
    /// Initializes the global configuration for the restaking program
    /// [`crate::RestakingInstruction::InitializeConfig`]
    pub fn initialize_config(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
        let accounts_iter = &mut accounts.iter();

        let config_account = next_account_info(accounts_iter)?;
        assert_with_msg(
            config_account.is_writable,
            ProgramError::InvalidAccountData,
            "Config account must be writable",
        )?;
        assert_with_msg(
            config_account.data_is_empty(),
            ProgramError::InvalidAccountData,
            "Config account must be empty",
        )?;

        let admin = SanitizedSignerAccount::sanitize(next_account_info(accounts_iter)?, true)?;

        let vault_program = next_account_info(accounts_iter)?;

        let system_program = SanitizedSystemProgram::sanitize(next_account_info(accounts_iter)?)?;

        let (expected_config_key, bump, mut config_seeds) =
            Config::find_program_address(program_id);
        config_seeds.push(vec![bump]);
        assert_with_msg(
            expected_config_key == *config_account.key,
            ProgramError::InvalidAccountData,
            "Config account is not at the correct PDA",
        )?;

        let config = Config::new(*admin.account().key, *vault_program.key, bump);

        msg!("Initializing config @ address {}", config_account.key);
        let config_serialized = config.try_to_vec()?;
        create_account(
            admin.account(),
            config_account,
            system_program.account(),
            program_id,
            &Rent::get()?,
            config_serialized.len() as u64,
            &config_seeds,
        )?;
        config_account.data.borrow_mut()[..config_serialized.len()]
            .copy_from_slice(&config_serialized);

        Ok(())
    }

    /// Initializes an AVS and associated accounts
    /// [`crate::RestakingInstruction::InitializeAvs`]
    pub fn initialize_avs(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
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

        let (
            expected_avs_operator_list_pubkey,
            avs_operator_list_bump,
            mut avs_operator_list_seeds,
        ) = AvsOperatorList::find_program_address(program_id, avs_account.key);
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

    /// Initializes a node operator and associated accounts.
    ///
    /// [`crate::RestakingInstruction::InitializeOperator`]
    pub fn initialize_node_operator(
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
        node_operator_vault_list_account.data.borrow_mut()
            [..serialized_node_operator_vault_list.len()]
            .copy_from_slice(&serialized_node_operator_vault_list);

        Ok(())
    }

    /// The AVS admin can add support for receiving delegation from a vault.
    /// The vault can be used at the end of epoch + 1.
    /// This method is permissioned to the AVS admin.
    ///
    /// [`crate::RestakingInstruction::AvsAddVault`]
    pub fn avs_add_vault(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
        let accounts_iter = &mut accounts.iter();

        let config =
            SanitizedConfig::sanitize(program_id, next_account_info(accounts_iter)?, false)?;

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
            &add_avs(
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

        let clock = Clock::get()?;

        assert_with_msg(
            avs_vault_list.avs_vault_list_mut().add_vault(
                *vault.key,
                clock.slot,
                config.config().epoch_duration(),
            ),
            ProgramError::InvalidAccountData,
            "Vault already exists in AVS vault list",
        )?;

        avs.save()?;
        avs_vault_list.save(&Rent::get()?, admin.account())?;

        Ok(())
    }

    /// The AVS admin can remove support for receiving delegation from a vault.
    /// The vault is removed at the end of epoch + 1.
    /// This method is permissioned to the AVS admin.
    ///
    /// [`crate::RestakingInstruction::AvsRemoveVault`]
    pub fn avs_remove_vault(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
        let accounts_iter = &mut accounts.iter();

        let config =
            SanitizedConfig::sanitize(program_id, next_account_info(accounts_iter)?, false)?;

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

        let clock = Clock::get()?;

        assert_with_msg(
            avs_vault_list.avs_vault_list_mut().remove_vault(
                *vault.key,
                clock.slot,
                config.config().epoch_duration(),
            ),
            ProgramError::InvalidAccountData,
            "Vault already exists in AVS vault list",
        )?;

        avs.save()?;
        avs_vault_list.save(&Rent::get()?, admin.account())?;

        Ok(())
    }

    /// The node operator admin can add support for receiving delegation from a vault.
    /// The vault can be used at the end of epoch + 1.
    /// This method is permissioned to the node operator admin.
    ///
    /// [`crate::RestakingInstruction::OperatorAddVault`]
    pub fn node_operator_add_vault(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
        let accounts_iter = &mut accounts.iter();

        let config =
            SanitizedConfig::sanitize(program_id, next_account_info(accounts_iter)?, false)?;

        let node_operator =
            SanitizedNodeOperator::sanitize(program_id, next_account_info(accounts_iter)?, true)?;

        let mut node_operator_vault_list = SanitizedNodeOperatorVaultList::sanitize(
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
            node_operator.node_operator().admin() == *admin.account().key,
            ProgramError::InvalidAccountData,
            "Admin is not the node operator admin",
        )?;

        let mut config_seeds = Config::seeds();
        config_seeds.push(vec![config.config().bump()]);
        let config_seeds_slice = config_seeds
            .iter()
            .map(|seed| seed.as_slice())
            .collect::<Vec<&[u8]>>();

        let mut node_operator_seeds = NodeOperator::seeds(&node_operator.node_operator().base());
        node_operator_seeds.push(vec![node_operator.node_operator().bump()]);
        let node_operator_seeds_slice = node_operator_seeds
            .iter()
            .map(|seed| seed.as_slice())
            .collect::<Vec<&[u8]>>();

        invoke_signed(
            &add_operator(
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
                .node_operator_vault_list_mut()
                .add_vault(*vault.key, clock.slot, config.config().epoch_duration(), 0),
            ProgramError::InvalidAccountData,
            "Vault already exists in operator vault list",
        )?;

        node_operator.save()?;
        node_operator_vault_list.save(&Rent::get()?, admin.account())?;

        Ok(())
    }

    /// The node operator admin can remove support for receiving delegation from a vault.
    /// The vault can be used at the end of epoch + 1.
    /// This method is permissioned to the node operator admin.
    /// [`crate::RestakingInstruction::OperatorRemoveVault`]
    pub fn node_operator_remove_vault(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
    ) -> ProgramResult {
        let accounts_iter = &mut accounts.iter();

        let config =
            SanitizedConfig::sanitize(program_id, next_account_info(accounts_iter)?, false)?;

        let node_operator =
            SanitizedNodeOperator::sanitize(program_id, next_account_info(accounts_iter)?, true)?;

        let mut node_operator_vault_list = SanitizedNodeOperatorVaultList::sanitize(
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
            node_operator.node_operator().admin() == *admin.account().key,
            ProgramError::InvalidAccountData,
            "Admin is not the node operator admin",
        )?;

        let mut config_seeds = Config::seeds();
        config_seeds.push(vec![config.config().bump()]);
        let config_seeds_slice = config_seeds
            .iter()
            .map(|seed| seed.as_slice())
            .collect::<Vec<&[u8]>>();

        let mut node_operator_seeds = NodeOperator::seeds(&node_operator.node_operator().base());
        node_operator_seeds.push(vec![node_operator.node_operator().bump()]);
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
                .node_operator_vault_list_mut()
                .remove_vault(*vault.key, clock.slot, config.config().epoch_duration()),
            ProgramError::InvalidAccountData,
            "Vault already exists in operator vault list",
        )?;

        node_operator.save()?;
        node_operator_vault_list.save(&Rent::get()?, admin.account())?;

        Ok(())
    }

    /// The node operator admin can add support for running an AVS.
    /// This method is permissioned to the node operator admin.
    ///
    /// [`crate::RestakingInstruction::OperatorAddAvs`]
    pub fn node_operator_add_avs(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
        let accounts_iter = &mut accounts.iter();

        let config =
            SanitizedConfig::sanitize(program_id, next_account_info(accounts_iter)?, false)?;
        let node_operator =
            SanitizedNodeOperator::sanitize(program_id, next_account_info(accounts_iter)?, true)?;
        let mut node_operator_avs_list = SanitizedNodeOperatorAvsList::sanitize(
            program_id,
            next_account_info(accounts_iter)?,
            true,
            node_operator.account().key,
        )?;

        let avs = SanitizedAvs::sanitize(program_id, next_account_info(accounts_iter)?, false)?;

        let admin = SanitizedSignerAccount::sanitize(next_account_info(accounts_iter)?, true)?;
        assert_with_msg(
            node_operator.node_operator().admin() == *admin.account().key,
            ProgramError::InvalidAccountData,
            "Admin is not the node operator admin",
        )?;

        let clock = Clock::get()?;

        assert_with_msg(
            node_operator_avs_list.node_operator_avs_list_mut().add_avs(
                *avs.account().key,
                clock.slot,
                config.config().epoch_duration(),
            ),
            ProgramError::InvalidAccountData,
            "AVS already exists in node operator AVS list",
        )?;

        // TODO (LB): notify

        // TODO (LB): handle re-alloc
        node_operator_avs_list.save(&Rent::get()?, admin.account())?;

        Ok(())
    }

    /// The node operator admin can remove support for running an AVS.
    /// This method is permissioned to the node operator admin.
    ///
    /// [`crate::RestakingInstruction::OperatorRemoveAvs`]
    pub fn node_operator_remove_avs(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
    ) -> ProgramResult {
        let accounts_iter = &mut accounts.iter();

        let config =
            SanitizedConfig::sanitize(program_id, next_account_info(accounts_iter)?, false)?;
        let node_operator =
            SanitizedNodeOperator::sanitize(program_id, next_account_info(accounts_iter)?, true)?;
        let mut node_operator_avs_list = SanitizedNodeOperatorAvsList::sanitize(
            program_id,
            next_account_info(accounts_iter)?,
            true,
            node_operator.account().key,
        )?;

        let avs = SanitizedAvs::sanitize(program_id, next_account_info(accounts_iter)?, false)?;

        let admin = SanitizedSignerAccount::sanitize(next_account_info(accounts_iter)?, true)?;
        assert_with_msg(
            node_operator.node_operator().admin() == *admin.account().key,
            ProgramError::InvalidAccountData,
            "Admin is not the node operator admin",
        )?;

        let clock = Clock::get()?;

        assert_with_msg(
            node_operator_avs_list
                .node_operator_avs_list_mut()
                .remove_avs(
                    *avs.account().key,
                    clock.slot,
                    config.config().epoch_duration(),
                ),
            ProgramError::InvalidAccountData,
            "AVS already exists in node operator AVS list",
        )?;

        // TODO (LB): notify

        // TODO (LB): handle re-alloc
        node_operator_avs_list.save(&Rent::get()?, admin.account())?;

        Ok(())
    }

    /// The node operator admin can set a new admin for the node operator.
    /// This method is permissioned to the node operator admin and both the old and new admins must sign.
    ///
    /// [`crate::RestakingInstruction::OperatorSetAdmin`]
    pub fn set_node_operator_admin(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
        let accounts_iter = &mut accounts.iter();

        let mut node_operator =
            SanitizedNodeOperator::sanitize(program_id, next_account_info(accounts_iter)?, true)?;
        let old_admin = SanitizedSignerAccount::sanitize(next_account_info(accounts_iter)?, false)?;
        let new_admin = SanitizedSignerAccount::sanitize(next_account_info(accounts_iter)?, true)?;

        assert_with_msg(
            node_operator.node_operator().admin() == *old_admin.account().key,
            ProgramError::InvalidAccountData,
            "Old admin is not the node operator admin",
        )?;

        node_operator
            .node_operator_mut()
            .set_admin(*new_admin.account().key);

        node_operator.save()?;

        Ok(())
    }

    /// The node operator admin can set a new voter for the node operator.
    /// This method is permissioned to the node operator admin.
    ///
    /// [`crate::RestakingInstruction::OperatorSetVoter`]
    pub fn set_node_operator_voter(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
        let accounts_iter = &mut accounts.iter();

        let mut node_operator =
            SanitizedNodeOperator::sanitize(program_id, next_account_info(accounts_iter)?, true)?;
        let admin = SanitizedSignerAccount::sanitize(next_account_info(accounts_iter)?, true)?;
        let voter = next_account_info(accounts_iter)?;

        assert_with_msg(
            node_operator.node_operator().admin() == *admin.account().key,
            ProgramError::InvalidAccountData,
            "Admin is not the node operator admin",
        )?;

        node_operator.node_operator_mut().set_voter(*voter.key);

        node_operator.save()?;

        Ok(())
    }

    /// The AVS admin can add a node operator to the AVS after the node operator has opted-in to the network.
    /// This method is permissioned to the AVS admin.
    /// [`crate::RestakingInstruction::AvsAddNodeOperator`]
    pub fn avs_add_node_operator(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
        let accounts_iter = &mut accounts.iter();

        let config =
            SanitizedConfig::sanitize(program_id, next_account_info(accounts_iter)?, false)?;
        let avs = SanitizedAvs::sanitize(program_id, next_account_info(accounts_iter)?, false)?;
        let mut avs_operator_list = SanitizedAvsOperatorList::sanitize(
            program_id,
            next_account_info(accounts_iter)?,
            true,
            avs.account().key,
        )?;
        let node_operator =
            SanitizedNodeOperator::sanitize(program_id, next_account_info(accounts_iter)?, false)?;
        let node_operator_avs_list = SanitizedNodeOperatorAvsList::sanitize(
            program_id,
            next_account_info(accounts_iter)?,
            false,
            node_operator.account().key,
        )?;

        let admin = SanitizedSignerAccount::sanitize(next_account_info(accounts_iter)?, true)?;

        assert_with_msg(
            avs.avs().admin() == *admin.account().key,
            ProgramError::InvalidAccountData,
            "Admin is not the AVS admin",
        )?;

        let clock = Clock::get()?;

        assert_with_msg(
            node_operator_avs_list
                .node_operator_avs_list()
                .contains_active_avs(
                    avs.account().key,
                    clock.slot,
                    config.config().epoch_duration(),
                ),
            ProgramError::InvalidAccountData,
            "Node operator does not have AVS in AVS list",
        )?;

        let clock = Clock::get()?;
        assert_with_msg(
            avs_operator_list.avs_operator_list_mut().add_operator(
                *node_operator.account().key,
                clock.slot,
                config.config().epoch_duration(),
            ),
            ProgramError::InvalidAccountData,
            "Node operator already exists in AVS operator list",
        )?;

        // TODO (LB): notify

        // TODO (LB): handle re-alloc
        avs_operator_list.save(&Rent::get()?, admin.account())?;

        Ok(())
    }

    /// The AVS admin can remove a node operator from the AVS.
    /// This method is permissioned to the AVS admin.
    /// [`crate::RestakingInstruction::AvsRemoveNodeOperator`]
    pub fn avs_remove_node_operator(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
    ) -> ProgramResult {
        let accounts_iter = &mut accounts.iter();

        let config =
            SanitizedConfig::sanitize(program_id, next_account_info(accounts_iter)?, false)?;
        let avs = SanitizedAvs::sanitize(program_id, next_account_info(accounts_iter)?, false)?;
        let mut avs_operator_list = SanitizedAvsOperatorList::sanitize(
            program_id,
            next_account_info(accounts_iter)?,
            true,
            avs.account().key,
        )?;
        let node_operator =
            SanitizedNodeOperator::sanitize(program_id, next_account_info(accounts_iter)?, false)?;

        let admin = SanitizedSignerAccount::sanitize(next_account_info(accounts_iter)?, true)?;

        assert_with_msg(
            avs.avs().admin() == *admin.account().key,
            ProgramError::InvalidAccountData,
            "Admin is not the AVS admin",
        )?;

        let clock = Clock::get()?;
        assert_with_msg(
            avs_operator_list.avs_operator_list_mut().remove_operator(
                *node_operator.account().key,
                clock.slot,
                config.config().epoch_duration(),
            ),
            ProgramError::InvalidAccountData,
            "Node operator does not exist in AVS operator list",
        )?;

        avs_operator_list.save(&Rent::get()?, admin.account())?;

        Ok(())
    }

    pub fn avs_add_vault_slasher(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        max_slashable_per_epoch: u64,
    ) -> ProgramResult {
        let accounts_iter = &mut accounts.iter();

        let _config =
            SanitizedConfig::sanitize(program_id, next_account_info(accounts_iter)?, false)?;
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
        let payer = SanitizedSignerAccount::sanitize(next_account_info(accounts_iter)?, true)?;
        let _system_program = SanitizedSystemProgram::sanitize(next_account_info(accounts_iter)?)?;

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
            avs_slasher_list.avs_slasher_list_mut().add_slasher(
                *vault.key,
                *slasher.key,
                clock.slot,
                max_slashable_per_epoch,
            ),
            ProgramError::InvalidAccountData,
            "Slasher, vault combination already exists in AVS slasher list",
        )?;

        avs_slasher_list.save(&Rent::get()?, payer.account())?;

        Ok(())
    }

    pub fn avs_deprecate_slasher(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
        let accounts_iter = &mut accounts.iter();

        let _config =
            SanitizedConfig::sanitize(program_id, next_account_info(accounts_iter)?, false)?;
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
        let payer = SanitizedSignerAccount::sanitize(next_account_info(accounts_iter)?, true)?;
        let _system_program = SanitizedSystemProgram::sanitize(next_account_info(accounts_iter)?)?;

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

        avs_slasher_list.save(&Rent::get()?, payer.account())?;

        Ok(())
    }
}
