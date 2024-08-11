use jito_account_traits::AccountDeserialize;
use jito_jsm_core::loader::load_signer;
use jito_restaking_core::{
    config::Config,
    loader::{load_config, load_ncn, load_operator, load_operator_ncn_ticket},
    operator::Operator,
    operator_ncn_ticket::OperatorNcnTicket,
};
use solana_program::{
    account_info::AccountInfo, clock::Clock, entrypoint::ProgramResult, msg,
    program_error::ProgramError, pubkey::Pubkey, sysvar::Sysvar,
};

/// [`crate::RestakingInstruction::WarmupOperatorNcnTicket`]
pub fn process_warmup_operator_ncn_ticket(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let [config, operator, ncn, operator_ncn_ticket, operator_ncn_admin] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    load_config(program_id, config, false)?;
    load_operator(program_id, operator, false)?;
    load_ncn(program_id, ncn, false)?;
    load_operator_ncn_ticket(program_id, operator_ncn_ticket, operator, ncn, true)?;
    load_signer(operator_ncn_admin, false)?;

    let operator_data = operator.data.borrow();
    let operator = Operator::try_from_slice(&operator_data)?;
    if operator.ncn_admin.ne(operator_ncn_admin.key) {
        msg!("Invalid NCN admin for operator");
        return Err(ProgramError::InvalidAccountData);
    }

    let config_data = config.data.borrow();
    let config = Config::try_from_slice(&config_data)?;

    let mut operator_ncn_ticket_data = operator_ncn_ticket.data.borrow_mut();
    let operator_ncn_ticket = OperatorNcnTicket::try_from_slice_mut(&mut operator_ncn_ticket_data)?;
    if !operator_ncn_ticket
        .state
        .activate(Clock::get()?.slot, config.epoch_length)
    {
        msg!("Operator is not ready to be activated");
        return Err(ProgramError::InvalidAccountData);
    }

    Ok(())
}
