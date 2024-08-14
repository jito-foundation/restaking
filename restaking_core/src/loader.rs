//! Loader functions for the restaking program
//! Thank you to HardhatChad for the inspiration with Ore account loading
use jito_account_traits::{AccountDeserialize, Discriminator};
use solana_program::{account_info::AccountInfo, msg, program_error::ProgramError, pubkey::Pubkey};

use crate::{
    config::Config, ncn::Ncn, ncn_operator_ticket::NcnOperatorTicket,
    ncn_vault_slasher_ticket::NcnVaultSlasherTicket, ncn_vault_ticket::NcnVaultTicket,
    operator::Operator, operator_ncn_ticket::OperatorNcnTicket,
    operator_vault_ticket::OperatorVaultTicket,
};

/// Attempts to load the account as [`Config`], returning an error if it's not valid.
///
/// # Arguments
/// * `program_id` - The program ID
/// * `account` - The account to load the configuration from
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

/// Attempts to load the account as [`Ncn`], returning an error if it's not valid.
///
/// # Arguments
/// * `program_id` - The program ID
/// * `account` - The account to load the NCN from
/// * `expect_writable` - Whether the account should be writable
///
/// # Returns
/// * `Result<(), ProgramError>` - The result of the operation
pub fn load_ncn(
    program_id: &Pubkey,
    account: &AccountInfo,
    expect_writable: bool,
) -> Result<(), ProgramError> {
    if account.owner.ne(program_id) {
        msg!("NCN account has an invalid owner");
        return Err(ProgramError::InvalidAccountOwner);
    }
    if account.data_is_empty() {
        msg!("NCN account data is empty");
        return Err(ProgramError::InvalidAccountData);
    }
    if expect_writable && !account.is_writable {
        msg!("NCN account is not writable");
        return Err(ProgramError::InvalidAccountData);
    }
    if account.data.borrow()[0].ne(&Ncn::DISCRIMINATOR) {
        msg!("NCN account discriminator is invalid");
        return Err(ProgramError::InvalidAccountData);
    }
    let base = Ncn::try_from_slice(&account.data.borrow())?.base;
    if account
        .key
        .ne(&Ncn::find_program_address(program_id, &base).0)
    {
        msg!("NCN account is not at the correct PDA");
        return Err(ProgramError::InvalidAccountData);
    }
    Ok(())
}

/// Attempts to load the account as [`Operator`], returning an error if it's not valid.
///
/// # Arguments
/// * `program_id` - The program ID
/// * `account` - The account to load the operator from
/// * `expect_writable` - Whether the account should be writable
///
/// # Returns
/// * `Result<(), ProgramError>` - The result of the operation
pub fn load_operator(
    program_id: &Pubkey,
    account: &AccountInfo,
    expect_writable: bool,
) -> Result<(), ProgramError> {
    if account.owner.ne(program_id) {
        msg!("Operator account has an invalid owner");
        return Err(ProgramError::InvalidAccountOwner);
    }
    if account.data_is_empty() {
        msg!("Operator account data is empty");
        return Err(ProgramError::InvalidAccountData);
    }
    if expect_writable && !account.is_writable {
        msg!("Operator account is not writable");
        return Err(ProgramError::InvalidAccountData);
    }
    if account.data.borrow()[0].ne(&Operator::DISCRIMINATOR) {
        msg!("Operator account discriminator is invalid");
        return Err(ProgramError::InvalidAccountData);
    }
    let base = Operator::try_from_slice(&account.data.borrow())?.base;
    if account
        .key
        .ne(&Operator::find_program_address(program_id, &base).0)
    {
        msg!("Operator account is not at the correct PDA");
        return Err(ProgramError::InvalidAccountData);
    }
    Ok(())
}

/// Loads the account as an [`OperatorVaultTicket`] account, returning an error if it is not.
///
/// # Arguments
/// * `program_id` - The program ID
/// * `operator_vault_ticket` - The account to load the operator vault ticket from
/// * `operator` - The operator account
/// * `vault` - The vault account
/// * `expect_writable` - Whether the account should be writable
///
/// # Returns
/// * `Result<(), ProgramError>` - The result of the operation
pub fn load_operator_vault_ticket(
    program_id: &Pubkey,
    operator_vault_ticket: &AccountInfo,
    operator: &AccountInfo,
    vault: &AccountInfo,
    expect_writable: bool,
) -> Result<(), ProgramError> {
    if operator_vault_ticket.owner.ne(program_id) {
        msg!("Operator vault ticket account has an invalid owner");
        return Err(ProgramError::InvalidAccountOwner);
    }
    if operator_vault_ticket.data_is_empty() {
        msg!("Operator vault ticket account data is empty");
        return Err(ProgramError::InvalidAccountData);
    }
    if expect_writable && !operator_vault_ticket.is_writable {
        msg!("Operator vault ticket account is not writable");
        return Err(ProgramError::InvalidAccountData);
    }
    if operator_vault_ticket.data.borrow()[0].ne(&OperatorVaultTicket::DISCRIMINATOR) {
        msg!("Operator vault ticket account discriminator is invalid");
        return Err(ProgramError::InvalidAccountData);
    }
    let expected_pubkey =
        OperatorVaultTicket::find_program_address(program_id, operator.key, vault.key).0;
    if operator_vault_ticket.key.ne(&expected_pubkey) {
        msg!("Operator vault ticket account is not at the correct PDA");
        return Err(ProgramError::InvalidAccountData);
    }
    Ok(())
}

/// Loads the account as an [`NcnOperatorTicket`] account, returning an error if it is not.
///
/// # Arguments
/// * `program_id` - The program ID
/// * `ncn_operator_ticket` - The account to load the NCN operator ticket from
/// * `ncn` - The NCN account
/// * `operator` - The operator account
/// * `expect_writable` - Whether the account should be writable
///
/// # Returns
/// * `Result<(), ProgramError>` - The result of the operation
pub fn load_ncn_operator_ticket(
    program_id: &Pubkey,
    ncn_operator_ticket: &AccountInfo,
    ncn: &AccountInfo,
    operator: &AccountInfo,
    expect_writable: bool,
) -> Result<(), ProgramError> {
    if ncn_operator_ticket.owner.ne(program_id) {
        msg!("NCN operator ticket account has an invalid owner");
        return Err(ProgramError::InvalidAccountOwner);
    }
    if ncn_operator_ticket.data_is_empty() {
        msg!("NCN operator ticket account data is empty");
        return Err(ProgramError::InvalidAccountData);
    }
    if expect_writable && !ncn_operator_ticket.is_writable {
        msg!("NCN operator ticket account is not writable");
        return Err(ProgramError::InvalidAccountData);
    }
    if ncn_operator_ticket.data.borrow()[0].ne(&NcnOperatorTicket::DISCRIMINATOR) {
        msg!("NCN operator ticket account discriminator is invalid");
        return Err(ProgramError::InvalidAccountData);
    }
    let expected_pubkey =
        NcnOperatorTicket::find_program_address(program_id, ncn.key, operator.key).0;
    if ncn_operator_ticket.key.ne(&expected_pubkey) {
        msg!("NCN operator ticket account is not at the correct PDA");
        return Err(ProgramError::InvalidAccountData);
    }
    Ok(())
}

/// Loads the account as an [`NcnVaultTicket`] account, returning an error if it is not.
///
/// # Arguments
/// * `program_id` - The program ID
/// * `ncn_vault_ticket` - The account to load the NCN vault ticket from
/// * `ncn` - The NCN account
/// * `vault` - The vault account
/// * `expect_writable` - Whether the account should be writable
///
/// # Returns
/// * `Result<(), ProgramError>` - The result of the operation
pub fn load_ncn_vault_ticket(
    program_id: &Pubkey,
    ncn_vault_ticket: &AccountInfo,
    ncn: &AccountInfo,
    vault: &AccountInfo,
    expect_writable: bool,
) -> Result<(), ProgramError> {
    if ncn_vault_ticket.owner.ne(program_id) {
        msg!("NCN vault ticket account has an invalid owner");
        return Err(ProgramError::InvalidAccountOwner);
    }
    if ncn_vault_ticket.data_is_empty() {
        msg!("NCN vault ticket account data is empty");
        return Err(ProgramError::InvalidAccountData);
    }
    if expect_writable && !ncn_vault_ticket.is_writable {
        msg!("NCN vault ticket account is not writable");
        return Err(ProgramError::InvalidAccountData);
    }
    if ncn_vault_ticket.data.borrow()[0].ne(&NcnVaultTicket::DISCRIMINATOR) {
        msg!("NCN vault ticket account discriminator is invalid");
        return Err(ProgramError::InvalidAccountData);
    }
    let expected_pubkey = NcnVaultTicket::find_program_address(program_id, ncn.key, vault.key).0;
    if ncn_vault_ticket.key.ne(&expected_pubkey) {
        msg!("NCN vault ticket account is not at the correct PDA");
        return Err(ProgramError::InvalidAccountData);
    }
    Ok(())
}

/// Loads the account as an [`OperatorNcnTicket`] account, returning an error if it is not.
///
/// # Arguments
/// * `program_id` - The program ID
/// * `operator_ncn_ticket` - The account to load the operator NCN ticket from
/// * `operator` - The operator account
/// * `ncn` - The NCN account
/// * `expect_writable` - Whether the account should be writable
///
/// # Returns
/// * `Result<(), ProgramError>` - The result of the operation
pub fn load_operator_ncn_ticket(
    program_id: &Pubkey,
    operator_ncn_ticket: &AccountInfo,
    operator: &AccountInfo,
    ncn: &AccountInfo,
    expect_writable: bool,
) -> Result<(), ProgramError> {
    if operator_ncn_ticket.owner.ne(program_id) {
        msg!("Operator NCN ticket account has an invalid owner");
        return Err(ProgramError::InvalidAccountOwner);
    }
    if operator_ncn_ticket.data_is_empty() {
        msg!("Operator NCN ticket account data is empty");
        return Err(ProgramError::InvalidAccountData);
    }
    if expect_writable && !operator_ncn_ticket.is_writable {
        msg!("Operator NCN ticket account is not writable");
        return Err(ProgramError::InvalidAccountData);
    }
    if operator_ncn_ticket.data.borrow()[0].ne(&OperatorNcnTicket::DISCRIMINATOR) {
        msg!("Operator NCN ticket account discriminator is invalid");
        return Err(ProgramError::InvalidAccountData);
    }
    let expected_pubkey =
        OperatorNcnTicket::find_program_address(program_id, operator.key, ncn.key).0;
    if operator_ncn_ticket.key.ne(&expected_pubkey) {
        msg!("Operator NCN ticket account is not at the correct PDA");
        return Err(ProgramError::InvalidAccountData);
    }
    Ok(())
}

/// Loads the account as an [`NcnVaultSlasherTicket`] account, returning an error if it is not.
///
/// # Arguments
/// * `program_id` - The program ID
/// * `ncn_vault_slasher_ticket` - The account to load the NCN vault slasher ticket from
/// * `ncn` - The NCN account
/// * `vault` - The vault account
/// * `slasher` - The slasher account
/// * `expect_writable` - Whether the account should be writable
///
/// # Returns
/// * `Result<(), ProgramError>` - The result of the operation
pub fn load_ncn_vault_slasher_ticket(
    program_id: &Pubkey,
    ncn_vault_slasher_ticket: &AccountInfo,
    ncn: &AccountInfo,
    vault: &AccountInfo,
    slasher: &AccountInfo,
    expect_writable: bool,
) -> Result<(), ProgramError> {
    if ncn_vault_slasher_ticket.owner.ne(program_id) {
        msg!("NCN vault slasher ticket account has an invalid owner");
        return Err(ProgramError::InvalidAccountOwner);
    }
    if ncn_vault_slasher_ticket.data_is_empty() {
        msg!("NCN vault slasher ticket account data is empty");
        return Err(ProgramError::InvalidAccountData);
    }
    if expect_writable && !ncn_vault_slasher_ticket.is_writable {
        msg!("NCN vault slasher ticket account is not writable");
        return Err(ProgramError::InvalidAccountData);
    }
    if ncn_vault_slasher_ticket.data.borrow()[0].ne(&NcnVaultSlasherTicket::DISCRIMINATOR) {
        msg!("NCN vault slasher ticket account discriminator is invalid");
        return Err(ProgramError::InvalidAccountData);
    }
    let expected_pubkey =
        NcnVaultSlasherTicket::find_program_address(program_id, ncn.key, vault.key, slasher.key).0;
    if ncn_vault_slasher_ticket.key.ne(&expected_pubkey) {
        msg!("NCN vault slasher ticket account is not at the correct PDA");
        return Err(ProgramError::InvalidAccountData);
    }
    Ok(())
}
