use jito_restaking_core::avs::SanitizedAvs;
use jito_restaking_sanitization::signer::SanitizedSignerAccount;
use jito_vault_core::{
    config::SanitizedConfig, vault::SanitizedVault, vault_avs_ticket::SanitizedVaultAvsTicket,
};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::Clock,
    entrypoint::ProgramResult,
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvar::Sysvar,
};

/// Remove a vault from the vault's AVS list.
///
/// # Behavior:
/// * The vault admin shall have the ability to remove support for a previously supported vault
/// at any time, independent of whether the AVS still supports the vault or not.
///
/// Instruction: [`crate::VaultInstruction::RemoveAvs`]
pub fn process_vault_remove_avs(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let SanitizedAccounts {
        vault,
        mut vault_avs_ticket,
        admin,
    } = SanitizedAccounts::sanitize(program_id, accounts)?;

    vault.vault().check_avs_admin(admin.account().key)?;

    let slot = Clock::get()?.slot;
    vault_avs_ticket.vault_avs_ticket_mut().deactivate(slot)?;

    vault_avs_ticket.save()?;

    Ok(())
}

struct SanitizedAccounts<'a, 'info> {
    vault: SanitizedVault<'a, 'info>,
    vault_avs_ticket: SanitizedVaultAvsTicket<'a, 'info>,
    admin: SanitizedSignerAccount<'a, 'info>,
}

impl<'a, 'info> SanitizedAccounts<'a, 'info> {
    /// Sanitize the accounts for instruction: [`crate::VaultInstruction::RemoveAvs`]
    fn sanitize(
        program_id: &Pubkey,
        accounts: &'a [AccountInfo<'info>],
    ) -> Result<SanitizedAccounts<'a, 'info>, ProgramError> {
        let mut accounts_iter = accounts.iter();

        let config =
            SanitizedConfig::sanitize(program_id, next_account_info(&mut accounts_iter)?, false)?;
        let vault =
            SanitizedVault::sanitize(program_id, next_account_info(&mut accounts_iter)?, false)?;
        let avs = SanitizedAvs::sanitize(
            &config.config().restaking_program(),
            next_account_info(&mut accounts_iter)?,
            false,
        )?;
        let vault_avs_ticket = SanitizedVaultAvsTicket::sanitize(
            program_id,
            next_account_info(&mut accounts_iter)?,
            true,
            vault.account().key,
            avs.account().key,
        )?;
        let admin =
            SanitizedSignerAccount::sanitize(next_account_info(&mut accounts_iter)?, false)?;

        Ok(SanitizedAccounts {
            vault,
            vault_avs_ticket,
            admin,
        })
    }
}
