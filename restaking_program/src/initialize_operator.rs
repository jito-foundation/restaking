use std::mem::size_of;

use borsh::BorshSerialize;
use jito_account_traits::{AccountDeserialize, Discriminator};
use jito_jsm_core::loader::{load_signer, load_system_account, load_system_program};
use jito_restaking_core::{
    config::{Config, SanitizedConfig},
    loader::load_config,
    operator::Operator,
};
use jito_restaking_sanitization::{
    assert_with_msg, create_account, empty_account::EmptyAccount, signer::SanitizedSignerAccount,
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

/// Initializes a node operator and associated accounts.
/// [`crate::RestakingInstruction::InitializeOperator`]
pub fn process_initialize_operator(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let [config, operator, admin, base, system_program] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    load_config(program_id, config, true)?;
    load_system_account(operator, true)?;
    load_signer(admin, true)?;
    load_signer(base, false)?;
    load_system_program(system_program)?;

    let (operator_pubkey, operator_bump, mut operator_seed) =
        Operator::find_program_address(program_id, base.key);
    operator_seed.push(vec![operator_bump]);
    if operator.key.ne(&operator_pubkey) {
        msg!("Operator account is not at the correct PDA");
        return Err(ProgramError::InvalidAccountData);
    }

    msg!("Initializing operator at address {}", operator.key);
    create_account(
        admin,
        config,
        system_program,
        program_id,
        &Rent::get()?,
        (8 + size_of::<Operator>()) as u64,
        &operator_seed,
    )?;

    let config = Config::try_from_slice_mut(&mut config.data.borrow_mut())?;

    let mut operator_data = operator.try_borrow_mut_data()?;
    operator_data[0] = Operator::DISCRIMINATOR;
    let operator = Operator::try_from_slice_mut(&mut operator_data)?;
    operator.base = *base.key;
    operator.admin = *admin.key;
    operator.ncn_admin = *admin.key;
    operator.vault_admin = *admin.key;
    operator.voter = *admin.key;
    operator.index = config.operator_count;
    operator.ncn_count = 0;
    operator.vault_count = 0;
    operator.bump = operator_bump;

    config.operator_count = config
        .operator_count
        .checked_add(1)
        .ok_or(ProgramError::InvalidAccountData)?;

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
    admin: SanitizedSignerAccount<'a, 'info>,
    base: SanitizedSignerAccount<'a, 'info>,
    system_program: SanitizedSystemProgram<'a, 'info>,
}

impl<'a, 'info> SanitizedAccounts<'a, 'info> {
    /// Sanitizes the accounts for the instruction: [`crate::RestakingInstruction::InitializeOperator`]
    fn sanitize(
        program_id: &Pubkey,
        accounts: &'a [AccountInfo<'info>],
    ) -> Result<SanitizedAccounts<'a, 'info>, ProgramError> {
        let accounts_iter = &mut accounts.iter();

        let config =
            SanitizedConfig::sanitize(program_id, next_account_info(accounts_iter)?, true)?;
        let operator_account = EmptyAccount::sanitize(next_account_info(accounts_iter)?, true)?;

        let admin = SanitizedSignerAccount::sanitize(next_account_info(accounts_iter)?, true)?;
        let base = SanitizedSignerAccount::sanitize(next_account_info(accounts_iter)?, false)?;
        let system_program = SanitizedSystemProgram::sanitize(next_account_info(accounts_iter)?)?;

        Ok(SanitizedAccounts {
            config,
            operator_account,
            admin,
            base,
            system_program,
        })
    }
}
