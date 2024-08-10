use std::mem::size_of;

use jito_account_traits::{AccountDeserialize, Discriminator};
use jito_jsm_core::{
    create_account,
    loader::{load_signer, load_system_account, load_system_program},
    slot_toggled_field::SlotToggle,
};
use jito_restaking_core::{
    loader::{load_ncn, load_operator},
    operator::Operator,
    operator_ncn_ticket::OperatorNcnTicket,
};
use jito_vault_core::loader::load_config;
use solana_program::{
    account_info::AccountInfo, clock::Clock, entrypoint::ProgramResult, msg,
    program_error::ProgramError, pubkey::Pubkey, rent::Rent, sysvar::Sysvar,
};

/// The node operator admin can add support for running an NCN.
/// This method is permissioned to the node operator admin.
///
/// [`crate::RestakingInstruction::OperatorAddNcn`]
pub fn process_operator_add_ncn(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let [config, operator_info, ncn, operator_ncn_ticket, operator_ncn_admin, payer, system_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    load_config(program_id, config, false)?;
    load_operator(program_id, operator_info, true)?;
    load_ncn(program_id, ncn, false)?;
    load_system_account(operator_ncn_ticket, true)?;
    load_signer(operator_ncn_admin, false)?;
    load_signer(payer, true)?;
    load_system_program(system_program)?;

    let (operator_ncn_ticket_pubkey, operator_ncn_ticket_bump, mut operator_ncn_ticket_seeds) =
        OperatorNcnTicket::find_program_address(program_id, operator_info.key, ncn.key);
    operator_ncn_ticket_seeds.push(vec![operator_ncn_ticket_bump]);
    if operator_ncn_ticket.key.ne(&operator_ncn_ticket_pubkey) {
        msg!("Operator NCN ticket account is not at the correct PDA");
        return Err(ProgramError::InvalidArgument);
    }

    let mut operator_data = operator_info.data.borrow_mut();
    let operator = Operator::try_from_slice_mut(&mut operator_data)?;
    if operator.ncn_admin.ne(operator_ncn_admin.key) {
        msg!("Invalid operator NCN admin");
        return Err(ProgramError::InvalidAccountData);
    }

    msg!(
        "Initializing OperatorNcnTicket at address {}",
        operator_ncn_ticket.key
    );
    create_account(
        payer,
        operator_ncn_ticket,
        system_program,
        program_id,
        &Rent::get()?,
        8_u64
            .checked_add(size_of::<OperatorNcnTicket>() as u64)
            .unwrap(),
        &operator_ncn_ticket_seeds,
    )?;
    let mut operator_ncn_ticket_data = operator_ncn_ticket.try_borrow_mut_data()?;
    operator_ncn_ticket_data[0] = OperatorNcnTicket::DISCRIMINATOR;
    let operator_ncn_ticket = OperatorNcnTicket::try_from_slice_mut(&mut operator_ncn_ticket_data)?;
    operator_ncn_ticket.operator = *operator_info.key;
    operator_ncn_ticket.ncn = *ncn.key;
    operator_ncn_ticket.index = operator.ncn_count;
    operator_ncn_ticket.state = SlotToggle::new(Clock::get()?.slot);
    operator_ncn_ticket.bump = operator_ncn_ticket_bump;

    operator.ncn_count = operator
        .ncn_count
        .checked_add(1)
        .ok_or(ProgramError::InvalidAccountData)?;

    Ok(())
}
