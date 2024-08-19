use jito_account_traits::AccountDeserialize;
use jito_jsm_core::loader::load_associated_token_account;
use jito_vault_core::{config::Config, vault::Vault};
use solana_program::clock::Clock;
use solana_program::sysvar::Sysvar;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
    program_pack::Pack, pubkey::Pubkey,
};
use spl_token::state::Account;

pub fn process_update_vault_balance(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let [config, vault_info, vault_token_account] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    Config::load(program_id, config, false)?;
    let config_data = config.data.borrow();
    let config = Config::try_from_slice(&config_data)?;
    Vault::load(program_id, vault_info, true)?;
    let mut vault_data = vault_info.data.borrow_mut();
    let vault = Vault::try_from_slice_unchecked_mut(&mut vault_data)?;
    load_associated_token_account(vault_token_account, vault_info.key, &vault.supported_mint)?;

    vault.check_update_state_ok(Clock::get()?.slot, config.epoch_length)?;

    // TODO (LB): pay out fee account for any accrued fees to vault fee wallet
    vault.tokens_deposited = Account::unpack(&vault_token_account.data.borrow())?.amount;

    Ok(())
}
