use jito_vault_core::{
    config::SanitizedConfig, vault::SanitizedVault, vault_operator_list::SanitizedVaultOperatorList,
};
use solana_program::{
    account_info::AccountInfo, clock::Clock, entrypoint::ProgramResult,
    program_error::ProgramError, pubkey::Pubkey, sysvar::Sysvar,
};

pub fn process_update_delegations(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let SanitizedAccounts {
        config,
        mut vault_operator_list,
    } = SanitizedAccounts::sanitize(program_id, accounts)?;

    let slot = Clock::get()?.slot;
    vault_operator_list
        .vault_operator_list_mut()
        .update_delegations(slot, config.config().epoch_length());

    vault_operator_list.save()?;

    Ok(())
}

struct SanitizedAccounts<'a, 'info> {
    config: SanitizedConfig<'a, 'info>,
    vault_operator_list: SanitizedVaultOperatorList<'a, 'info>,
}

impl<'a, 'info> SanitizedAccounts<'a, 'info> {
    fn sanitize(
        program_id: &Pubkey,
        accounts: &'a [AccountInfo<'info>],
    ) -> Result<SanitizedAccounts<'a, 'info>, ProgramError> {
        let accounts_iter = &mut accounts.iter();

        let config = SanitizedConfig::sanitize(program_id, accounts_iter.next().unwrap(), false)?;
        let vault = SanitizedVault::sanitize(program_id, accounts_iter.next().unwrap(), false)?;

        let vault_operator_list = SanitizedVaultOperatorList::sanitize(
            program_id,
            accounts_iter.next().unwrap(),
            true,
            vault.account().key,
        )?;
        Ok(SanitizedAccounts {
            config,
            vault_operator_list,
        })
    }
}
