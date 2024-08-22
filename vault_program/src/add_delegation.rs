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

/// Process the addition of a delegation: [`jito_vault_sdk::instruction::VaultInstruction::AddDelegation`]
///
/// This instruction handles the process of adding a delegation amount to an operator. It ensures that the vault is
/// up-to-date and that the delegation is authorized by the correct admin. The instruction updates both the vault
/// and the associated operator delegation with the new delegation amount.
pub fn process_add_delegation(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64,
) -> ProgramResult {
    let [config, vault_info, operator, vault_operator_delegation, vault_delegation_admin] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    let clock = Clock::get()?;

    Config::load(program_id, config, false)?;
    let config_data = config.data.borrow();
    let config = Config::try_from_slice_unchecked(&config_data)?;
    Vault::load(program_id, vault_info, true)?;
    let mut vault_data = vault_info.data.borrow_mut();
    let vault = Vault::try_from_slice_unchecked_mut(&mut vault_data)?;
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

    // The Vault delegation admin shall be the signer of the transaction
    vault.check_delegation_admin(vault_delegation_admin.key)?;

    // The Vault shall be up-to-date before adding delegation
    vault.check_update_state_ok(clock.slot, config.epoch_length())?;

    vault.delegate(amount)?;
    vault_operator_delegation
        .delegation_state
        .delegate(amount)?;

    Ok(())
}
