use jito_account_traits::AccountDeserialize;
use jito_jsm_core::loader::load_signer;
use jito_restaking_core::config::Config;
use jito_restaking_core::loader::{load_config, load_ncn, load_ncn_operator_ticket, load_operator};
use jito_restaking_core::ncn::Ncn;
use jito_restaking_core::ncn_operator_ticket::NcnOperatorTicket;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::Clock,
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvar::Sysvar,
};

/// The NCN admin can remove a node operator from the NCN.
/// This method is permissioned to the NCN admin.
/// [`crate::RestakingInstruction::NcnRemoveOperator`]
pub fn process_ncn_cooldown_operator(
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

    let config = Config::try_from_slice(&config.data.borrow())?;

    let ncn = Ncn::try_from_slice(&ncn.data.borrow())?;
    if !ncn.operator_admin.eq(&ncn_operator_admin.key) {
        msg!("Invalid operator admin for NCN");
        return Err(ProgramError::InvalidAccountData);
    }

    let ncn_operator_ticket =
        NcnOperatorTicket::try_from_slice_mut(&mut ncn_operator_ticket.data.borrow_mut())?;

    if !ncn_operator_ticket
        .state
        .deactivate(Clock::get()?.slot, config.epoch_length)
    {
        msg!("Operator is not ready to be deactivated");
        return Err(ProgramError::InvalidAccountData);
    }

    Ok(())
}
