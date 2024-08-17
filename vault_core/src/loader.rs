//! Loader functions for the vault program.
use jito_account_traits::{AccountDeserialize, Discriminator};
use jito_vault_sdk::inline_mpl_token_metadata::{self, pda::find_metadata_account};
use solana_program::{account_info::AccountInfo, msg, program_error::ProgramError, pubkey::Pubkey};

use crate::{
    config::Config, vault::Vault, vault_ncn_slasher_operator_ticket::VaultNcnSlasherOperatorTicket,
    vault_ncn_slasher_ticket::VaultNcnSlasherTicket, vault_ncn_ticket::VaultNcnTicket,
    vault_operator_delegation::VaultOperatorDelegation,
    vault_staker_withdrawal_ticket::VaultStakerWithdrawalTicket,
    vault_update_state_tracker::VaultUpdateStateTracker,
};

/// Loads the vault [`Config`] account
///
/// # Arguments
/// * `program_id` - The program ID
/// * `account` - The account to load
/// * `expect_writable` - Whether the account should be writable
///
/// # Returns
/// * `Result<(), ProgramError>` - The result of the operation
pub fn load_config(
    program_id: &Pubkey,
    account: &AccountInfo,
    expect_writable: bool,
) -> Result<(), ProgramError> {
    if account.owner.ne(program_id) {
        msg!("Config account has an invalid owner");
        return Err(ProgramError::InvalidAccountOwner);
    }
    if account.data_is_empty() {
        msg!("Config account data is empty");
        return Err(ProgramError::InvalidAccountData);
    }
    if expect_writable && !account.is_writable {
        msg!("Config account is not writable");
        return Err(ProgramError::InvalidAccountData);
    }
    if account.data.borrow()[0].ne(&Config::DISCRIMINATOR) {
        msg!("Config account discriminator is invalid");
        return Err(ProgramError::InvalidAccountData);
    }
    if account.key.ne(&Config::find_program_address(program_id).0) {
        msg!("Config account is not at the correct PDA");
        return Err(ProgramError::InvalidAccountData);
    }
    Ok(())
}

/// Loads the [`Vault`] account
///
/// # Arguments
/// * `program_id` - The program ID
/// * `account` - The account to load
/// * `expect_writable` - Whether the account should be writable
///
/// # Returns
/// * `Result<(), ProgramError>` - The result of the operation
pub fn load_vault(
    program_id: &Pubkey,
    account: &AccountInfo,
    expect_writable: bool,
) -> Result<(), ProgramError> {
    if account.owner.ne(program_id) {
        msg!("Vault account has an invalid owner");
        return Err(ProgramError::InvalidAccountOwner);
    }
    if account.data_is_empty() {
        msg!("Vault account data is empty");
        return Err(ProgramError::InvalidAccountData);
    }
    if expect_writable && !account.is_writable {
        msg!("Vault account is not writable");
        return Err(ProgramError::InvalidAccountData);
    }
    if account.data.borrow()[0].ne(&Vault::DISCRIMINATOR) {
        msg!("Vault account discriminator is invalid");
        return Err(ProgramError::InvalidAccountData);
    }
    let base = Vault::try_from_slice(&account.data.borrow())?.base;
    if account
        .key
        .ne(&Vault::find_program_address(program_id, &base).0)
    {
        msg!("Vault account is not at the correct PDA");
        return Err(ProgramError::InvalidAccountData);
    }
    Ok(())
}

/// Loads the [`VaultNcnTicket`] account
///
/// # Arguments
/// * `program_id` - The program ID
/// * `vault_ncn_ticket` - The account to load
/// * `vault` - The vault account
/// * `ncn` - The ncn account
/// * `expect_writable` - Whether the account should be writable
///
/// # Returns
/// * `Result<(), ProgramError>` - The result of the operation
pub fn load_vault_ncn_ticket(
    program_id: &Pubkey,
    vault_ncn_ticket: &AccountInfo,
    vault: &AccountInfo,
    ncn: &AccountInfo,
    expect_writable: bool,
) -> Result<(), ProgramError> {
    if vault_ncn_ticket.owner.ne(program_id) {
        msg!("Vault NCN ticket account has an invalid owner");
        return Err(ProgramError::InvalidAccountOwner);
    }
    if vault_ncn_ticket.data_is_empty() {
        msg!("Vault NCN ticket account data is empty");
        return Err(ProgramError::InvalidAccountData);
    }
    if expect_writable && !vault_ncn_ticket.is_writable {
        msg!("Vault NCN ticket account is not writable");
        return Err(ProgramError::InvalidAccountData);
    }
    if vault_ncn_ticket.data.borrow()[0].ne(&VaultNcnTicket::DISCRIMINATOR) {
        msg!("Vault NCN ticket account discriminator is invalid");
        return Err(ProgramError::InvalidAccountData);
    }
    let expected_pubkey = VaultNcnTicket::find_program_address(program_id, vault.key, ncn.key).0;
    if vault_ncn_ticket.key.ne(&expected_pubkey) {
        msg!("Vault NCN ticket account is not at the correct PDA");
        return Err(ProgramError::InvalidAccountData);
    }
    Ok(())
}

/// Loads the [`VaultOperatorDelegation`] account
///
/// # Arguments
/// * `program_id` - The program ID
/// * `vault_operator_delegation` - The [`VaultOperatorDelegation`] account
/// * `vault` - The vault account
/// * `operator` - The operator account
/// * `expect_writable` - Whether the account should be writable
///
/// # Returns
/// * `Result<(), ProgramError>` - The result of the operation
pub fn load_vault_operator_delegation(
    program_id: &Pubkey,
    vault_operator_delegation: &AccountInfo,
    vault: &AccountInfo,
    operator: &AccountInfo,
    expect_writable: bool,
) -> Result<(), ProgramError> {
    if vault_operator_delegation.owner.ne(program_id) {
        msg!("Vault operator ticket account has an invalid owner");
        return Err(ProgramError::InvalidAccountOwner);
    }
    if vault_operator_delegation.data_is_empty() {
        msg!("Vault operator ticket account data is empty");
        return Err(ProgramError::InvalidAccountData);
    }
    if expect_writable && !vault_operator_delegation.is_writable {
        msg!("Vault operator ticket account is not writable");
        return Err(ProgramError::InvalidAccountData);
    }
    if vault_operator_delegation.data.borrow()[0].ne(&VaultOperatorDelegation::DISCRIMINATOR) {
        msg!("Vault operator ticket account discriminator is invalid");
        return Err(ProgramError::InvalidAccountData);
    }
    let expected_pubkey =
        VaultOperatorDelegation::find_program_address(program_id, vault.key, operator.key).0;
    if vault_operator_delegation.key.ne(&expected_pubkey) {
        msg!("Vault operator ticket account is not at the correct PDA");
        return Err(ProgramError::InvalidAccountData);
    }
    Ok(())
}

/// Loads the [`VaultNcnSlasherTicket`] account
///
/// # Arguments
/// * `program_id` - The program ID
/// * `vault_ncn_slasher_ticket` - The [`VaultNcnSlasherTicket`] account
/// * `vault` - The [`Vault`] account
/// * `ncn` - The ncn account
/// * `slasher` - The slasher account
/// * `expect_writable` - Whether the account should be writable
///
/// # Returns
/// * `Result<(), ProgramError>` - The result of the operation
pub fn load_vault_ncn_slasher_ticket(
    program_id: &Pubkey,
    vault_ncn_slasher_ticket: &AccountInfo,
    vault: &AccountInfo,
    ncn: &AccountInfo,
    slasher: &AccountInfo,
    expect_writable: bool,
) -> Result<(), ProgramError> {
    if vault_ncn_slasher_ticket.owner.ne(program_id) {
        msg!("Vault NCN slasher ticket account has an invalid owner");
        return Err(ProgramError::InvalidAccountOwner);
    }
    if vault_ncn_slasher_ticket.data_is_empty() {
        msg!("Vault NCN slasher ticket account data is empty");
        return Err(ProgramError::InvalidAccountData);
    }
    if expect_writable && !vault_ncn_slasher_ticket.is_writable {
        msg!("Vault NCN slasher ticket account is not writable");
        return Err(ProgramError::InvalidAccountData);
    }
    if vault_ncn_slasher_ticket.data.borrow()[0].ne(&VaultNcnSlasherTicket::DISCRIMINATOR) {
        msg!("Vault NCN slasher ticket account discriminator is invalid");
        return Err(ProgramError::InvalidAccountData);
    }
    let expected_pubkey =
        VaultNcnSlasherTicket::find_program_address(program_id, vault.key, ncn.key, slasher.key).0;
    if vault_ncn_slasher_ticket.key.ne(&expected_pubkey) {
        msg!("Vault NCN slasher ticket account is not at the correct PDA");
        return Err(ProgramError::InvalidAccountData);
    }
    Ok(())
}

/// Loads the [`VaultNcnSlasherOperatorTicket`] account
///
/// # Arguments
/// * `program_id` - The program ID
/// * `vault_ncn_slasher_operator_ticket` - The [`VaultNcnSlasherOperatorTicket`] account
/// * `vault` - The [`Vault`] account
/// * `ncn` - The ncn account
/// * `slasher` - The slasher account
/// * `operator` - The operator account
/// * `ncn_epoch` - The ncn epoch
/// * `expect_writable` - Whether the account should be writable
///
/// # Returns
/// * `Result<(), ProgramError>` - The result of the operation
#[allow(clippy::too_many_arguments)]
pub fn load_vault_ncn_slasher_operator_ticket(
    program_id: &Pubkey,
    vault_ncn_slasher_operator_ticket: &AccountInfo,
    vault: &AccountInfo,
    ncn: &AccountInfo,
    slasher: &AccountInfo,
    operator: &AccountInfo,
    ncn_epoch: u64,
    expect_writable: bool,
) -> Result<(), ProgramError> {
    if vault_ncn_slasher_operator_ticket.owner.ne(program_id) {
        msg!("Vault NCN slasher operator has an invalid owner");
        return Err(ProgramError::InvalidAccountOwner);
    }
    if vault_ncn_slasher_operator_ticket.data_is_empty() {
        msg!("Vault NCN slasher operator data is empty");
        return Err(ProgramError::InvalidAccountData);
    }
    if expect_writable && !vault_ncn_slasher_operator_ticket.is_writable {
        msg!("Vault NCN slasher operator is not writable");
        return Err(ProgramError::InvalidAccountData);
    }
    if vault_ncn_slasher_operator_ticket.data.borrow()[0]
        .ne(&VaultNcnSlasherOperatorTicket::DISCRIMINATOR)
    {
        msg!("Vault NCN slasher operator discriminator is invalid");
        return Err(ProgramError::InvalidAccountData);
    }
    let expected_pubkey = VaultNcnSlasherOperatorTicket::find_program_address(
        program_id,
        vault.key,
        ncn.key,
        slasher.key,
        operator.key,
        ncn_epoch,
    )
    .0;
    if vault_ncn_slasher_operator_ticket.key.ne(&expected_pubkey) {
        msg!("Vault NCN slasher operator is not at the correct PDA");
        return Err(ProgramError::InvalidAccountData);
    }
    Ok(())
}

/// Loads the [`VaultStakerWithdrawalTicket`] account
///
/// # Arguments
/// * `program_id` - The program ID
/// * `vault_staker_withdrawal_ticket` - The [`VaultStakerWithdrawalTicket`] account
/// * `vault` - The [`Vault`] account
/// * `staker` - The staker account
/// * `expect_writable` - Whether the account should be writable
///
/// # Returns
/// * `Result<(), ProgramError>` - The result of the operation
pub fn load_vault_staker_withdrawal_ticket(
    program_id: &Pubkey,
    vault_staker_withdrawal_ticket: &AccountInfo,
    vault: &AccountInfo,
    staker: &AccountInfo,
    expect_writable: bool,
) -> Result<(), ProgramError> {
    if vault_staker_withdrawal_ticket.owner.ne(program_id) {
        msg!("Vault staker withdraw ticket has an invalid owner");
        return Err(ProgramError::InvalidAccountOwner);
    }
    if vault_staker_withdrawal_ticket.data_is_empty() {
        msg!("Vault staker withdraw ticket data is empty");
        return Err(ProgramError::InvalidAccountData);
    }
    if expect_writable && !vault_staker_withdrawal_ticket.is_writable {
        msg!("Vault staker withdraw ticket is not writable");
        return Err(ProgramError::InvalidAccountData);
    }
    if vault_staker_withdrawal_ticket.data.borrow()[0]
        .ne(&VaultStakerWithdrawalTicket::DISCRIMINATOR)
    {
        msg!("Vault staker withdraw ticket discriminator is invalid");
        return Err(ProgramError::InvalidAccountData);
    }
    let vault_staker_withdraw_ticket_data = vault_staker_withdrawal_ticket.data.borrow();
    let base =
        VaultStakerWithdrawalTicket::try_from_slice(&vault_staker_withdraw_ticket_data)?.base;
    let expected_pubkey =
        VaultStakerWithdrawalTicket::find_program_address(program_id, vault.key, staker.key, &base)
            .0;
    if vault_staker_withdrawal_ticket.key.ne(&expected_pubkey) {
        msg!("Vault staker withdraw ticket is not at the correct PDA");
        return Err(ProgramError::InvalidAccountData);
    }
    Ok(())
}

/// Loads the account as a mpl metadata program, returning an error if it is not.
///
/// # Arguments
/// * `info` - The account to load the mpl metadata program from
///
/// # Returns
/// * `Result<(), ProgramError>` - The result of the operation
pub fn load_mpl_metadata_program(info: &AccountInfo) -> Result<(), ProgramError> {
    if info.key.ne(&inline_mpl_token_metadata::id()) {
        msg!(
            "Expected mpl metadata program {}, received {}",
            inline_mpl_token_metadata::id(),
            info.key
        );
        return Err(ProgramError::IncorrectProgramId);
    }

    Ok(())
}

/// Loads the account as a mpl metadata account, returning an error if it is not.
///
/// # Arguments
/// * `info` - The account to load the mpl metadata program from
///
/// # Returns
/// * `Result<(), ProgramError>` - The result of the operation
pub fn load_mpl_metadata(info: &AccountInfo, vrt_mint: &Pubkey) -> Result<(), ProgramError> {
    let (metadata_account_pubkey, _) = find_metadata_account(vrt_mint);

    if metadata_account_pubkey.ne(info.key) {
        Err(ProgramError::InvalidAccountData)
    } else {
        Ok(())
    }
}

pub fn load_vault_update_state_tracker(
    program_id: &Pubkey,
    vault_update_delegation_ticket: &AccountInfo,
    vault: &AccountInfo,
    ncn_epoch: u64,
    expect_writable: bool,
) -> Result<(), ProgramError> {
    if vault_update_delegation_ticket.owner.ne(program_id) {
        msg!("Vault update delegations ticket has an invalid owner");
        return Err(ProgramError::InvalidAccountOwner);
    }
    if vault_update_delegation_ticket.data_is_empty() {
        msg!("Vault update delegations ticket data is empty");
        return Err(ProgramError::InvalidAccountData);
    }
    if expect_writable && !vault_update_delegation_ticket.is_writable {
        msg!("Vault update delegations ticket is not writable");
        return Err(ProgramError::InvalidAccountData);
    }
    if vault_update_delegation_ticket.data.borrow()[0].ne(&VaultUpdateStateTracker::DISCRIMINATOR) {
        msg!("Vault update delegations ticket discriminator is invalid");
        return Err(ProgramError::InvalidAccountData);
    }
    let expected_pubkey =
        VaultUpdateStateTracker::find_program_address(program_id, vault.key, ncn_epoch).0;
    if vault_update_delegation_ticket.key.ne(&expected_pubkey) {
        msg!("Vault update delegations ticket is not at the correct PDA");
        return Err(ProgramError::InvalidAccountData);
    }
    Ok(())
}
