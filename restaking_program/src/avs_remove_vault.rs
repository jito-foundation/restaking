use jito_restaking_core::{
    avs::SanitizedAvs, avs_vault_ticket::SanitizedAvsVaultTicket, config::SanitizedConfig,
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

/// [`crate::RestakingInstruction::AvsRemoveVault`]
pub fn process_avs_remove_vault(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let SanitizedAccounts {
        avs,
        mut avs_vault_ticket,
        admin,
    } = SanitizedAccounts::sanitize(program_id, accounts)?;

    avs.avs().check_vault_admin(admin.account().key)?;

    let slot = Clock::get()?.slot;
    avs_vault_ticket.avs_vault_ticket_mut().deactivate(slot)?;

    avs_vault_ticket.save()?;

    Ok(())
}

struct SanitizedAccounts<'a, 'info> {
    avs: SanitizedAvs<'a, 'info>,
    avs_vault_ticket: SanitizedAvsVaultTicket<'a, 'info>,
    admin: SanitizedSignerAccount<'a, 'info>,
}

impl<'a, 'info> SanitizedAccounts<'a, 'info> {
    /// Sanitizes the accounts for the instruction: [`crate::RestakingInstruction::AvsRemoveVault`]
    pub fn sanitize(
        program_id: &Pubkey,
        accounts: &'a [AccountInfo<'info>],
    ) -> Result<SanitizedAccounts<'a, 'info>, ProgramError> {
        let accounts_iter = &mut accounts.iter();

        let _config =
            SanitizedConfig::sanitize(program_id, next_account_info(accounts_iter)?, false)?;
        let avs = SanitizedAvs::sanitize(program_id, next_account_info(accounts_iter)?, false)?;
        let vault = next_account_info(accounts_iter)?;
        let avs_vault_ticket = SanitizedAvsVaultTicket::sanitize(
            program_id,
            next_account_info(accounts_iter)?,
            true,
            avs.account().key,
            vault.key,
        )?;
        let admin = SanitizedSignerAccount::sanitize(next_account_info(accounts_iter)?, false)?;

        Ok(SanitizedAccounts {
            avs,
            avs_vault_ticket,
            admin,
        })
    }
}
