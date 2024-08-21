use jito_bytemuck::AccountDeserialize;
use jito_jsm_core::loader::load_signer;
use jito_vault_core::{config::Config, vault::Vault};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
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
    let mut vault_data = vault.data.borrow_mut();
    let vault = Vault::try_from_slice_unchecked_mut(&mut vault_data)?;
    load_signer(vault_capacity_admin, false)?;

    vault.check_capacity_admin(vault_capacity_admin.key)?;
    vault.set_capacity(capacity);

    Ok(())
}
