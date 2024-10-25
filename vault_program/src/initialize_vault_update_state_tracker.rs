use std::mem::size_of;

use jito_bytemuck::{AccountDeserialize, Discriminator};
use jito_jsm_core::{
    create_account,
    loader::{load_signer, load_system_account, load_system_program},
};
use jito_vault_core::{
    config::Config, vault::Vault, vault_update_state_tracker::VaultUpdateStateTracker,
};
use jito_vault_sdk::{error::VaultError, instruction::WithdrawalAllocationMethod};
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
    withdrawal_allocation_method: WithdrawalAllocationMethod,
) -> ProgramResult {
    let [config, vault_info, vault_update_state_tracker, payer, system_program] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    Config::load(program_id, config, false)?;
    let config_data = config.data.borrow();
    let config = Config::try_from_slice_unchecked(&config_data)?;
    Vault::load(program_id, vault_info, true)?;
    let mut vault_data = vault_info.data.borrow_mut();
    let vault = Vault::try_from_slice_unchecked_mut(&mut vault_data)?;
    load_system_account(vault_update_state_tracker, true)?;
    load_signer(payer, true)?;
    load_system_program(system_program)?;

    // The VaultUpdateStateTracker shall be at the canonical PDA
    let slot = Clock::get()?.slot;
    let ncn_epoch = config.get_epoch_from_slot(slot)?;

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

    if vault
        .check_update_state_ok(Clock::get()?.slot, config.epoch_length())
        .is_ok()
    {
        msg!("Vault update state tracker is not needed");
        return Err(VaultError::VaultIsUpdated.into());
    }
    vault.check_is_paused()?;

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
            .ok_or(VaultError::ArithmeticOverflow)?,
        &vault_update_state_tracker_seeds,
    )?;

    let additional_assets_need_unstaking = if vault.additional_assets_need_unstaking() > 0 {
        // indicates an old partially cranked state
        // no vault operations can be done until a full update cycle has run
        // this means that the `additional_assets_need_unstaking` will not change
        // between epochs and we can effectively carry on where we left off
        vault.additional_assets_need_unstaking()
    } else {
        // If the vault is not in the middle of unstaking, calculate the additional assets needed
        // to unstake
        vault.calculate_additional_supported_assets_needed_to_unstake(
            Clock::get()?.slot,
            config.epoch_length(),
            config.program_fee_bps(),
        )?
    };

    vault.set_additional_assets_need_unstaking(additional_assets_need_unstaking);

    let mut vault_update_state_tracker_data = vault_update_state_tracker.try_borrow_mut_data()?;
    vault_update_state_tracker_data[0] = VaultUpdateStateTracker::DISCRIMINATOR;
    let vault_update_state_tracker = VaultUpdateStateTracker::try_from_slice_unchecked_mut(
        &mut vault_update_state_tracker_data,
    )?;
    *vault_update_state_tracker = VaultUpdateStateTracker::new(
        *vault_info.key,
        ncn_epoch,
        withdrawal_allocation_method as u8,
    );

    Ok(())
}
