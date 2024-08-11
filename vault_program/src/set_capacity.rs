use jito_account_traits::AccountDeserialize;
use jito_jsm_core::loader::load_signer;
use jito_restaking_core::loader::load_config;
use jito_vault_core::{loader::load_vault, vault::Vault};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
    pubkey::Pubkey,
};

pub fn process_set_deposit_capacity(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    capacity: u64,
) -> ProgramResult {
    let [config, vault, vault_capacity_admin] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    load_config(program_id, config, false)?;
    load_vault(program_id, vault, false)?;
    load_signer(vault_capacity_admin, false)?;

    let mut vault_data = vault.data.borrow_mut();
    let vault = Vault::try_from_slice_mut(&mut vault_data)?;
    if vault.capacity_admin.ne(vault_capacity_admin.key) {
        msg!("Invalid capacity admin for vault");
        return Err(ProgramError::InvalidAccountData);
    }

    vault.capacity = capacity;

    Ok(())
}
