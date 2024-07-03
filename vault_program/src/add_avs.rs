use jito_restaking_core::{avs::SanitizedAvs, avs_vault_list::SanitizedAvsVaultList};
use jito_restaking_sanitization::{
    signer::SanitizedSignerAccount, system_program::SanitizedSystemProgram,
};
use jito_vault_core::{
    config::SanitizedConfig, vault::SanitizedVault, vault_avs_list::SanitizedVaultAvsList,
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

/// Adds an AVS to the vault AVS list, which means delegation applied to operators staking to the AVS
/// will be applied.
///
/// # Behavior
/// * The vault admin shall have the ability to add support for a new AVS
/// if the AVS is actively supporting the vault
///
/// Instruction: [`crate::VaultInstruction::AddAvs`]
pub fn process_vault_add_avs(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let SanitizedAccounts {
        vault,
        mut vault_avs_list,
        avs,
        avs_vault_list,
        admin,
        payer,
    } = SanitizedAccounts::sanitize(program_id, accounts)?;

    vault.vault().check_admin(admin.account().key)?;

    let slot = Clock::get()?.slot;
    // The AVS must support the vault for it to be added
    avs_vault_list
        .avs_vault_list()
        .check_active_vault(vault.account().key, slot)?;
    vault_avs_list
        .vault_avs_list_mut()
        .add_avs(*avs.account().key, slot)?;

    let rent = Rent::get()?;
    vault_avs_list.save_with_realloc(&rent, payer.account())?;

    Ok(())
}

struct SanitizedAccounts<'a, 'info> {
    // config: SanitizedConfig<'a, 'info>,
    vault: SanitizedVault<'a, 'info>,
    vault_avs_list: SanitizedVaultAvsList<'a, 'info>,
    avs: SanitizedAvs<'a, 'info>,
    avs_vault_list: SanitizedAvsVaultList<'a, 'info>,
    admin: SanitizedSignerAccount<'a, 'info>,
    payer: SanitizedSignerAccount<'a, 'info>,
}

impl<'a, 'info> SanitizedAccounts<'a, 'info> {
    fn sanitize(
        program_id: &Pubkey,
        accounts: &'a [AccountInfo<'info>],
    ) -> Result<SanitizedAccounts<'a, 'info>, ProgramError> {
        let mut accounts_iter = accounts.iter();

        let config =
            SanitizedConfig::sanitize(program_id, next_account_info(&mut accounts_iter)?, false)?;
        let vault =
            SanitizedVault::sanitize(program_id, next_account_info(&mut accounts_iter)?, false)?;
        let vault_avs_list = SanitizedVaultAvsList::sanitize(
            program_id,
            next_account_info(&mut accounts_iter)?,
            true,
            vault.account().key,
        )?;
        let avs = SanitizedAvs::sanitize(
            &config.config().restaking_program(),
            next_account_info(&mut accounts_iter)?,
            false,
        )?;
        let avs_vault_list = SanitizedAvsVaultList::sanitize(
            &config.config().restaking_program(),
            next_account_info(&mut accounts_iter)?,
            false,
            avs.account().key,
        )?;
        let admin =
            SanitizedSignerAccount::sanitize(next_account_info(&mut accounts_iter)?, false)?;
        let payer = SanitizedSignerAccount::sanitize(next_account_info(&mut accounts_iter)?, true)?;
        let _system_program =
            SanitizedSystemProgram::sanitize(next_account_info(&mut accounts_iter)?)?;

        Ok(SanitizedAccounts {
            // config,
            vault,
            vault_avs_list,
            avs,
            avs_vault_list,
            admin,
            payer,
        })
    }
}
