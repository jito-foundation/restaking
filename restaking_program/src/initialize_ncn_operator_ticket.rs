use std::mem::size_of;

use jito_account_traits::{AccountDeserialize, Discriminator};
use jito_jsm_core::{
    create_account,
    loader::{load_signer, load_system_account, load_system_program},
};
use jito_restaking_core::{
    config::Config,
    loader::{load_config, load_ncn, load_operator, load_operator_ncn_ticket},
    ncn::Ncn,
    ncn_operator_ticket::NcnOperatorTicket,
    operator_ncn_ticket::OperatorNcnTicket,
};
use jito_restaking_sdk::error::RestakingError;
use solana_program::{
    account_info::AccountInfo, clock::Clock, entrypoint::ProgramResult, msg,
    program_error::ProgramError, pubkey::Pubkey, rent::Rent, sysvar::Sysvar,
};

/// After an operator opts-in to an NCN, the NCN operator admin can add the operator to the NCN.
/// The operator must have opted-in to the NCN before the NCN opts-in to the operator.
///
/// [`crate::RestakingInstruction::InitializeNcnOperatorTicket`]
pub fn process_initialize_ncn_operator_ticket(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let [config, ncn_info, operator, ncn_operator_ticket, operator_ncn_ticket, ncn_operator_admin, payer, system_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    load_config(program_id, config, false)?;
    load_ncn(program_id, ncn_info, true)?;
    load_operator(program_id, operator, false)?;
    load_system_account(ncn_operator_ticket, false)?;
    load_operator_ncn_ticket(program_id, operator_ncn_ticket, operator, ncn_info, false)?;
    load_signer(ncn_operator_admin, false)?;
    load_signer(payer, true)?;
    load_system_program(system_program)?;

    // The NcnOperatorTicket shall be at the canonical PDA
    let (ncn_operator_ticket_pubkey, ncn_operator_ticket_bump, mut ncn_operator_ticket_seeds) =
        NcnOperatorTicket::find_program_address(program_id, ncn_info.key, operator.key);
    ncn_operator_ticket_seeds.push(vec![ncn_operator_ticket_bump]);
    if ncn_operator_ticket_pubkey.ne(ncn_operator_ticket.key) {
        msg!("NCN operator ticket is not at the correct PDA");
        return Err(ProgramError::InvalidAccountData);
    }

    let slot = Clock::get()?.slot;

    let mut config_data = config.data.borrow_mut();
    let config = Config::try_from_slice_mut(&mut config_data)?;

    // The NCN operator admin must be the signer for adding an operator to the NCN
    let mut ncn_data = ncn_info.data.borrow_mut();
    let ncn = Ncn::try_from_slice_mut(&mut ncn_data)?;
    if ncn.operator_admin.ne(ncn_operator_admin.key) {
        msg!("Invalid operator admin for NCN");
        return Err(RestakingError::NcnOperatorAdminInvalid.into());
    }

    // The operator must have opted-in to the NCN and it must be active
    let operator_ncn_ticket_data = operator_ncn_ticket.data.borrow();
    let operator_ncn_ticket = OperatorNcnTicket::try_from_slice(&operator_ncn_ticket_data)?;
    if !operator_ncn_ticket
        .state
        .is_active(slot, config.epoch_length)
    {
        msg!("Operator NCN ticket is not active or in cooldown");
        return Err(RestakingError::OperatorNcnTicketNotActive.into());
    }

    msg!("Initializing NcnOperatorTicket at address {}", operator.key);
    create_account(
        payer,
        ncn_operator_ticket,
        system_program,
        program_id,
        &Rent::get()?,
        8_u64
            .checked_add(size_of::<NcnOperatorTicket>() as u64)
            .unwrap(),
        &ncn_operator_ticket_seeds,
    )?;

    let mut ncn_operator_ticket_data = ncn_operator_ticket.try_borrow_mut_data()?;
    ncn_operator_ticket_data[0] = NcnOperatorTicket::DISCRIMINATOR;
    let ncn_operator_ticket = NcnOperatorTicket::try_from_slice_mut(&mut ncn_operator_ticket_data)?;
    *ncn_operator_ticket = NcnOperatorTicket::new(
        *ncn_info.key,
        *operator.key,
        ncn.operator_count,
        slot,
        ncn_operator_ticket_bump,
    );

    ncn.operator_count = ncn
        .operator_count
        .checked_add(1)
        .ok_or(ProgramError::InvalidAccountData)?;

    Ok(())
}
