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

/// The NCN admin can remove a node operator from the NCN.
/// This method is permissioned to the NCN admin.
/// [`crate::RestakingInstruction::NcnCooldownOperator`]
pub fn process_ncn_cooldown_operator(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let [config, ncn, operator, ncn_operator_state, ncn_operator_admin] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    Config::load(program_id, config, false)?;
    Ncn::load(program_id, ncn, false)?;
    Operator::load(program_id, operator, false)?;
    NcnOperatorState::load(program_id, ncn_operator_state, ncn, operator, true)?;
    load_signer(ncn_operator_admin, false)?;

    // The NCN operator admin shall be the signer of the transaction
    let ncn_data = ncn.data.borrow();
    let ncn = Ncn::try_from_slice_unchecked(&ncn_data)?;
    if !ncn.operator_admin.eq(ncn_operator_admin.key) {
        msg!("Invalid operator admin for NCN");
        return Err(RestakingError::NcnOperatorAdminInvalid.into());
    }

    // The NcnOperatorTicket shall be active before it can be cooled down
    let config_data = config.data.borrow();
    let config = Config::try_from_slice_unchecked(&config_data)?;
    let mut ncn_operator_state_data = ncn_operator_state.data.borrow_mut();
    let ncn_operator_ticket =
        NcnOperatorState::try_from_slice_unchecked_mut(&mut ncn_operator_state_data)?;
    if !ncn_operator_ticket
        .ncn_opt_in_state
        .deactivate(Clock::get()?.slot, config.epoch_length())?
    {
        msg!("NCN is not ready to deactivate operator");
        return Err(RestakingError::NcnCooldownOperatorFailed.into());
    }

    Ok(())
}
