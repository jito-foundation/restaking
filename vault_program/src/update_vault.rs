use jito_account_traits::AccountDeserialize;
use jito_jsm_core::loader::load_associated_token_account;
use jito_vault_core::{
    config::Config,
    loader::{load_config, load_vault, load_vault_delegation_list},
    vault::Vault,
    vault_delegation_list::{VaultDelegationList, VaultDelegationUpdateSummary},
};
use solana_program::{
    account_info::AccountInfo, clock::Clock, entrypoint::ProgramResult,
    program_error::ProgramError, program_pack::Pack, pubkey::Pubkey, sysvar::Sysvar,
};
use spl_token::state::Account;

pub fn process_update_vault(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let [config, vault_info, vault_delegation_list, vault_token_account] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    load_config(program_id, config, false)?;
    load_vault(program_id, vault_info, true)?;
    load_vault_delegation_list(program_id, vault_delegation_list, vault_info, true)?;
    let mut vault_data = vault_info.data.borrow_mut();
    let vault = Vault::try_from_slice_mut(&mut vault_data)?;
    load_associated_token_account(vault_token_account, vault_info.key, &vault.supported_mint)?;

    let config_data = config.data.borrow();
    let config = Config::try_from_slice(&config_data)?;

    let mut vault_delegation_list_data = vault_delegation_list.data.borrow_mut();
    let vault_delegation_list =
        VaultDelegationList::try_from_slice_mut(&mut vault_delegation_list_data)?;

    // Update the vault delegation list
    vault_delegation_list.update(Clock::get()?.slot, config.epoch_length)?;

    // Update the total amount of tokens
    let vault_token_account_data = vault_token_account.data.borrow();
    let vault_token_account = Account::unpack(&vault_token_account_data)?;
    vault.tokens_deposited = vault_token_account.amount;

    Ok(())
}
