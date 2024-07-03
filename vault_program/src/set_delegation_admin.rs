use jito_restaking_sanitization::signer::SanitizedSignerAccount;
use jito_vault_core::vault::SanitizedVault;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program_error::ProgramError,
    pubkey::Pubkey,
};

/// Processes the set delegation admin instruction: [`crate::VaultInstruction::SetDelegationAdmin`]
pub fn process_set_delegation_admin(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let SanitizedAccounts {
        mut vault,
        admin,
        new_delegation_admin,
    } = SanitizedAccounts::sanitize(program_id, accounts)?;

    if vault.vault().check_admin(admin.account().key).is_err() {
        vault.vault().check_delegation_admin(admin.account().key)?;
    }
    vault
        .vault_mut()
        .set_delegation_admin(*new_delegation_admin.account().key);
    vault.save()?;

    Ok(())
}

struct SanitizedAccounts<'a, 'info> {
    vault: SanitizedVault<'a, 'info>,
    admin: SanitizedSignerAccount<'a, 'info>,
    new_delegation_admin: SanitizedSignerAccount<'a, 'info>,
}

impl<'a, 'info> SanitizedAccounts<'a, 'info> {
    fn sanitize(
        program_id: &Pubkey,
        accounts: &'a [AccountInfo<'info>],
    ) -> Result<SanitizedAccounts<'a, 'info>, ProgramError> {
        let mut accounts_iter = accounts.iter();

        let vault =
            SanitizedVault::sanitize(program_id, next_account_info(&mut accounts_iter)?, true)?;
        let admin =
            SanitizedSignerAccount::sanitize(next_account_info(&mut accounts_iter)?, false)?;
        let new_delegation_admin =
            SanitizedSignerAccount::sanitize(next_account_info(&mut accounts_iter)?, false)?;

        Ok(SanitizedAccounts {
            vault,
            admin,
            new_delegation_admin,
        })
    }
}
