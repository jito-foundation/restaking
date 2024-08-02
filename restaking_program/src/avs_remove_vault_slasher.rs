use jito_restaking_core::{
    avs::SanitizedAvs, avs_vault_slasher_ticket::SanitizedAvsVaultSlasherTicket,
    config::SanitizedConfig,
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

pub fn process_avs_remove_slasher(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let SanitizedAccounts {
        config,
        avs,
        mut avs_vault_slasher_ticket,
        admin,
    } = SanitizedAccounts::sanitize(program_id, accounts)?;

    avs.avs().check_slasher_admin(admin.account().key)?;

    let clock = Clock::get()?;

    avs_vault_slasher_ticket
        .avs_vault_slasher_ticket_mut()
        .deactivate(clock.slot, config.config().epoch_length())?;

    avs_vault_slasher_ticket.save()?;

    Ok(())
}

struct SanitizedAccounts<'a, 'info> {
    config: SanitizedConfig<'a, 'info>,
    avs: SanitizedAvs<'a, 'info>,
    avs_vault_slasher_ticket: SanitizedAvsVaultSlasherTicket<'a, 'info>,
    admin: SanitizedSignerAccount<'a, 'info>,
}

impl<'a, 'info> SanitizedAccounts<'a, 'info> {
    /// Sanitizes the accounts for the instruction: [`crate::RestakingInstruction::AvsRemoveVaultSlasher`]
    fn sanitize(
        program_id: &Pubkey,
        accounts: &'a [AccountInfo<'info>],
    ) -> Result<SanitizedAccounts<'a, 'info>, ProgramError> {
        let accounts_iter = &mut accounts.iter();

        let config =
            SanitizedConfig::sanitize(program_id, next_account_info(accounts_iter)?, false)?;
        let avs = SanitizedAvs::sanitize(program_id, next_account_info(accounts_iter)?, false)?;
        let vault = next_account_info(accounts_iter)?;
        let slasher = next_account_info(accounts_iter)?;
        let avs_vault_slasher_ticket = SanitizedAvsVaultSlasherTicket::sanitize(
            program_id,
            next_account_info(accounts_iter)?,
            true,
            avs.account().key,
            vault.key,
            slasher.key,
        )?;
        let admin = SanitizedSignerAccount::sanitize(next_account_info(accounts_iter)?, false)?;

        Ok(SanitizedAccounts {
            config,
            avs,
            avs_vault_slasher_ticket,
            admin,
        })
    }
}
