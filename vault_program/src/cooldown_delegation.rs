use jito_account_traits::AccountDeserialize;
use jito_jsm_core::loader::load_signer;
use jito_restaking_core::operator::Operator;
use jito_vault_core::{
    config::Config, vault::Vault, vault_operator_delegation::VaultOperatorDelegation,
};
use jito_vault_sdk::error::VaultError;
use solana_program::{
    account_info::AccountInfo, clock::Clock, entrypoint::ProgramResult, msg,
    program_error::ProgramError, pubkey::Pubkey, sysvar::Sysvar,
};

pub fn process_cooldown_delegation(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64,
    for_withdrawal: bool,
) -> ProgramResult {
    let [config, vault, operator, vault_operator_delegation, vault_delegation_admin] = accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    Config::load(program_id, config, false)?;
    Vault::load(program_id, vault, false)?;
    let config_data = config.data.borrow();
    let config = Config::try_from_slice_unchecked(&config_data)?;
    Operator::load(&config.restaking_program, operator, false)?;
    VaultOperatorDelegation::load(program_id, vault_operator_delegation, vault, operator, true)?;
    load_signer(vault_delegation_admin, false)?;

    // The Vault delegation admin shall be the signer of the transaction
    let mut vault_data = vault.data.borrow_mut();
    let vault = Vault::try_from_slice_unchecked_mut(&mut vault_data)?;
    if vault.delegation_admin.ne(vault_delegation_admin.key) {
        msg!("Invalid delegation admin for vault");
        return Err(VaultError::VaultDelegationAdminInvalid.into());
    }

    // The Vault shall be up-to-date before removing delegation
    if vault.is_update_needed(Clock::get()?.slot, config.epoch_length) {
        msg!("Vault update is needed");
        return Err(VaultError::VaultUpdateNeeded.into());
    }

    let mut vault_operator_delegation_data = vault_operator_delegation.data.borrow_mut();
    let vault_operator_delegation =
        VaultOperatorDelegation::try_from_slice_unchecked_mut(&mut vault_operator_delegation_data)?;

    vault
        .delegation_state
        .subtract(&vault_operator_delegation.delegation_state)?;
    if for_withdrawal {
        vault_operator_delegation
            .delegation_state
            .cooldown_for_withdrawal(amount)?;
    } else {
        vault_operator_delegation
            .delegation_state
            .cooldown(amount)?;
    }
    vault
        .delegation_state
        .accumulate(&vault_operator_delegation.delegation_state)?;

    Ok(())
}
