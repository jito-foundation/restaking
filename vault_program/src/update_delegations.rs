use jito_vault_core::{
    config::SanitizedConfig, vault::SanitizedVault, vault_operator_list::SanitizedVaultOperatorList,
};
use solana_program::{
    account_info::AccountInfo, clock::Clock, entrypoint::ProgramResult, pubkey::Pubkey,
    sysvar::Sysvar,
};

pub fn process_update_delegations(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    let config = SanitizedConfig::sanitize(program_id, accounts_iter.next().unwrap(), false)?;
    let vault = SanitizedVault::sanitize(program_id, accounts_iter.next().unwrap(), false)?;

    let mut vault_operator_list = SanitizedVaultOperatorList::sanitize(
        program_id,
        accounts_iter.next().unwrap(),
        true,
        vault.account().key,
    )?;

    let slot = Clock::get()?.slot;
    vault_operator_list
        .vault_operator_list_mut()
        .update_delegations(slot, config.config().epoch_length());

    vault_operator_list.save()?;

    Ok(())
}
