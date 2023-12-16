use borsh::BorshSerialize;
use jito_lrt_core::{
    config::{Config, SanitizedConfig},
    vault::{SanitizedVault, Vault},
    vault_avs_list::{SanitizedVaultAvsList, VaultAvsList},
    vault_operator_list::{SanitizedVaultOperatorList, VaultOperatorList},
};
use jito_restaking_sanitization::{
    assert_with_msg, associated_token_account::SanitizedAssociatedTokenAccount, create_account,
    signer::SanitizedSignerAccount, system_program::SanitizedSystemProgram,
    token_mint::SanitizedTokenMint, token_program::SanitizedTokenProgram,
};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::Clock,
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction,
    sysvar::Sysvar,
};
use spl_token::state::Mint;

pub struct LrtProcessor;

impl LrtProcessor {
    pub fn initialize_config(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
        let mut accounts_iter = accounts.iter();

        let config_account = next_account_info(&mut accounts_iter)?;
        assert_with_msg(
            config_account.data_is_empty(),
            ProgramError::AccountAlreadyInitialized,
            "Config account already initialized",
        )?;
        assert_with_msg(
            config_account.is_writable,
            ProgramError::InvalidAccountData,
            "Config account is not writable",
        )?;

        let admin = SanitizedSignerAccount::sanitize(next_account_info(&mut accounts_iter)?, true)?;

        let restaking_program_signer = next_account_info(&mut accounts_iter)?;

        let system_program =
            SanitizedSystemProgram::sanitize(next_account_info(&mut accounts_iter)?)?;

        let (config_address, bump, mut config_seeds) = Config::find_program_address(program_id);
        config_seeds.push(vec![bump]);
        assert_with_msg(
            config_address == *config_account.key,
            ProgramError::InvalidAccountData,
            "Config account is not at the correct PDA",
        )?;

        let config = Config::new(*admin.account().key, *restaking_program_signer.key, bump);
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

    /// Processes the create instruction: [`crate::LrtInstruction::InitializeVault`]
    pub(crate) fn initialize_vault(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
        let mut accounts_iter = accounts.iter();

        let mut config_account =
            SanitizedConfig::sanitize(program_id, next_account_info(&mut accounts_iter)?, true)?;

        let vault_account = next_account_info(&mut accounts_iter)?;
        assert_with_msg(
            vault_account.data_is_empty(),
            ProgramError::AccountAlreadyInitialized,
            "LRT account already initialized",
        )?;
        assert_with_msg(
            vault_account.is_writable,
            ProgramError::InvalidAccountData,
            "LRT account is not writable",
        )?;

        let vault_avs_list_account = next_account_info(&mut accounts_iter)?;
        assert_with_msg(
            vault_avs_list_account.data_is_empty(),
            ProgramError::AccountAlreadyInitialized,
            "Vault AVS list account already initialized",
        )?;
        assert_with_msg(
            vault_avs_list_account.is_writable,
            ProgramError::InvalidAccountData,
            "Vault AVS list account is not writable",
        )?;

        let vault_operator_list_account = next_account_info(&mut accounts_iter)?;
        assert_with_msg(
            vault_operator_list_account.data_is_empty(),
            ProgramError::AccountAlreadyInitialized,
            "Vault operator list account already initialized",
        )?;
        assert_with_msg(
            vault_operator_list_account.is_writable,
            ProgramError::InvalidAccountData,
            "Vault operator list account is not writable",
        )?;

        let lrt_mint = next_account_info(&mut accounts_iter)?;
        assert_with_msg(
            lrt_mint.is_writable,
            ProgramError::InvalidAccountData,
            "Mint account is not writable",
        )?;
        assert_with_msg(
            lrt_mint.is_signer,
            ProgramError::InvalidAccountData,
            "Mint account is not a signer",
        )?;
        assert_with_msg(
            lrt_mint.data_is_empty(),
            ProgramError::InvalidAccountData,
            "Mint account already initialized",
        )?;

        let mint = SanitizedTokenMint::sanitize(next_account_info(&mut accounts_iter)?)?;

        let admin = SanitizedSignerAccount::sanitize(next_account_info(&mut accounts_iter)?, true)?;
        let base = SanitizedSignerAccount::sanitize(next_account_info(&mut accounts_iter)?, false)?;

        let system_program =
            SanitizedSystemProgram::sanitize(next_account_info(&mut accounts_iter)?)?;
        let token_program =
            SanitizedTokenProgram::sanitize(next_account_info(&mut accounts_iter)?)?;

        let rent = Rent::get()?;

        msg!("Initializing mint @ address {}", lrt_mint.key);
        invoke(
            &system_instruction::create_account(
                admin.account().key,
                lrt_mint.key,
                rent.minimum_balance(Mint::get_packed_len()),
                Mint::get_packed_len() as u64,
                token_program.account().key,
            ),
            &[
                admin.account().clone(),
                lrt_mint.clone(),
                system_program.account().clone(),
            ],
        )?;

        invoke(
            &spl_token::instruction::initialize_mint2(
                &spl_token::id(),
                lrt_mint.key,
                vault_account.key,
                None,
                9,
            )?,
            &[lrt_mint.clone()],
        )?;

        let (vault_address, bump, mut vault_seeds) =
            Vault::find_program_address(program_id, base.account().key);
        vault_seeds.push(vec![bump]);
        assert_with_msg(
            vault_address == *vault_account.key,
            ProgramError::InvalidAccountData,
            "Vault account is not at the correct PDA",
        )?;
        let vault = Vault::new(
            *lrt_mint.key,
            *mint.account().key,
            *admin.account().key,
            config_account.config().vaults_count(),
            *base.account().key,
            bump,
        );

        let (vault_avs_list_address, bump, mut vault_avs_list_seeds) =
            VaultAvsList::find_program_address(program_id, vault_account.key);
        vault_avs_list_seeds.push(vec![bump]);
        assert_with_msg(
            vault_avs_list_address == *vault_avs_list_account.key,
            ProgramError::InvalidAccountData,
            "Vault AVS list account is not at the correct PDA",
        )?;
        let vault_avs_list = VaultAvsList::new(vault_address, bump);

        let (vault_operator_list_address, bump, mut vault_operator_list_seeds) =
            VaultOperatorList::find_program_address(program_id, vault_account.key);
        vault_operator_list_seeds.push(vec![bump]);
        assert_with_msg(
            vault_operator_list_address == *vault_operator_list_account.key,
            ProgramError::InvalidAccountData,
            "Vault operator list account is not at the correct PDA",
        )?;
        let vault_operator_list = VaultOperatorList::new(vault_address, bump);

        let num_vaults = config_account.config_mut().increment_vaults();
        assert_with_msg(
            num_vaults.is_some(),
            ProgramError::InvalidAccountData,
            "Overflow when incrementing number of vaults",
        )?;

        config_account.save()?;

        let rent = Rent::get()?;

        msg!("Initializing vault @ address {}", vault_account.key);
        let vault_serialized = vault.try_to_vec()?;
        create_account(
            admin.account(),
            vault_account,
            system_program.account(),
            program_id,
            &rent,
            vault_serialized.len() as u64,
            &vault_seeds,
        )?;
        vault_account.data.borrow_mut()[..vault_serialized.len()]
            .copy_from_slice(&vault_serialized);

        msg!(
            "Initializing vault AVS list @ address {}",
            vault_avs_list_account.key
        );
        let vault_avs_list_serialized = vault_avs_list.try_to_vec()?;
        create_account(
            admin.account(),
            vault_avs_list_account,
            system_program.account(),
            program_id,
            &rent,
            vault_avs_list_serialized.len() as u64,
            &vault_avs_list_seeds,
        )?;
        vault_avs_list_account.data.borrow_mut()[..vault_avs_list_serialized.len()]
            .copy_from_slice(&vault_avs_list_serialized);

        msg!(
            "Initializing vault operator list @ address {}",
            vault_operator_list_account.key
        );
        let vault_operator_list_serialized = vault_operator_list.try_to_vec()?;
        create_account(
            admin.account(),
            vault_operator_list_account,
            system_program.account(),
            program_id,
            &rent,
            vault_operator_list_serialized.len() as u64,
            &vault_operator_list_seeds,
        )?;
        vault_operator_list_account.data.borrow_mut()[..vault_operator_list_serialized.len()]
            .copy_from_slice(&vault_operator_list_serialized);

        Ok(())
    }

    pub fn set_capacity(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        capacity: u64,
    ) -> ProgramResult {
        let mut accounts_iter = accounts.iter();

        let mut lrt_account =
            SanitizedVault::sanitize(program_id, next_account_info(&mut accounts_iter)?, true)?;

        let admin =
            SanitizedSignerAccount::sanitize(next_account_info(&mut accounts_iter)?, false)?;

        assert_with_msg(
            *admin.account().key == lrt_account.vault().admin(),
            ProgramError::InvalidAccountData,
            "Admin account does not match LRT admin",
        )?;

        lrt_account.vault_mut().set_capacity(capacity);
        lrt_account.save()?;

        Ok(())
    }

    /// Processes the mint instruction: [`crate::LrtInstruction::MintTo`]
    pub(crate) fn mint(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        amount: u64,
    ) -> ProgramResult {
        let mut accounts_iter = &mut accounts.iter();

        let mut vault =
            SanitizedVault::sanitize(program_id, next_account_info(&mut accounts_iter)?, true)?;
        let mut lrt_mint = SanitizedTokenMint::sanitize(next_account_info(&mut accounts_iter)?)?;
        assert_with_msg(
            lrt_mint.account().is_writable,
            ProgramError::InvalidAccountData,
            "Mint account is not writable",
        )?;
        assert_with_msg(
            *lrt_mint.account().key == vault.vault().lrt_mint(),
            ProgramError::InvalidAccountData,
            "Mint account does not match LRT mint",
        )?;
        let source_owner =
            SanitizedSignerAccount::sanitize(next_account_info(&mut accounts_iter)?, true)?;
        let source_token_account = SanitizedAssociatedTokenAccount::sanitize(
            next_account_info(&mut accounts_iter)?,
            &vault.vault().supported_mint(),
            source_owner.account().key,
        )?;
        let mut dest_token_account = SanitizedAssociatedTokenAccount::sanitize(
            next_account_info(&mut accounts_iter)?,
            &vault.vault().supported_mint(),
            vault.account().key,
        )?;
        let lrt_receiver = SanitizedAssociatedTokenAccount::sanitize(
            next_account_info(&mut accounts_iter)?,
            &vault.vault().lrt_mint(),
            source_owner.account().key,
        )?;
        let _token_program =
            SanitizedTokenProgram::sanitize(next_account_info(&mut accounts_iter)?)?;
        if let Some(vault_mint_signer) = vault.vault().mint_burn_authority() {
            let mint_signer =
                SanitizedSignerAccount::sanitize(next_account_info(&mut accounts_iter)?, false)?;
            assert_with_msg(
                *mint_signer.account().key == vault_mint_signer,
                ProgramError::InvalidAccountData,
                "Mint signer does not match vault mint signer",
            )?;
        }

        // check capacity
        let amount_after_deposit = amount.checked_add(dest_token_account.token_account().amount);
        assert_with_msg(
            amount_after_deposit.is_some(),
            ProgramError::InvalidArgument,
            "Overflow when adding amount to destination token account",
        )?;
        let amount_after_deposit = amount_after_deposit.unwrap();
        assert_with_msg(
            vault.vault().capacity() <= amount_after_deposit,
            ProgramError::InvalidArgument,
            "Amount exceeds vault capacity",
        )?;

        // transfer the amount from the source token account to the destination token account
        invoke(
            &spl_token::instruction::transfer(
                &spl_token::id(),
                source_token_account.account().key,
                dest_token_account.account().key,
                source_owner.account().key,
                &[],
                amount,
            )?,
            &[
                source_token_account.account().clone(),
                dest_token_account.account().clone(),
                source_owner.account().clone(),
            ],
        )?;

        let (_, bump, mut seeds) = Vault::find_program_address(program_id, &vault.vault().base());
        seeds.push(vec![bump]);
        let seed_slices: Vec<&[u8]> = seeds.iter().map(|seed| seed.as_slice()).collect();

        // mint the amount to the LRT receiver in a 1:1 ratio
        invoke_signed(
            &spl_token::instruction::mint_to(
                &spl_token::id(),
                lrt_mint.account().key,
                lrt_receiver.account().key,
                vault.account().key,
                &[],
                amount,
            )?,
            &[
                lrt_mint.account().clone(),
                lrt_receiver.account().clone(),
                vault.account().clone(),
            ],
            &[&seed_slices],
        )?;

        // need to reload after CPI
        lrt_mint.reload()?;
        dest_token_account.reload()?;

        // TODO (LB): should do this incrementally or refresh based?
        vault
            .vault_mut()
            .set_tokens_deposited(dest_token_account.token_account().amount);
        vault.vault_mut().set_lrt_supply(lrt_mint.mint().supply);

        vault.save()?;

        Ok(())
    }

    /// Processes the vault add AVS instruction: [`crate::LrtInstruction::AddAvs`]
    pub fn vault_add_avs(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
        let mut accounts_iter = accounts.iter();

        let restaking_program_signer =
            SanitizedSignerAccount::sanitize(next_account_info(&mut accounts_iter)?, false)?;
        let avs = SanitizedSignerAccount::sanitize(next_account_info(&mut accounts_iter)?, false)?;
        let vault =
            SanitizedVault::sanitize(program_id, next_account_info(&mut accounts_iter)?, false)?;
        let config =
            SanitizedConfig::sanitize(program_id, next_account_info(&mut accounts_iter)?, false)?;
        let mut vault_avs_list = SanitizedVaultAvsList::sanitize(
            program_id,
            next_account_info(&mut accounts_iter)?,
            true,
            vault.account().key,
        )?;
        let payer = SanitizedSignerAccount::sanitize(next_account_info(&mut accounts_iter)?, true)?;
        let _system_program =
            SanitizedSystemProgram::sanitize(next_account_info(&mut accounts_iter)?)?;

        assert_with_msg(
            config.config().restaking_program_signer() == *restaking_program_signer.account().key,
            ProgramError::InvalidAccountData,
            "Restaking program signer does not match config",
        )?;

        let clock = Clock::get()?;

        assert_with_msg(
            vault_avs_list
                .vault_avs_list_mut()
                .add_avs(*avs.account().key, clock.slot),
            ProgramError::InvalidArgument,
            "AVS already added to vault",
        )?;

        msg!(
            "AVS @ {} added to vault @ {} in slot {}",
            avs.account().key,
            vault.account().key,
            clock.slot
        );

        vault_avs_list.save(&Rent::get()?, payer.account())?;

        Ok(())
    }

    /// Processes the vault remove AVS instruction: [`crate::LrtInstruction::RemoveAvs`]
    pub fn vault_remove_avs(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
        let mut accounts_iter = accounts.iter();

        let restaking_program_signer =
            SanitizedSignerAccount::sanitize(next_account_info(&mut accounts_iter)?, false)?;
        let avs = SanitizedSignerAccount::sanitize(next_account_info(&mut accounts_iter)?, false)?;
        let vault =
            SanitizedVault::sanitize(program_id, next_account_info(&mut accounts_iter)?, false)?;
        let config =
            SanitizedConfig::sanitize(program_id, next_account_info(&mut accounts_iter)?, false)?;
        let mut vault_avs_list = SanitizedVaultAvsList::sanitize(
            program_id,
            next_account_info(&mut accounts_iter)?,
            true,
            vault.account().key,
        )?;
        let payer = SanitizedSignerAccount::sanitize(next_account_info(&mut accounts_iter)?, true)?;
        let _system_program =
            SanitizedSystemProgram::sanitize(next_account_info(&mut accounts_iter)?)?;

        assert_with_msg(
            config.config().restaking_program_signer() == *restaking_program_signer.account().key,
            ProgramError::InvalidAccountData,
            "Restaking program signer does not match config",
        )?;

        let clock = Clock::get()?;

        assert_with_msg(
            vault_avs_list
                .vault_avs_list_mut()
                .remove_avs(*avs.account().key, clock.slot),
            ProgramError::InvalidArgument,
            "AVS not found in vault",
        )?;

        msg!(
            "AVS @ {} removed from vault @ {} in slot {}",
            avs.account().key,
            vault.account().key,
            clock.slot
        );

        vault_avs_list.save(&Rent::get()?, payer.account())?;

        Ok(())
    }

    /// Processes the vault add NO instruction: [`crate::LrtInstruction::AddOperator`]
    pub fn vault_add_node_operator(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
        let mut accounts_iter = accounts.iter();

        let restaking_program_signer =
            SanitizedSignerAccount::sanitize(next_account_info(&mut accounts_iter)?, false)?;
        let operator =
            SanitizedSignerAccount::sanitize(next_account_info(&mut accounts_iter)?, false)?;
        let vault =
            SanitizedVault::sanitize(program_id, next_account_info(&mut accounts_iter)?, false)?;
        let config =
            SanitizedConfig::sanitize(program_id, next_account_info(&mut accounts_iter)?, false)?;
        let mut vault_operator_list = SanitizedVaultOperatorList::sanitize(
            program_id,
            next_account_info(&mut accounts_iter)?,
            true,
            vault.account().key,
        )?;
        let payer = SanitizedSignerAccount::sanitize(next_account_info(&mut accounts_iter)?, true)?;
        let _system_program =
            SanitizedSystemProgram::sanitize(next_account_info(&mut accounts_iter)?)?;

        assert_with_msg(
            config.config().restaking_program_signer() == *restaking_program_signer.account().key,
            ProgramError::InvalidAccountData,
            "Restaking program signer does not match config",
        )?;

        let clock = Clock::get()?;

        assert_with_msg(
            vault_operator_list
                .vault_operator_list_mut()
                .add_operator(*operator.account().key, clock.slot),
            ProgramError::InvalidArgument,
            "Operator already added to vault",
        )?;

        msg!(
            "Operator @ {} added to vault @ {} in slot {}",
            operator.account().key,
            vault.account().key,
            clock.slot
        );

        vault_operator_list.save(&Rent::get()?, payer.account())?;

        Ok(())
    }

    /// Processes the vault remove NO instruction: [`crate::LrtInstruction::RemoveOperator`]
    pub fn vault_remove_node_operator(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
    ) -> ProgramResult {
        let mut accounts_iter = accounts.iter();

        let restaking_program_signer =
            SanitizedSignerAccount::sanitize(next_account_info(&mut accounts_iter)?, false)?;
        let operator =
            SanitizedSignerAccount::sanitize(next_account_info(&mut accounts_iter)?, false)?;
        let vault =
            SanitizedVault::sanitize(program_id, next_account_info(&mut accounts_iter)?, false)?;
        let config =
            SanitizedConfig::sanitize(program_id, next_account_info(&mut accounts_iter)?, false)?;
        let mut vault_operator_list = SanitizedVaultOperatorList::sanitize(
            program_id,
            next_account_info(&mut accounts_iter)?,
            true,
            vault.account().key,
        )?;
        let payer = SanitizedSignerAccount::sanitize(next_account_info(&mut accounts_iter)?, true)?;
        let _system_program =
            SanitizedSystemProgram::sanitize(next_account_info(&mut accounts_iter)?)?;

        assert_with_msg(
            config.config().restaking_program_signer() == *restaking_program_signer.account().key,
            ProgramError::InvalidAccountData,
            "Restaking program signer does not match config",
        )?;

        let clock = Clock::get()?;

        assert_with_msg(
            vault_operator_list
                .vault_operator_list_mut()
                .remove_operator(*operator.account().key, clock.slot),
            ProgramError::InvalidArgument,
            "Operator not found in vault",
        )?;

        msg!(
            "Operator @ {} removed from vault @ {} in slot {}",
            operator.account().key,
            vault.account().key,
            clock.slot
        );

        vault_operator_list.save(&Rent::get()?, payer.account())?;

        Ok(())
    }

    pub fn add_delegation(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        amount: u64,
    ) -> ProgramResult {
        let mut accounts_iter = accounts.iter();

        let _config =
            SanitizedConfig::sanitize(program_id, next_account_info(&mut accounts_iter)?, false)?;
        let vault =
            SanitizedVault::sanitize(program_id, next_account_info(&mut accounts_iter)?, false)?;
        let mut vault_operator_list = SanitizedVaultOperatorList::sanitize(
            program_id,
            next_account_info(&mut accounts_iter)?,
            true,
            vault.account().key,
        )?;
        let operator = next_account_info(&mut accounts_iter)?;
        let delegation_admin =
            SanitizedSignerAccount::sanitize(next_account_info(&mut accounts_iter)?, false)?;
        let payer = SanitizedSignerAccount::sanitize(next_account_info(&mut accounts_iter)?, true)?;

        assert_with_msg(
            vault.vault().delegation_admin() == *delegation_admin.account().key,
            ProgramError::InvalidAccountData,
            "Admin account does not match vault delegation admin",
        )?;

        vault_operator_list.vault_operator_list_mut().delegate(
            *operator.key,
            amount,
            vault.vault().tokens_deposited(),
        )?;

        // TODO (LB): CPI into restaking program?

        vault_operator_list.save(&Rent::get()?, payer.account())?;

        Ok(())
    }

    pub fn remove_delegation(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        amount: u64,
    ) -> ProgramResult {
        let mut accounts_iter = accounts.iter();

        let _config =
            SanitizedConfig::sanitize(program_id, next_account_info(&mut accounts_iter)?, false)?;
        let vault =
            SanitizedVault::sanitize(program_id, next_account_info(&mut accounts_iter)?, false)?;
        let mut vault_operator_list = SanitizedVaultOperatorList::sanitize(
            program_id,
            next_account_info(&mut accounts_iter)?,
            true,
            vault.account().key,
        )?;
        let operator = next_account_info(&mut accounts_iter)?;
        let delegation_admin =
            SanitizedSignerAccount::sanitize(next_account_info(&mut accounts_iter)?, false)?;
        let payer = SanitizedSignerAccount::sanitize(next_account_info(&mut accounts_iter)?, true)?;

        assert_with_msg(
            vault.vault().delegation_admin() == *delegation_admin.account().key,
            ProgramError::InvalidAccountData,
            "Admin account does not match vault delegation admin",
        )?;

        vault_operator_list
            .vault_operator_list_mut()
            .undelegate(*operator.key, amount)?;

        // TODO (LB): CPI into restaking program?

        vault_operator_list.save(&Rent::get()?, payer.account())?;

        Ok(())
    }

    /// Processes the set delegation admin instruction: [`crate::LrtInstruction::SetDelegationAdmin`]
    pub fn set_delegation_admin(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
        let mut accounts_iter = accounts.iter();

        let mut vault =
            SanitizedVault::sanitize(program_id, next_account_info(&mut accounts_iter)?, true)?;
        let admin =
            SanitizedSignerAccount::sanitize(next_account_info(&mut accounts_iter)?, false)?;
        let new_admin =
            SanitizedSignerAccount::sanitize(next_account_info(&mut accounts_iter)?, false)?;

        assert_with_msg(
            *admin.account().key == vault.vault().delegation_admin()
                || *admin.account().key == vault.vault().admin(),
            ProgramError::InvalidAccountData,
            "Admin account does not match vault delegation admin or admin",
        )?;

        vault
            .vault_mut()
            .set_delegation_admin(*new_admin.account().key);

        vault.save()?;

        Ok(())
    }

    /// Processes the set admin instruction: [`crate::LrtInstruction::SetAdmin`]
    pub fn set_admin(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
        let mut accounts_iter = accounts.iter();

        let mut vault =
            SanitizedVault::sanitize(program_id, next_account_info(&mut accounts_iter)?, true)?;
        let old_admin =
            SanitizedSignerAccount::sanitize(next_account_info(&mut accounts_iter)?, false)?;
        let new_admin =
            SanitizedSignerAccount::sanitize(next_account_info(&mut accounts_iter)?, false)?;

        assert_with_msg(
            *old_admin.account().key == vault.vault().admin(),
            ProgramError::InvalidAccountData,
            "Old admin account does not match vault admin",
        )?;

        vault.vault_mut().set_admin(*new_admin.account().key);

        vault.save()?;

        Ok(())
    }
}
