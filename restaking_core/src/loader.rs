//! Loader functions for the restaking program
//! Thank you to HardhatChad for the inspiration with Ore account loading
use jito_account_traits::{AccountDeserialize, Discriminator};
use solana_program::{account_info::AccountInfo, msg, program_error::ProgramError, pubkey::Pubkey};

use crate::ncn_operator_ticket::NcnOperatorTicket;
use crate::{config::Config, ncn::Ncn, operator::Operator};

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
    if *account.key != Config::find_program_address(program_id).0 {
        msg!("Config account is not at the correct PDA");
        return Err(ProgramError::InvalidAccountData);
    }

    Ok(())
}

pub fn load_ncn(
    program_id: &Pubkey,
    account: &AccountInfo,
    expect_writable: bool,
) -> Result<(), ProgramError> {
    if account.owner.ne(program_id) {
        msg!("NCN account has an invalid owner");
        return Err(ProgramError::IncorrectProgramId);
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
    // TODO (LB): try not to double deserialize and return NCN?
    let ncn = Ncn::try_from_slice(&account.data.borrow())?;
    if *account.key != Ncn::find_program_address(program_id, &ncn.base).0 {
        msg!("NCN account is not at the correct PDA");
        return Err(ProgramError::InvalidAccountData);
    }
    Ok(())
}

pub fn load_operator(
    program_id: &Pubkey,
    account: &AccountInfo,
    expect_writable: bool,
) -> Result<(), ProgramError> {
    if account.owner.ne(program_id) {
        msg!("Operator account has an invalid owner");
        return Err(ProgramError::IncorrectProgramId);
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
    let operator = Operator::try_from_slice(&account.data.borrow())?;
    if *account.key != Operator::find_program_address(program_id, &operator.base).0 {
        msg!("Operator account is not at the correct PDA");
        return Err(ProgramError::InvalidAccountData);
    }
    Ok(())
}

pub fn load_ncn_operator_ticket(
    program_id: &Pubkey,
    ncn_operator_ticket: &AccountInfo,
    ncn: &AccountInfo,
    operator: &AccountInfo,
    expect_writable: bool,
) -> Result<(), ProgramError> {
    if ncn_operator_ticket.owner.ne(program_id) {
        msg!("NCN operator ticket account has an invalid owner");
        return Err(ProgramError::IncorrectProgramId);
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
    if *ncn_operator_ticket.key != expected_pubkey {
        msg!("NCN operator ticket account is not at the correct PDA");
        return Err(ProgramError::InvalidAccountData);
    }
    Ok(())
}

pub fn load_operator_ncn_ticket(
    program_id: &Pubkey,
    operator_ncn_ticket: &AccountInfo,
    operator: &AccountInfo,
    ncn: &AccountInfo,
    expect_writable: bool,
) -> Result<(), ProgramError> {
    Ok(())
}
