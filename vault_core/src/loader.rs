use crate::config::Config;
use crate::vault::Vault;
use crate::vault_ncn_slasher_ticket::VaultNcnSlasherTicket;
use crate::vault_ncn_ticket::VaultNcnTicket;
use crate::vault_operator_ticket::VaultOperatorTicket;
use jito_account_traits::{AccountDeserialize, Discriminator};
use solana_program::account_info::AccountInfo;
use solana_program::msg;
use solana_program::program_error::ProgramError;
use solana_program::pubkey::Pubkey;

pub fn load_config(
    program_id: &Pubkey,
    account: &AccountInfo,
    expect_writable: bool,
) -> Result<(), ProgramError> {
    if account.owner.ne(program_id) {
        msg!("Config account has an invalid owner");
        return Err(ProgramError::IncorrectProgramId);
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

pub fn load_vault(
    program_id: &Pubkey,
    account: &AccountInfo,
    expect_writable: bool,
) -> Result<(), ProgramError> {
    if account.owner.ne(program_id) {
        msg!("Vault account has an invalid owner");
        return Err(ProgramError::IncorrectProgramId);
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

pub fn load_vault_ncn_ticket(
    program_id: &Pubkey,
    vault_ncn_ticket: &AccountInfo,
    vault: &AccountInfo,
    ncn: &AccountInfo,
    expect_writable: bool,
) -> Result<(), ProgramError> {
    if vault_ncn_ticket.owner.ne(program_id) {
        msg!("Vault NCN ticket account has an invalid owner");
        return Err(ProgramError::IncorrectProgramId);
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

pub fn load_vault_operator_ticket(
    program_id: &Pubkey,
    vault_operator_ticket: &AccountInfo,
    vault: &AccountInfo,
    operator: &AccountInfo,
    expect_writable: bool,
) -> Result<(), ProgramError> {
    if vault_operator_ticket.owner.ne(program_id) {
        msg!("Vault operator ticket account has an invalid owner");
        return Err(ProgramError::IncorrectProgramId);
    }
    if vault_operator_ticket.data_is_empty() {
        msg!("Vault operator ticket account data is empty");
        return Err(ProgramError::InvalidAccountData);
    }
    if expect_writable && !vault_operator_ticket.is_writable {
        msg!("Vault operator ticket account is not writable");
        return Err(ProgramError::InvalidAccountData);
    }
    if vault_operator_ticket.data.borrow()[0].ne(&VaultOperatorTicket::DISCRIMINATOR) {
        msg!("Vault operator ticket account discriminator is invalid");
        return Err(ProgramError::InvalidAccountData);
    }
    let expected_pubkey =
        VaultOperatorTicket::find_program_address(program_id, vault.key, operator.key).0;
    if vault_operator_ticket.key.ne(&expected_pubkey) {
        msg!("Vault operator ticket account is not at the correct PDA");
        return Err(ProgramError::InvalidAccountData);
    }
    Ok(())
}

pub fn load_vault_ncn_slasher_operator_ticket(
    program_id: &Pubkey,
    vault_ncn_slasher_operator_ticket: &AccountInfo,
    vault: &AccountInfo,
    ncn: &AccountInfo,
    slasher: &AccountInfo,
    operator: &AccountInfo,
    expect_writable: bool,
) -> Result<(), ProgramError> {
    Ok(())
}

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
        return Err(ProgramError::IncorrectProgramId);
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

pub fn load_vault_staker_withdrawal_ticket(
    program_id: &Pubkey,
    vault_staker_withdrawal_ticket: &AccountInfo,
    vault: &AccountInfo,
    staker: &AccountInfo,
    expect_writable: bool,
) -> Result<(), ProgramError> {
    Ok(())
}
