use jito_account_traits::AccountDeserialize;
use jito_jsm_core::loader::load_signer;
use jito_vault_core::config::Config;
use jito_vault_core::vault::Vault;
use jito_vault_sdk::error::VaultError;
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
    Config::load(program_id, config, false)?;
    Vault::load(program_id, vault, false)?;
    load_signer(vault_capacity_admin, false)?;

    // The vault capacity admin shall be the signer of the transaction
    let mut vault_data = vault.data.borrow_mut();
    let vault = Vault::try_from_slice_unchecked_mut(&mut vault_data)?;
    if vault.capacity_admin.ne(vault_capacity_admin.key) {
        msg!("Invalid capacity admin for vault");
        return Err(VaultError::VaultCapacityAdminInvalid.into());
    }

    vault.capacity = capacity;

    Ok(())
}
