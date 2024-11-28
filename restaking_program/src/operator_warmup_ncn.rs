use jito_bytemuck::AccountDeserialize;
use jito_jsm_core::loader::load_signer;
use jito_restaking_core::{
    config::Config, ncn::Ncn, ncn_operator_state::NcnOperatorState, operator::Operator,
};
use jito_restaking_sdk::error::RestakingError;
use solana_program::{
    account_info::AccountInfo, clock::Clock, entrypoint::ProgramResult, msg,
    program_error::ProgramError, pubkey::Pubkey, sysvar::Sysvar,
};

/// [`crate::RestakingInstruction::OperatorWarmupNcn`]
pub fn process_operator_warmup_ncn(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let [config, ncn, operator, ncn_operator_state, operator_ncn_admin] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    Config::load(program_id, config, false)?;
    Operator::load(program_id, operator, false)?;
    Ncn::load(program_id, ncn, false)?;
    NcnOperatorState::load(program_id, ncn_operator_state, ncn, operator, true)?;
    load_signer(operator_ncn_admin, false)?;

    // The operator NCN admin shall be the signer of the transaction
    let operator_data = operator.data.borrow();
    let operator = Operator::try_from_slice_unchecked(&operator_data)?;
    if operator.ncn_admin.ne(operator_ncn_admin.key) {
        msg!("Invalid NCN admin for operator");
        return Err(RestakingError::OperatorNcnAdminInvalid.into());
    }

    // The OperatorNcnTicket shall be inactive before it can warmed up
    let config_data = config.data.borrow();
    let config = Config::try_from_slice_unchecked(&config_data)?;
    let mut ncn_operator_state_data = ncn_operator_state.data.borrow_mut();
    let ncn_operator_state =
        NcnOperatorState::try_from_slice_unchecked_mut(&mut ncn_operator_state_data)?;
    if !ncn_operator_state
        .operator_opt_in_state
        .activate(Clock::get()?.slot, config.epoch_length())?
    {
        msg!("Operator is not ready to warm up NCN");
        return Err(RestakingError::OperatorWarmupNcnFailed.into());
    }

    Ok(())
}
