use borsh::BorshSerialize;
use jito_restaking_core::{config::SanitizedConfig, operator::Operator};
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

    config.config_mut().increment_operators()?;
    config.save()?;

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
