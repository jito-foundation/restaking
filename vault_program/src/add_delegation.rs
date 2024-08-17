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

pub fn process_add_delegation(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64,
) -> ProgramResult {
    let [config, vault, operator, vault_operator_delegation, vault_delegation_admin, payer] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    Config::load(program_id, config, false)?;
    Vault::load(program_id, vault, true)?;
    let config_data = config.data.borrow();
    let config = Config::try_from_slice_unchecked(&config_data)?;
    Operator::load(&config.restaking_program, operator, false)?;
    VaultOperatorDelegation::load(program_id, vault_operator_delegation, vault, operator, true)?;
    load_signer(vault_delegation_admin, false)?;
    load_signer(payer, true)?;

    // The Vault delegation admin shall be the signer of the transaction
    let mut vault_data = vault.data.borrow_mut();
    let vault = Vault::try_from_slice_unchecked_mut(&mut vault_data)?;
    if vault.delegation_admin.ne(vault_delegation_admin.key) {
        msg!("Invalid delegation admin for vault");
        return Err(VaultError::VaultDelegationAdminInvalid.into());
    }

    // The Vault shall be up-to-date before adding delegation
    if vault.is_update_needed(Clock::get()?.slot, config.epoch_length) {
        msg!("Vault update is needed");
        return Err(VaultError::VaultUpdateNeeded.into());
    }

    // The vault shall not over allocate assets for delegation
    // TODO (LB): need to check withdrawable reserve amount to make sure not to over-delegate
    let assets_available_for_staking = vault
        .tokens_deposited
        .checked_sub(vault.delegation_state.total_security()?)
        .ok_or(VaultError::VaultOverflow)?;

    if amount > assets_available_for_staking {
        msg!("Insufficient funds in vault for delegation");
        return Err(VaultError::VaultInsufficientFunds.into());
    }

    let mut vault_operator_delegation_data = vault_operator_delegation.data.borrow_mut();
    let vault_operator_delegation =
        VaultOperatorDelegation::try_from_slice_unchecked_mut(&mut vault_operator_delegation_data)?;

    vault_operator_delegation
        .delegation_state
        .delegate(amount)?;
    vault.delegation_state.delegate(amount)?;

    Ok(())
}
