use std::mem::size_of;

use jito_account_traits::{AccountDeserialize, Discriminator};
use jito_jsm_core::{
    create_account,
    loader::{load_signer, load_system_account, load_system_program},
};
use jito_restaking_core::operator::Operator;
use jito_restaking_core::{
    config::Config,
    loader::{load_config, load_ncn, load_operator},
    ncn::Ncn,
    ncn_operator_state::NcnOperatorState,
};
use jito_restaking_sdk::error::RestakingError;
use solana_program::{
    account_info::AccountInfo, clock::Clock, entrypoint::ProgramResult, msg,
    program_error::ProgramError, pubkey::Pubkey, rent::Rent, sysvar::Sysvar,
};

/// After an operator opts-in to an NCN, the NCN operator admin can add the operator to the NCN.
/// The operator must have opted-in to the NCN before the NCN opts-in to the operator.
///
/// [`crate::RestakingInstruction::InitializeNcnOperatorState`]
pub fn process_initialize_ncn_operator_state(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let [config, ncn_info, operator, ncn_operator_state, ncn_operator_admin, payer, system_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    load_config(program_id, config, false)?;
    load_ncn(program_id, ncn_info, true)?;
    load_operator(program_id, operator, true)?;
    load_system_account(ncn_operator_state, true)?;
    load_signer(ncn_operator_admin, false)?;
    load_signer(payer, true)?;
    load_system_program(system_program)?;

    // The NcnOperatorTicket shall be at the canonical PDA
    let (ncn_operator_state_pubkey, ncn_operator_state_bump, mut ncn_operator_state_seeds) =
        NcnOperatorState::find_program_address(program_id, ncn_info.key, operator.key);
    ncn_operator_state_seeds.push(vec![ncn_operator_state_bump]);
    if ncn_operator_state_pubkey.ne(ncn_operator_state.key) {
        msg!("NcnOperatorState is not at the correct PDA");
        return Err(ProgramError::InvalidAccountData);
    }

    // The NCN operator admin must be the signer for adding an operator to the NCN
    let mut ncn_data = ncn_info.data.borrow_mut();
    let ncn = Ncn::try_from_slice_mut(&mut ncn_data)?;
    if ncn.operator_admin.ne(ncn_operator_admin.key) {
        msg!("Invalid operator admin for NCN");
        return Err(RestakingError::NcnOperatorAdminInvalid.into());
    }

    msg!("Initializing NcnOperatorState at address {}", operator.key);
    create_account(
        payer,
        ncn_operator_state,
        system_program,
        program_id,
        &Rent::get()?,
        8_u64
            .checked_add(size_of::<NcnOperatorState>() as u64)
            .unwrap(),
        &ncn_operator_state_seeds,
    )?;

    let mut ncn_operator_state_data = ncn_operator_state.try_borrow_mut_data()?;
    ncn_operator_state_data[0] = NcnOperatorState::DISCRIMINATOR;
    let ncn_operator_state = NcnOperatorState::try_from_slice_mut(&mut ncn_operator_state_data)?;
    *ncn_operator_state = NcnOperatorState::new(
        *ncn_info.key,
        *operator.key,
        ncn.operator_count,
        ncn_operator_state_bump,
    );

    let mut operator_data = operator.data.borrow_mut();
    let operator = Operator::try_from_slice_mut(&mut operator_data)?;

    ncn.operator_count = ncn
        .operator_count
        .checked_add(1)
        .ok_or(ProgramError::InvalidAccountData)?;
    operator.ncn_count = operator
        .ncn_count
        .checked_add(1)
        .ok_or(ProgramError::InvalidAccountData)?;

    Ok(())
}
