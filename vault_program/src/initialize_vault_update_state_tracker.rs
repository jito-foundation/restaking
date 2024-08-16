use std::mem::size_of;

use jito_account_traits::{AccountDeserialize, Discriminator};
use jito_jsm_core::{
    create_account,
    loader::{load_signer, load_system_account, load_system_program},
};
use jito_vault_core::{
    config::Config,
    loader::{load_config, load_vault},
    vault::Vault,
    vault_update_state_tracker::VaultUpdateStateTracker,
};
use jito_vault_sdk::error::VaultError;
use solana_program::{
    account_info::AccountInfo, clock::Clock, entrypoint::ProgramResult, msg,
    program_error::ProgramError, pubkey::Pubkey, rent::Rent, sysvar::Sysvar,
};

/// Instruction: [`crate::VaultInstruction::InitializeVaultUpdateDelegationsTicket`]
/// Initializes a new [`VaultUpdateStateTracker`] account, which is used to track the delegations
/// that are to be updated at the epoch boundary.
pub fn process_initialize_vault_update_state_tracker(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let [config, vault_info, vault_update_state_tracker, payer, system_program] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    load_config(program_id, config, false)?;
    load_vault(program_id, vault_info, true)?;
    load_system_account(vault_update_state_tracker, true)?;
    load_signer(payer, true)?;
    load_system_program(system_program)?;

    let config_data = config.data.borrow();
    let config = Config::try_from_slice(&config_data)?;

    // The vault update state tracker shall not be initialized if an update is not needed
    let vault_data = vault_info.data.borrow();
    let vault = Vault::try_from_slice(&vault_data)?;
    if !vault.is_update_needed(Clock::get()?.slot, config.epoch_length) {
        msg!("Vault is up-to-date");
        return Err(VaultError::VaultIsUpdated.into());
    }

    let ncn_epoch = Clock::get()?.slot.checked_div(config.epoch_length).unwrap();

    // The VaultUpdateStateTracker shall be at the canonical PDA
    let (
        vault_update_state_tracker_pubkey,
        vault_update_state_tracker_bump,
        mut vault_update_state_tracker_seeds,
    ) = VaultUpdateStateTracker::find_program_address(program_id, vault_info.key, ncn_epoch);
    vault_update_state_tracker_seeds.push(vec![vault_update_state_tracker_bump]);
    if vault_update_state_tracker_pubkey.ne(vault_update_state_tracker.key) {
        msg!("Vault update delegations ticket is not at the correct PDA");
        return Err(ProgramError::InvalidAccountData);
    }

    msg!(
        "Initializing VaultUpdateDelegationsTicket at address {}",
        vault_update_state_tracker.key
    );
    create_account(
        payer,
        vault_update_state_tracker,
        system_program,
        program_id,
        &Rent::get()?,
        8_u64
            .checked_add(size_of::<VaultUpdateStateTracker>() as u64)
            .unwrap(),
        &vault_update_state_tracker_seeds,
    )?;

    let mut vault_update_state_tracker_data = vault_update_state_tracker.try_borrow_mut_data()?;
    vault_update_state_tracker_data[0] = VaultUpdateStateTracker::DISCRIMINATOR;
    let vault_update_state_tracker =
        VaultUpdateStateTracker::try_from_slice_mut(&mut vault_update_state_tracker_data)?;
    *vault_update_state_tracker = VaultUpdateStateTracker::new(*vault_info.key, ncn_epoch);

    Ok(())
}
