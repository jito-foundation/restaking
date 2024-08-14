use std::mem::size_of;

use jito_account_traits::{AccountDeserialize, Discriminator};
use jito_jsm_core::{
    create_account,
    loader::{load_signer, load_system_program},
    realloc,
};
use jito_vault_core::{
    loader::{load_config, load_vault},
    vault_delegation_list::VaultDelegationList,
};
use solana_program::{
    account_info::AccountInfo,
    clock::Clock,
    entrypoint::{ProgramResult, MAX_PERMITTED_DATA_INCREASE},
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
    rent::Rent,
    system_program,
    sysvar::Sysvar,
};

/// Processes the instruction: [`crate::VaultInstruction::InitializeVaultDelegationList`]
/// This shall be called repeatedly until fully initialized
pub fn process_initialize_vault_delegation_list(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let [config, vault, vault_delegation_list, payer, system_program] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    load_config(program_id, config, false)?;
    load_vault(program_id, vault, false)?;
    load_signer(payer, true)?;
    load_system_program(system_program)?;

    // The vault delegation list account shall be at the canonical PDA
    let (vault_delegation_list_pubkey, vault_delegation_list_bump, mut vault_delegation_list_seeds) =
        VaultDelegationList::find_program_address(program_id, vault.key);
    vault_delegation_list_seeds.push(vec![vault_delegation_list_bump]);
    if vault_delegation_list.key.ne(&vault_delegation_list_pubkey) {
        msg!("Vault delegation list account is not at the correct PDA");
        return Err(ProgramError::InvalidAccountData);
    }

    // The vault delegation list shall not be initialized
    if vault_delegation_list.data_len() > 0
        && vault_delegation_list.data.borrow()[0] == VaultDelegationList::DISCRIMINATOR
    {
        msg!("Vault delegation list account is already initialized");
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    // Each initialization step shall not exceed the maximum permitted data increase
    let allocation_size = 8_usize
        .checked_add(size_of::<VaultDelegationList>())
        .unwrap();
    let allocation_left = allocation_size
        .checked_sub(vault_delegation_list.data_len())
        .unwrap();
    let allocation = if allocation_left > MAX_PERMITTED_DATA_INCREASE {
        MAX_PERMITTED_DATA_INCREASE
    } else {
        allocation_left
    };

    if *vault_delegation_list.owner == system_program::id() {
        msg!(
            "Initializing vault delegation list at address {} (allocation_left: {})",
            vault_delegation_list.key,
            allocation_left
        );
        create_account(
            payer,
            vault_delegation_list,
            system_program,
            program_id,
            &Rent::get()?,
            allocation as u64,
            &vault_delegation_list_seeds,
        )?;
    } else if vault_delegation_list.owner == program_id {
        let new_size = vault_delegation_list
            .data_len()
            .checked_add(allocation)
            .unwrap();
        msg!(
            "Resizing vault delegation list at address {} to {} ({} left)",
            vault_delegation_list.key,
            new_size,
            allocation_left
        );
        realloc(vault_delegation_list, new_size, payer, &Rent::get()?)?;
    } else {
        msg!("Vault delegation list account is not owned by the program");
        return Err(ProgramError::InvalidAccountOwner);
    }

    // When the vault delegation list is fully allocated, initialize it by writing the discriminator
    // to the first byte
    let mut vault_delegation_list_data = vault_delegation_list.try_borrow_mut_data()?;
    if vault_delegation_list_data.len() == allocation_size {
        msg!("Initializing vault delegation list data");
        vault_delegation_list_data[0] = VaultDelegationList::DISCRIMINATOR;
        let vault_delegation_list =
            VaultDelegationList::try_from_slice_mut(&mut vault_delegation_list_data)?;
        vault_delegation_list.vault = *vault.key;
        vault_delegation_list.last_slot_updated = Clock::get()?.slot;
        vault_delegation_list.bump = vault_delegation_list_bump;
    }

    Ok(())
}
