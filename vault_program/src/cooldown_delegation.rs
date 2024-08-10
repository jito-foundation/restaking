use jito_account_traits::AccountDeserialize;
use jito_jsm_core::loader::load_signer;
use jito_restaking_core::loader::load_operator;
use jito_vault_core::{
    config::Config,
    loader::{load_config, load_vault, load_vault_delegation_list},
    vault::Vault,
    vault_delegation_list::VaultDelegationList,
};
use solana_program::{
    account_info::AccountInfo, clock::Clock, entrypoint::ProgramResult, msg,
    program_error::ProgramError, pubkey::Pubkey, sysvar::Sysvar,
};

pub fn process_cooldown_delegation(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64,
) -> ProgramResult {
    let [config, vault, operator, vault_delegation_list, vault_delegation_admin] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    load_config(program_id, config, false)?;
    load_vault(program_id, vault, false)?;
    let config_data = config.data.borrow();
    let config = Config::try_from_slice(&config_data)?;
    load_operator(&config.restaking_program, operator, false)?;
    load_vault_delegation_list(program_id, vault_delegation_list, vault, true)?;
    load_signer(vault_delegation_admin, false)?;

    let vault_data = vault.data.borrow();
    let vault = Vault::try_from_slice(&vault_data)?;
    if vault.delegation_admin.ne(vault_delegation_admin.key) {
        msg!("Invalid delegation admin for vault");
        return Err(ProgramError::InvalidAccountData);
    }

    let mut vault_delegation_list_data = vault_delegation_list.data.borrow_mut();
    let vault_delegation_list =
        VaultDelegationList::try_from_slice_mut(&mut vault_delegation_list_data)?;
    if vault_delegation_list.is_update_needed(Clock::get()?.slot, config.epoch_length) {
        msg!("Vault delegation list is not up to date");
        return Err(ProgramError::InvalidAccountData);
    }
    vault_delegation_list.undelegate(*operator.key, amount)?;

    Ok(())
}
