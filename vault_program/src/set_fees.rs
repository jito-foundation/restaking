use jito_account_traits::AccountDeserialize;
use jito_jsm_core::loader::load_signer;
use jito_restaking_core::loader::load_config;
use jito_vault_core::{config::Config, loader::load_vault, vault::Vault};
use jito_vault_sdk::error::VaultError;
use solana_program::sysvar::Sysvar;
use solana_program::{
    account_info::AccountInfo, clock::Clock, entrypoint::ProgramResult, msg,
    program_error::ProgramError, pubkey::Pubkey,
};

pub fn process_set_fees(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    deposit_fee_bps: u16,
    withdrawal_fee_bps: u16,
) -> ProgramResult {
    let [config, vault, vault_fee_admin] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    load_config(program_id, config, false)?;
    load_vault(program_id, vault, false)?;
    load_signer(vault_fee_admin, false)?;

    // Fee cannot be larger than the fee cap set in the config
    let mut config_data = config.data.borrow_mut();
    let config = Config::try_from_slice_mut(&mut config_data)?;

    if deposit_fee_bps > config.fee_cap_bps || withdrawal_fee_bps > config.fee_cap_bps {
        msg!("Fee cap exceeds maximum allowed of {}", config.fee_cap_bps);
        return Err(VaultError::VaultFeeCapExceeded.into());
    }

    // The vault capacity admin shall be the signer of the transaction
    let mut vault_data = vault.data.borrow_mut();
    let vault = Vault::try_from_slice_mut(&mut vault_data)?;

    if vault.fee_admin.ne(vault_fee_admin.key) {
        msg!("Invalid fee admin for vault");
        return Err(VaultError::VaultFeeAdminInvalid.into());
    }

    // Fees changes have a cooldown of 1 full epoch
    let current_slot = Clock::get()?.slot;
    let epoch_length = config.epoch_length;
    let current_epoch = current_slot.checked_div(epoch_length).unwrap();
    let last_fee_change_epoch = vault
        .last_fee_change_slot
        .checked_div(epoch_length)
        .unwrap();

    if current_epoch <= last_fee_change_epoch + 1 {
        msg!("Fee changes are only allowed once per epoch");
        return Err(VaultError::VaultFeeChangeTooSoon.into());
    }

    vault.deposit_fee_bps = deposit_fee_bps;
    vault.withdrawal_fee_bps = withdrawal_fee_bps;
    vault.last_fee_change_slot = current_slot;

    Ok(())
}
