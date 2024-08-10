use borsh::BorshSerialize;
use jito_restaking_core::{config::SanitizedConfig, ncn::Ncn};
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

/// Initializes an NCN and associated accounts
/// [`crate::RestakingInstruction::InitializeNcn`]
pub fn process_initialize_ncn(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let SanitizedAccounts {
        mut config,
        ncn_account,
        admin,
        base,
        system_program,
    } = SanitizedAccounts::sanitize(program_id, accounts)?;

    let rent = Rent::get()?;

    _create_ncn(
        program_id,
        &config,
        &ncn_account,
        &base,
        &admin,
        &system_program,
        &rent,
    )?;

    config.config_mut().increment_ncn()?;
    config.save()?;

    Ok(())
}

fn _create_ncn<'a, 'info>(
    program_id: &Pubkey,
    config: &SanitizedConfig,
    ncn_account: &EmptyAccount<'a, 'info>,
    base: &SanitizedSignerAccount<'a, 'info>,
    admin: &SanitizedSignerAccount<'a, 'info>,
    system_program: &SanitizedSystemProgram<'a, 'info>,
    rent: &Rent,
) -> ProgramResult {
    let (expected_ncn_pubkey, ncn_bump, mut ncn_seeds) =
        Ncn::find_program_address(program_id, base.account().key);
    ncn_seeds.push(vec![ncn_bump]);
    assert_with_msg(
        expected_ncn_pubkey == *ncn_account.account().key,
        ProgramError::InvalidAccountData,
        "NCN account is not at the correct PDA",
    )?;

    let ncn = Ncn::new(
        *base.account().key,
        *admin.account().key,
        *admin.account().key,
        *admin.account().key,
        *admin.account().key,
        *admin.account().key,
        config.config().ncn_count(),
        ncn_bump,
    );

    msg!("Initializing NCN @ address {}", ncn_account.account().key);
    let serialized_ncn = ncn.try_to_vec()?;
    create_account(
        admin.account(),
        ncn_account.account(),
        system_program.account(),
        program_id,
        rent,
        serialized_ncn.len() as u64,
        &ncn_seeds,
    )?;
    ncn_account.account().data.borrow_mut()[..serialized_ncn.len()]
        .copy_from_slice(&serialized_ncn);

    Ok(())
}

struct SanitizedAccounts<'a, 'info> {
    config: SanitizedConfig<'a, 'info>,
    ncn_account: EmptyAccount<'a, 'info>,
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
        let ncn_account = EmptyAccount::sanitize(next_account_info(accounts_iter)?, true)?;
        let admin = SanitizedSignerAccount::sanitize(next_account_info(accounts_iter)?, true)?;
        let base = SanitizedSignerAccount::sanitize(next_account_info(accounts_iter)?, false)?;
        let system_program = SanitizedSystemProgram::sanitize(next_account_info(accounts_iter)?)?;

        Ok(SanitizedAccounts {
            config,
            ncn_account,
            admin,
            base,
            system_program,
        })
    }
}
