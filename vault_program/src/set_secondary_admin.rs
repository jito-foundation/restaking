use jito_restaking_sanitization::signer::SanitizedSignerAccount;
use jito_vault_core::vault::SanitizedVault;
use jito_vault_sdk::VaultAdminRole;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program_error::ProgramError,
    pubkey::Pubkey,
};

/// Processes the set delegation admin instruction: [`crate::VaultInstruction::SetSecondaryAdmin`]
pub fn process_set_secondary_admin(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    role: VaultAdminRole,
) -> ProgramResult {
    let SanitizedAccounts {
        mut vault,
        admin,
        new_admin,
    } = SanitizedAccounts::sanitize(program_id, accounts)?;

    vault.vault().check_admin(admin.account().key)?;

    match role {
        VaultAdminRole::Delegataion => {
            vault.vault_mut().set_delegation_admin(*new_admin.key);
        }
        VaultAdminRole::FeeOwner => {
            vault.vault_mut().set_fee_wallet(*new_admin.key);
        }
        VaultAdminRole::MintBurnAuthority => {
            vault.vault_mut().set_mint_burn_authority(*new_admin.key);
        }
    }

    vault.save()?;

    Ok(())
}

struct SanitizedAccounts<'a, 'info> {
    vault: SanitizedVault<'a, 'info>,
    admin: SanitizedSignerAccount<'a, 'info>,
    new_admin: &'a AccountInfo<'info>,
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
        let new_admin = next_account_info(&mut accounts_iter)?;

        Ok(SanitizedAccounts {
            vault,
            admin,
            new_admin,
        })
    }
}
