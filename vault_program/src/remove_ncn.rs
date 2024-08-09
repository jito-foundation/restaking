use jito_restaking_core::ncn::SanitizedNcn;
use jito_restaking_sanitization::signer::SanitizedSignerAccount;
use jito_vault_core::{
    config::SanitizedConfig, vault::SanitizedVault, vault_ncn_ticket::SanitizedVaultNcnTicket,
};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::Clock,
    entrypoint::ProgramResult,
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvar::Sysvar,
};

/// Remove a vault from the vault's NCN list.
///
/// # Behavior:
/// * The vault admin shall have the ability to remove support for a previously supported vault
/// at any time, independent of whether the NCN still supports the vault or not.
///
/// Instruction: [`crate::VaultInstruction::RemoveNcn`]
pub fn process_vault_remove_ncn(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let SanitizedAccounts {
        config,
        vault,
        mut vault_ncn_ticket,
        admin,
    } = SanitizedAccounts::sanitize(program_id, accounts)?;

    vault.vault().check_ncn_admin(admin.account().key)?;

    let slot = Clock::get()?.slot;
    vault_ncn_ticket
        .vault_ncn_ticket_mut()
        .deactivate(slot, config.config().epoch_length())?;

    vault_ncn_ticket.save()?;

    Ok(())
}

struct SanitizedAccounts<'a, 'info> {
    config: SanitizedConfig<'a, 'info>,
    vault: SanitizedVault<'a, 'info>,
    vault_ncn_ticket: SanitizedVaultNcnTicket<'a, 'info>,
    admin: SanitizedSignerAccount<'a, 'info>,
}

impl<'a, 'info> SanitizedAccounts<'a, 'info> {
    /// Sanitize the accounts for instruction: [`crate::VaultInstruction::RemoveNcn`]
    fn sanitize(
        program_id: &Pubkey,
        accounts: &'a [AccountInfo<'info>],
    ) -> Result<SanitizedAccounts<'a, 'info>, ProgramError> {
        let mut accounts_iter = accounts.iter();

        let config =
            SanitizedConfig::sanitize(program_id, next_account_info(&mut accounts_iter)?, false)?;
        let vault =
            SanitizedVault::sanitize(program_id, next_account_info(&mut accounts_iter)?, false)?;
        let ncn = SanitizedNcn::sanitize(
            &config.config().restaking_program(),
            next_account_info(&mut accounts_iter)?,
            false,
        )?;
        let vault_ncn_ticket = SanitizedVaultNcnTicket::sanitize(
            program_id,
            next_account_info(&mut accounts_iter)?,
            true,
            vault.account().key,
            ncn.account().key,
        )?;
        let admin =
            SanitizedSignerAccount::sanitize(next_account_info(&mut accounts_iter)?, false)?;

        Ok(SanitizedAccounts {
            config,
            vault,
            vault_ncn_ticket,
            admin,
        })
    }
}
