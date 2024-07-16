use borsh::BorshSerialize;
use jito_restaking_core::{avs::Avs, config::SanitizedConfig};
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

/// Initializes an AVS and associated accounts
/// [`crate::RestakingInstruction::InitializeAvs`]
pub fn process_initialize_avs(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let SanitizedAccounts {
        mut config,
        avs_account,
        admin,
        base,
        system_program,
    } = SanitizedAccounts::sanitize(program_id, accounts)?;

    let rent = Rent::get()?;

    _create_avs(
        program_id,
        &config,
        &avs_account,
        &base,
        &admin,
        &system_program,
        &rent,
    )?;

    config.config_mut().increment_avs()?;
    config.save()?;

    Ok(())
}

fn _create_avs<'a, 'info>(
    program_id: &Pubkey,
    config: &SanitizedConfig,
    avs_account: &EmptyAccount<'a, 'info>,
    base: &SanitizedSignerAccount<'a, 'info>,
    admin: &SanitizedSignerAccount<'a, 'info>,
    system_program: &SanitizedSystemProgram<'a, 'info>,
    rent: &Rent,
) -> ProgramResult {
    let (expected_avs_pubkey, avs_bump, mut avs_seeds) =
        Avs::find_program_address(program_id, base.account().key);
    avs_seeds.push(vec![avs_bump]);
    assert_with_msg(
        expected_avs_pubkey == *avs_account.account().key,
        ProgramError::InvalidAccountData,
        "AVS account is not at the correct PDA",
    )?;

    let avs = Avs::new(
        *base.account().key,
        *admin.account().key,
        *admin.account().key,
        *admin.account().key,
        *admin.account().key,
        *admin.account().key,
        config.config().avs_count(),
        avs_bump,
    );

    msg!("Initializing AVS @ address {}", avs_account.account().key);
    let serialized_avs = avs.try_to_vec()?;
    create_account(
        admin.account(),
        avs_account.account(),
        system_program.account(),
        program_id,
        rent,
        serialized_avs.len() as u64,
        &avs_seeds,
    )?;
    avs_account.account().data.borrow_mut()[..serialized_avs.len()]
        .copy_from_slice(&serialized_avs);

    Ok(())
}

struct SanitizedAccounts<'a, 'info> {
    config: SanitizedConfig<'a, 'info>,
    avs_account: EmptyAccount<'a, 'info>,
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
        let avs_account = EmptyAccount::sanitize(next_account_info(accounts_iter)?, true)?;
        let admin = SanitizedSignerAccount::sanitize(next_account_info(accounts_iter)?, true)?;
        let base = SanitizedSignerAccount::sanitize(next_account_info(accounts_iter)?, false)?;
        let system_program = SanitizedSystemProgram::sanitize(next_account_info(accounts_iter)?)?;

        Ok(SanitizedAccounts {
            config,
            avs_account,
            admin,
            base,
            system_program,
        })
    }
}
