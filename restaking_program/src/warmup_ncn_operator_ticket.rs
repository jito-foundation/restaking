use jito_account_traits::AccountDeserialize;
use jito_jsm_core::loader::load_signer;
use jito_restaking_core::{
    config::Config,
    loader::{load_config, load_ncn, load_ncn_operator_ticket, load_operator},
    ncn::Ncn,
    ncn_operator_ticket::NcnOperatorTicket,
};
use solana_program::{
    account_info::AccountInfo, clock::Clock, entrypoint::ProgramResult, msg,
    program_error::ProgramError, pubkey::Pubkey, sysvar::Sysvar,
};

/// [`crate::RestakingInstruction::WarmupNcnOperatorTicket`]
pub fn process_warmup_ncn_operator_ticket(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let [config, ncn, operator, ncn_operator_ticket, ncn_operator_admin] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    load_config(program_id, config, false)?;
    load_ncn(program_id, ncn, false)?;
    load_operator(program_id, operator, false)?;
    load_ncn_operator_ticket(program_id, ncn_operator_ticket, ncn, operator, true)?;
    load_signer(ncn_operator_admin, false)?;

    let ncn_data = ncn.data.borrow();
    let ncn = Ncn::try_from_slice(&ncn_data)?;
    if ncn.operator_admin.ne(ncn_operator_admin.key) {
        msg!("Invalid operator admin for NCN");
        return Err(ProgramError::InvalidAccountData);
    }

    let config_data = config.data.borrow_mut();
    let config = Config::try_from_slice(&config_data)?;

    let mut ncn_operator_ticket_data = ncn_operator_ticket.data.borrow_mut();
    let ncn_operator_ticket = NcnOperatorTicket::try_from_slice_mut(&mut ncn_operator_ticket_data)?;

    if !ncn_operator_ticket
        .state
        .activate(Clock::get()?.slot, config.epoch_length)
    {
        msg!("Operator is not ready to be activated");
        return Err(ProgramError::InvalidAccountData);
    }

    Ok(())
}
