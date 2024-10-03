use jito_bytemuck::AccountDeserialize;
use jito_jsm_core::loader::load_signer;
use jito_restaking_core::operator::Operator;
use jito_vault_core::{
    config::Config, vault::Vault, vault_operator_delegation::VaultOperatorDelegation,
};
use solana_program::{
    account_info::AccountInfo, clock::Clock, entrypoint::ProgramResult,
    program_error::ProgramError, pubkey::Pubkey, sysvar::Sysvar,
};

/// Cools down a delegation
///
/// Specification:
/// - The amount to cooldown shall be greater than zero
/// - The vault shall be up-to-date
/// - The vault delegation admin shall be a signer on the transaction
/// - The assets enqueued for cooldown shall be subtracted from the staked amount and added to the
///   enqueued for cooldown amount
/// - The vault shall be updated to reflect the cooldown amount and the delegation state shall match the sum of all operator delegations
pub fn process_cooldown_delegation(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64,
) -> ProgramResult {
    let [config, vault_info, operator, vault_operator_delegation, vault_delegation_admin] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    Config::load(program_id, config, false)?;
    Vault::load(program_id, vault_info, true)?;
    let mut vault_data = vault_info.data.borrow_mut();
    let vault = Vault::try_from_slice_unchecked_mut(&mut vault_data)?;
    let config_data = config.data.borrow();
    let config = Config::try_from_slice_unchecked(&config_data)?;
    Operator::load(&config.restaking_program, operator, false)?;
    VaultOperatorDelegation::load(
        program_id,
        vault_operator_delegation,
        vault_info,
        operator,
        true,
    )?;
    let mut vault_operator_delegation_data = vault_operator_delegation.data.borrow_mut();
    let vault_operator_delegation =
        VaultOperatorDelegation::try_from_slice_unchecked_mut(&mut vault_operator_delegation_data)?;
    load_signer(vault_delegation_admin, false)?;

    vault.check_delegation_admin(vault_delegation_admin.key)?;
    vault.check_update_state_ok(Clock::get()?.slot, config.epoch_length())?;
    vault.check_is_paused()?;

    vault_operator_delegation
        .delegation_state
        .cooldown(amount)?;
    vault.delegation_state.cooldown(amount)?;

    Ok(())
}
