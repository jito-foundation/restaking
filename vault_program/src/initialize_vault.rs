use borsh::BorshSerialize;
use jito_restaking_sanitization::{
    assert_with_msg, create_account, empty_account::EmptyAccount, signer::SanitizedSignerAccount,
    system_program::SanitizedSystemProgram, token_mint::SanitizedTokenMint,
    token_program::SanitizedTokenProgram,
};
use jito_vault_core::{
    config::SanitizedConfig, vault::Vault, vault_delegation_list::VaultDelegationList,
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
    let SanitizedAccounts {
        mut config,
        vault_account,
        vault_delegation_list_account,
        lrt_mint,
        mint,
        admin,
        base,
        system_program,
        token_program,
    } = SanitizedAccounts::sanitize(program_id, accounts)?;

    assert_with_msg(
        lrt_mint.account().is_signer,
        ProgramError::InvalidAccountData,
        "Mint account is not a signer",
    )?;

    let rent = Rent::get()?;

    _create_lrt_mint(
        &lrt_mint,
        &admin,
        &system_program,
        &token_program,
        &vault_account,
        &rent,
    )?;

    _create_vault(
        program_id,
        &config,
        &vault_account,
        &lrt_mint,
        &mint,
        &admin,
        &base,
        &system_program,
        deposit_fee_bps,
        withdrawal_fee_bps,
        &rent,
    )?;

    _create_vault_delegation_list(
        program_id,
        &vault_account,
        &vault_delegation_list_account,
        &admin,
        &system_program,
        &rent,
    )?;

    let num_vaults = config.config_mut().increment_vaults();
    assert_with_msg(
        num_vaults.is_some(),
        ProgramError::InvalidAccountData,
        "Overflow when incrementing number of vaults",
    )?;
    config.save()?;

    Ok(())
}

fn _create_vault_delegation_list<'a, 'info>(
    program_id: &Pubkey,
    vault_account: &EmptyAccount<'a, 'info>,
    vault_delegation_list_account: &EmptyAccount<'a, 'info>,
    admin: &SanitizedSignerAccount<'a, 'info>,
    system_program: &SanitizedSystemProgram<'a, 'info>,
    rent: &Rent,
) -> ProgramResult {
    let (vault_delegation_list_address, bump, mut vault_delegation_list_seeds) =
        VaultDelegationList::find_program_address(program_id, vault_account.account().key);
    vault_delegation_list_seeds.push(vec![bump]);
    assert_with_msg(
        vault_delegation_list_address == *vault_delegation_list_account.account().key,
        ProgramError::InvalidAccountData,
        "Vault delegation list account is not at the correct PDA",
    )?;
    let vault_delegation_list = VaultDelegationList::new(*vault_account.account().key, bump);

    msg!(
        "Initializing vault delegation list @ address {}",
        vault_delegation_list_account.account().key
    );
    let vault_delegation_list_serialized = vault_delegation_list.try_to_vec()?;
    create_account(
        admin.account(),
        vault_delegation_list_account.account(),
        system_program.account(),
        program_id,
        rent,
        vault_delegation_list_serialized.len() as u64,
        &vault_delegation_list_seeds,
    )?;
    vault_delegation_list_account.account().data.borrow_mut()
        [..vault_delegation_list_serialized.len()]
        .copy_from_slice(&vault_delegation_list_serialized);

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn _create_vault<'a, 'info>(
    program_id: &Pubkey,
    config: &SanitizedConfig,
    vault_account: &EmptyAccount<'a, 'info>,
    lrt_mint: &EmptyAccount<'a, 'info>,
    mint: &SanitizedTokenMint<'a, 'info>,
    admin: &SanitizedSignerAccount<'a, 'info>,
    base: &SanitizedSignerAccount<'a, 'info>,
    system_program: &SanitizedSystemProgram<'a, 'info>,
    deposit_fee_bps: u16,
    withdrawal_fee_bps: u16,
    rent: &Rent,
) -> ProgramResult {
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
        config.config().vaults_count(),
        *base.account().key,
        deposit_fee_bps,
        withdrawal_fee_bps,
        bump,
    );

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
        rent,
        vault_serialized.len() as u64,
        &vault_seeds,
    )?;
    vault_account.account().data.borrow_mut()[..vault_serialized.len()]
        .copy_from_slice(&vault_serialized);

    Ok(())
}

fn _create_lrt_mint<'a, 'info>(
    lrt_mint: &EmptyAccount<'a, 'info>,
    admin: &SanitizedSignerAccount<'a, 'info>,
    system_program: &SanitizedSystemProgram<'a, 'info>,
    token_program: &SanitizedTokenProgram<'a, 'info>,
    vault_account: &EmptyAccount<'a, 'info>,
    rent: &Rent,
) -> ProgramResult {
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
    Ok(())
}

struct SanitizedAccounts<'a, 'info> {
    config: SanitizedConfig<'a, 'info>,
    vault_account: EmptyAccount<'a, 'info>,
    vault_delegation_list_account: EmptyAccount<'a, 'info>,
    lrt_mint: EmptyAccount<'a, 'info>,
    mint: SanitizedTokenMint<'a, 'info>,
    admin: SanitizedSignerAccount<'a, 'info>,
    base: SanitizedSignerAccount<'a, 'info>,
    system_program: SanitizedSystemProgram<'a, 'info>,
    token_program: SanitizedTokenProgram<'a, 'info>,
}

impl<'a, 'info> SanitizedAccounts<'a, 'info> {
    fn sanitize(
        program_id: &Pubkey,
        accounts: &'a [AccountInfo<'info>],
    ) -> Result<SanitizedAccounts<'a, 'info>, ProgramError> {
        let mut accounts_iter = accounts.iter();

        let config =
            SanitizedConfig::sanitize(program_id, next_account_info(&mut accounts_iter)?, true)?;
        let vault_account = EmptyAccount::sanitize(next_account_info(&mut accounts_iter)?, true)?;
        let vault_delegation_list_account =
            EmptyAccount::sanitize(next_account_info(&mut accounts_iter)?, true)?;
        let lrt_mint = EmptyAccount::sanitize(next_account_info(&mut accounts_iter)?, true)?;
        let mint = SanitizedTokenMint::sanitize(next_account_info(&mut accounts_iter)?, false)?;
        let admin = SanitizedSignerAccount::sanitize(next_account_info(&mut accounts_iter)?, true)?;
        let base = SanitizedSignerAccount::sanitize(next_account_info(&mut accounts_iter)?, false)?;
        let system_program =
            SanitizedSystemProgram::sanitize(next_account_info(&mut accounts_iter)?)?;
        let token_program =
            SanitizedTokenProgram::sanitize(next_account_info(&mut accounts_iter)?)?;

        Ok(SanitizedAccounts {
            config,
            vault_account,
            vault_delegation_list_account,
            lrt_mint,
            mint,
            admin,
            base,
            system_program,
            token_program,
        })
    }
}
