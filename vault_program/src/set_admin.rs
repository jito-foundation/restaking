use jito_restaking_sanitization::signer::SanitizedSignerAccount;
use jito_vault_core::vault::SanitizedVault;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program_error::ProgramError,
    pubkey::Pubkey,
};

/// Processes the set admin instruction: [`crate::VaultInstruction::SetAdmin`]
pub fn process_set_admin(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let SanitizedAccounts {
        old_admin,
        new_admin,
        mut vault,
    } = SanitizedAccounts::sanitize(program_id, accounts)?;

    vault.vault().check_admin(old_admin.account().key)?;
    vault.vault_mut().set_admin(*new_admin.account().key);
    vault.save()?;

    Ok(())
}

struct SanitizedAccounts<'a, 'info> {
    old_admin: SanitizedSignerAccount<'a, 'info>,
    new_admin: SanitizedSignerAccount<'a, 'info>,
    vault: SanitizedVault<'a, 'info>,
}

impl<'a, 'info> SanitizedAccounts<'a, 'info> {
    fn sanitize(
        program_id: &Pubkey,
        accounts: &'a [AccountInfo<'info>],
    ) -> Result<SanitizedAccounts<'a, 'info>, ProgramError> {
        let mut accounts_iter = accounts.iter();

        let vault =
            SanitizedVault::sanitize(program_id, next_account_info(&mut accounts_iter)?, true)?;
        let old_admin =
            SanitizedSignerAccount::sanitize(next_account_info(&mut accounts_iter)?, false)?;
        let new_admin =
            SanitizedSignerAccount::sanitize(next_account_info(&mut accounts_iter)?, false)?;

        Ok(SanitizedAccounts {
            old_admin,
            new_admin,
            vault,
        })
    }
}
