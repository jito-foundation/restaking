use std::mem::size_of;

use jito_account_traits::{AccountDeserialize, Discriminator};
use jito_jsm_core::{
    create_account,
    loader::{load_signer, load_system_account, load_system_program},
};
use jito_restaking_core::{operator::Operator, operator_vault_ticket::OperatorVaultTicket};
use jito_vault_core::{
    config::Config, vault::Vault, vault_operator_delegation::VaultOperatorDelegation,
};
use jito_vault_sdk::error::VaultError;
use solana_program::{
    account_info::AccountInfo, clock::Clock, entrypoint::ProgramResult, msg,
    program_error::ProgramError, pubkey::Pubkey, rent::Rent, sysvar::Sysvar,
};

/// Instruction: [`crate::VaultInstruction::InitializeVaultOperatorDelegation`]
pub fn process_initialize_vault_operator_delegation(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let [config, vault_info, operator, operator_vault_ticket, vault_operator_delegation, vault_operator_admin, payer, system_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    Config::load(program_id, config, false)?;
    Vault::load(program_id, vault_info, false)?;
    let config_data = config.data.borrow();
    let config = Config::try_from_slice_unchecked(&config_data)?;
    Operator::load(&config.restaking_program, operator, false)?;
    OperatorVaultTicket::load(
        &config.restaking_program,
        operator_vault_ticket,
        operator,
        vault_info,
        false,
    )?;
    load_system_account(vault_operator_delegation, true)?;
    load_signer(vault_operator_admin, false)?;
    load_signer(payer, true)?;
    load_system_program(system_program)?;

    // The VaultOperatorDelegation shall be at the canonical PDA
    let (
        vault_operator_delegation_pubkey,
        vault_operator_delegation_bump,
        mut vault_operator_delegation_seeds,
    ) = VaultOperatorDelegation::find_program_address(program_id, vault_info.key, operator.key);
    vault_operator_delegation_seeds.push(vec![vault_operator_delegation_bump]);
    if vault_operator_delegation_pubkey.ne(vault_operator_delegation.key) {
        msg!("Vault operator ticket is not at the correct PDA");
        return Err(ProgramError::InvalidAccountData);
    }

    // The vault operator admin shall be a signer on the transaction
    let mut vault_data = vault_info.data.borrow_mut();
    let vault = Vault::try_from_slice_unchecked_mut(&mut vault_data)?;
    if vault.operator_admin.ne(vault_operator_admin.key) {
        msg!("Invalid operator admin for vault");
        return Err(VaultError::VaultOperatorAdminInvalid.into());
    }

    // The Vault shall be up-to-date before adding the operator
    if vault.is_update_needed(Clock::get()?.slot, config.epoch_length) {
        msg!("Vault update is needed");
        return Err(VaultError::VaultUpdateNeeded.into());
    }

    msg!(
        "Initializing VaultOperatorDelegation at address {}",
        vault_operator_delegation.key
    );
    create_account(
        payer,
        vault_operator_delegation,
        system_program,
        program_id,
        &Rent::get()?,
        8_u64
            .checked_add(size_of::<VaultOperatorDelegation>() as u64)
            .unwrap(),
        &vault_operator_delegation_seeds,
    )?;

    let mut vault_operator_delegation_data = vault_operator_delegation.try_borrow_mut_data()?;
    vault_operator_delegation_data[0] = VaultOperatorDelegation::DISCRIMINATOR;
    let vault_operator_delegation =
        VaultOperatorDelegation::try_from_slice_unchecked_mut(&mut vault_operator_delegation_data)?;
    *vault_operator_delegation = VaultOperatorDelegation::new(
        *vault_info.key,
        *operator.key,
        vault.operator_count,
        vault_operator_delegation_bump,
    );

    vault.operator_count = vault
        .operator_count
        .checked_add(1)
        .ok_or(ProgramError::InvalidAccountData)?;

    Ok(())
}
