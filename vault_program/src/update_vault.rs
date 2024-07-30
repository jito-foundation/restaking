use jito_restaking_sanitization::associated_token_account::SanitizedAssociatedTokenAccount;
use jito_vault_core::{
    config::SanitizedConfig, vault::SanitizedVault,
    vault_delegation_list::SanitizedVaultDelegationList,
};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::Clock,
    entrypoint::ProgramResult,
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvar::Sysvar,
};

pub fn process_update_vault(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let SanitizedAccounts {
        config,
        mut vault,
        mut vault_delegation_list,
        vault_token_account,
    } = SanitizedAccounts::sanitize(program_id, accounts)?;

    let slot = Clock::get()?.slot;
    vault_delegation_list
        .vault_delegation_list_mut()
        .update(slot, config.config().epoch_length())?;

    vault
        .vault_mut()
        .set_tokens_deposited(vault_token_account.token_account().amount);

    vault.save()?;
    vault_delegation_list.save()?;

    Ok(())
}

struct SanitizedAccounts<'a, 'info> {
    config: SanitizedConfig<'a, 'info>,
    vault: SanitizedVault<'a, 'info>,
    vault_delegation_list: SanitizedVaultDelegationList<'a, 'info>,
    vault_token_account: SanitizedAssociatedTokenAccount<'a, 'info>,
}

impl<'a, 'info> SanitizedAccounts<'a, 'info> {
    fn sanitize(
        program_id: &Pubkey,
        accounts: &'a [AccountInfo<'info>],
    ) -> Result<SanitizedAccounts<'a, 'info>, ProgramError> {
        let accounts_iter = &mut accounts.iter();

        let config =
            SanitizedConfig::sanitize(program_id, next_account_info(accounts_iter)?, false)?;
        let vault = SanitizedVault::sanitize(program_id, next_account_info(accounts_iter)?, true)?;

        let vault_delegation_list = SanitizedVaultDelegationList::sanitize(
            program_id,
            next_account_info(accounts_iter)?,
            true,
            vault.account().key,
        )?;
        let vault_token_account = SanitizedAssociatedTokenAccount::sanitize(
            next_account_info(accounts_iter)?,
            &vault.vault().supported_mint(),
            vault.account().key,
        )?;

        Ok(SanitizedAccounts {
            config,
            vault,
            vault_delegation_list,
            vault_token_account,
        })
    }
}
