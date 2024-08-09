use jito_restaking_core::{
    config::SanitizedConfig, ncn::SanitizedNcn, ncn_vault_ticket::SanitizedNcnVaultTicket,
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

/// [`crate::RestakingInstruction::NcnRemoveVault`]
pub fn process_ncn_remove_vault(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let SanitizedAccounts {
        config,
        ncn,
        mut ncn_vault_ticket,
        admin,
    } = SanitizedAccounts::sanitize(program_id, accounts)?;

    ncn.ncn().check_vault_admin(admin.account().key)?;

    let slot = Clock::get()?.slot;
    ncn_vault_ticket
        .ncn_vault_ticket_mut()
        .deactivate(slot, config.config().epoch_length())?;

    ncn_vault_ticket.save()?;

    Ok(())
}

struct SanitizedAccounts<'a, 'info> {
    config: SanitizedConfig<'a, 'info>,
    ncn: SanitizedNcn<'a, 'info>,
    ncn_vault_ticket: SanitizedNcnVaultTicket<'a, 'info>,
    admin: SanitizedSignerAccount<'a, 'info>,
}

impl<'a, 'info> SanitizedAccounts<'a, 'info> {
    /// Sanitizes the accounts for the instruction: [`crate::RestakingInstruction::ncnRemoveVault`]
    pub fn sanitize(
        program_id: &Pubkey,
        accounts: &'a [AccountInfo<'info>],
    ) -> Result<SanitizedAccounts<'a, 'info>, ProgramError> {
        let accounts_iter = &mut accounts.iter();

        let config =
            SanitizedConfig::sanitize(program_id, next_account_info(accounts_iter)?, false)?;
        let ncn = SanitizedNcn::sanitize(program_id, next_account_info(accounts_iter)?, false)?;
        let vault = next_account_info(accounts_iter)?;
        let ncn_vault_ticket = SanitizedNcnVaultTicket::sanitize(
            program_id,
            next_account_info(accounts_iter)?,
            true,
            ncn.account().key,
            vault.key,
        )?;
        let admin = SanitizedSignerAccount::sanitize(next_account_info(accounts_iter)?, false)?;

        Ok(SanitizedAccounts {
            config,
            ncn,
            ncn_vault_ticket,
            admin,
        })
    }
}
