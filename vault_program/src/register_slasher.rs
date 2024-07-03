use jito_restaking_core::{avs::SanitizedAvs, avs_slasher_list::SanitizedAvsSlasherList};
use jito_restaking_sanitization::{
    assert_with_msg, signer::SanitizedSignerAccount, system_program::SanitizedSystemProgram,
};
use jito_vault_core::{
    config::SanitizedConfig, vault::SanitizedVault, vault_slasher_list::SanitizedVaultSlasherList,
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

/// Processes the register slasher instruction: [`crate::VaultInstruction::AddSlasher`]
pub fn process_register_slasher(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let SanitizedAccounts {
        vault,
        mut vault_slasher_list,
        slasher,
        admin,
        payer,
        avs,
        avs_slasher_list,
    } = SanitizedAccounts::sanitize(program_id, accounts)?;

    vault.vault().check_admin(admin.account().key)?;

    let slot = Clock::get()?.slot;
    let slasher_info = avs_slasher_list.avs_slasher_list().get_slasher_info(
        *vault.account().key,
        *slasher.key,
        slot,
    );
    assert_with_msg(
        slasher_info.is_some(),
        ProgramError::InvalidArgument,
        "Slasher for this vault does not exist",
    )?;

    let slasher_added = vault_slasher_list.vault_slasher_list_mut().add_slasher(
        avs.account().key,
        slasher.key,
        slasher_info.unwrap().max_slashable_per_epoch(),
        slot,
    );
    assert_with_msg(
        slasher_added,
        ProgramError::InvalidArgument,
        "Slasher already exists",
    )?;

    vault_slasher_list.save(&Rent::get()?, payer.account())?;

    Ok(())
}

struct SanitizedAccounts<'a, 'info> {
    vault: SanitizedVault<'a, 'info>,
    vault_slasher_list: SanitizedVaultSlasherList<'a, 'info>,
    slasher: &'a AccountInfo<'info>,
    admin: SanitizedSignerAccount<'a, 'info>,
    payer: SanitizedSignerAccount<'a, 'info>,
    avs: SanitizedAvs<'a, 'info>,
    avs_slasher_list: SanitizedAvsSlasherList<'a, 'info>,
}

impl<'a, 'info> SanitizedAccounts<'a, 'info> {
    fn sanitize(
        program_id: &Pubkey,
        accounts: &'a [AccountInfo<'info>],
    ) -> Result<SanitizedAccounts<'a, 'info>, ProgramError> {
        let account_iter = &mut accounts.iter();

        let config =
            SanitizedConfig::sanitize(program_id, next_account_info(account_iter)?, false)?;
        let vault = SanitizedVault::sanitize(program_id, next_account_info(account_iter)?, false)?;
        let vault_slasher_list = SanitizedVaultSlasherList::sanitize(
            program_id,
            next_account_info(account_iter)?,
            true,
            vault.account().key,
        )?;
        let slasher = next_account_info(account_iter)?;
        let admin = SanitizedSignerAccount::sanitize(next_account_info(account_iter)?, false)?;
        let payer = SanitizedSignerAccount::sanitize(next_account_info(account_iter)?, true)?;

        let avs = SanitizedAvs::sanitize(
            &config.config().restaking_program(),
            next_account_info(account_iter)?,
            false,
        )?;
        let avs_slasher_list = SanitizedAvsSlasherList::sanitize(
            &config.config().restaking_program(),
            next_account_info(account_iter)?,
            false,
            avs.account().key,
        )?;
        let _system_program = SanitizedSystemProgram::sanitize(next_account_info(account_iter)?)?;

        Ok(SanitizedAccounts {
            vault,
            vault_slasher_list,
            slasher,
            admin,
            payer,
            avs,
            avs_slasher_list,
        })
    }
}
