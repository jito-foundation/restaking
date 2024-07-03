use jito_restaking_core::{
    avs::SanitizedAvs, avs_slasher_list::SanitizedAvsSlasherList, config::SanitizedConfig,
};
use jito_restaking_sanitization::signer::SanitizedSignerAccount;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::Clock,
    entrypoint::ProgramResult,
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvar::Sysvar,
};

pub fn process_avs_deprecate_slasher(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let SanitizedAccounts {
        avs,
        mut avs_slasher_list,
        vault,
        slasher,
        admin,
    } = SanitizedAccounts::sanitize(program_id, accounts)?;

    avs.avs().check_slasher_admin(admin.account().key)?;

    let clock = Clock::get()?;
    avs_slasher_list.avs_slasher_list_mut().deprecate_slasher(
        *vault.key,
        *slasher.key,
        clock.slot,
    )?;

    avs_slasher_list.save()?;

    Ok(())
}

struct SanitizedAccounts<'a, 'info> {
    avs: SanitizedAvs<'a, 'info>,
    avs_slasher_list: SanitizedAvsSlasherList<'a, 'info>,
    vault: &'a AccountInfo<'info>,
    slasher: &'a AccountInfo<'info>,
    admin: SanitizedSignerAccount<'a, 'info>,
}

impl<'a, 'info> SanitizedAccounts<'a, 'info> {
    fn sanitize(
        program_id: &Pubkey,
        accounts: &'a [AccountInfo<'info>],
    ) -> Result<SanitizedAccounts<'a, 'info>, ProgramError> {
        let accounts_iter = &mut accounts.iter();

        let _config =
            SanitizedConfig::sanitize(program_id, next_account_info(accounts_iter)?, false)?;
        let avs = SanitizedAvs::sanitize(program_id, next_account_info(accounts_iter)?, false)?;

        let avs_slasher_list = SanitizedAvsSlasherList::sanitize(
            program_id,
            next_account_info(accounts_iter)?,
            true,
            avs.account().key,
        )?;
        let vault = next_account_info(accounts_iter)?;
        let slasher = next_account_info(accounts_iter)?;
        let admin = SanitizedSignerAccount::sanitize(next_account_info(accounts_iter)?, false)?;
        Ok(SanitizedAccounts {
            avs,
            avs_slasher_list,
            vault,
            slasher,
            admin,
        })
    }
}
