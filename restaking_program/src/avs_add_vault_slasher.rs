use jito_restaking_core::{
    avs::SanitizedAvs, avs_slasher_list::SanitizedAvsSlasherList,
    avs_vault_list::SanitizedAvsVaultList, config::SanitizedConfig,
};
use jito_restaking_sanitization::{
    signer::SanitizedSignerAccount, system_program::SanitizedSystemProgram,
};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::Clock,
    entrypoint::ProgramResult,
    program_error::ProgramError,
    pubkey::Pubkey,
    rent::Rent,
    sysvar::Sysvar,
};

pub fn process_avs_add_vault_slasher(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    max_slashable_per_epoch: u64,
) -> ProgramResult {
    let SanitizedAccounts {
        avs,
        avs_vault_list,
        mut avs_slasher_list,
        vault,
        slasher,
        admin,
        payer,
    } = SanitizedAccounts::sanitize(program_id, accounts)?;

    avs.avs().check_vault_admin(admin.account().key)?;

    let slot = Clock::get()?.slot;
    avs_vault_list
        .avs_vault_list()
        .check_active_vault(vault.key, slot)?;
    avs_slasher_list.avs_slasher_list_mut().add_slasher(
        *vault.key,
        *slasher.key,
        slot,
        max_slashable_per_epoch,
    )?;

    avs_slasher_list.save_with_realloc(&Rent::get()?, payer.account())?;

    Ok(())
}

struct SanitizedAccounts<'a, 'info> {
    avs: SanitizedAvs<'a, 'info>,
    avs_vault_list: SanitizedAvsVaultList<'a, 'info>,
    avs_slasher_list: SanitizedAvsSlasherList<'a, 'info>,
    vault: &'a AccountInfo<'info>,
    slasher: &'a AccountInfo<'info>,
    admin: SanitizedSignerAccount<'a, 'info>,
    payer: SanitizedSignerAccount<'a, 'info>,
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
        let avs_vault_list = SanitizedAvsVaultList::sanitize(
            program_id,
            next_account_info(accounts_iter)?,
            false,
            avs.account().key,
        )?;
        let avs_slasher_list = SanitizedAvsSlasherList::sanitize(
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

        Ok(SanitizedAccounts {
            avs,
            avs_vault_list,
            avs_slasher_list,
            vault,
            slasher,
            admin,
            payer,
        })
    }
}
