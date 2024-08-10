use std::mem::size_of;

use borsh::BorshSerialize;
use jito_restaking_sanitization::{
    assert_with_msg, create_account, empty_account::EmptyAccount, signer::SanitizedSignerAccount,
    system_program::SanitizedSystemProgram,
};
use jito_vault_core::config::Config;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
    rent::Rent,
    sysvar::Sysvar,
};

/// Processes the initialize config instruction: [`crate::VaultInstruction::InitializeConfig`]
pub fn process_initialize_config(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let SanitizedAccounts {
        config_account,
        admin,
        restaking_program,
        system_program,
    } = SanitizedAccounts::sanitize(accounts)?;

    let (config_address, bump, mut config_seeds) = Config::find_program_address(program_id);
    config_seeds.push(vec![bump]);
    assert_with_msg(
        config_address == *config_account.account().key,
        ProgramError::InvalidAccountData,
        "Config account is not at the correct PDA",
    )?;

    let config = Config::new(*admin.account().key, *restaking_program.key, bump);
    msg!(
        "Initializing config @ address {}",
        config_account.account().key
    );
    // let config_serialized = config.try_to_vec()?;
    let config_serialized = bytemuck::bytes_of(&config);
    create_account(
        admin.account(),
        config_account.account(),
        system_program.account(),
        program_id,
        &Rent::get()?,
        config_serialized.len() as u64,
        &config_seeds,
    )?;
    config_account.account().data.borrow_mut()[..config_serialized.len()]
        .copy_from_slice(&config_serialized);

    Ok(())
}

struct SanitizedAccounts<'a, 'info> {
    config_account: EmptyAccount<'a, 'info>,
    admin: SanitizedSignerAccount<'a, 'info>,
    restaking_program: &'a AccountInfo<'info>,
    system_program: SanitizedSystemProgram<'a, 'info>,
}

impl<'a, 'info> SanitizedAccounts<'a, 'info> {
    fn sanitize(
        accounts: &'a [AccountInfo<'info>],
    ) -> Result<SanitizedAccounts<'a, 'info>, ProgramError> {
        let mut accounts_iter = accounts.iter();

        let config_account = EmptyAccount::sanitize(next_account_info(&mut accounts_iter)?, true)?;

        let admin = SanitizedSignerAccount::sanitize(next_account_info(&mut accounts_iter)?, true)?;

        let restaking_program = next_account_info(&mut accounts_iter)?;

        let system_program =
            SanitizedSystemProgram::sanitize(next_account_info(&mut accounts_iter)?)?;

        Ok(SanitizedAccounts {
            config_account,
            admin,
            restaking_program,
            system_program,
        })
    }
}
