use jito_account_traits::AccountDeserialize;
use jito_jsm_core::loader::load_associated_token_account;
use jito_vault_core::{
    loader::{load_config, load_vault},
    vault::Vault,
};
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

    load_config(program_id, config, false)?;
    load_vault(program_id, vault_info, true)?;
    let mut vault_data = vault_info.data.borrow_mut();
    let vault = Vault::try_from_slice_mut(&mut vault_data)?;
    load_associated_token_account(vault_token_account, vault_info.key, &vault.supported_mint)?;

    // TODO (LB): pay out fee account for any accrued fees to vault fee wallet
    vault.tokens_deposited = Account::unpack(&vault_token_account.data.borrow())?.amount;

    Ok(())
}
