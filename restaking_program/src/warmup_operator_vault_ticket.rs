use jito_account_traits::AccountDeserialize;
use jito_jsm_core::loader::load_signer;
use jito_restaking_core::{
    config::Config,
    loader::{load_config, load_operator, load_operator_vault_ticket},
    operator::Operator,
    operator_vault_ticket::OperatorVaultTicket,
};
use jito_vault_core::loader::load_vault;
use solana_program::{
    account_info::AccountInfo, clock::Clock, entrypoint::ProgramResult, msg,
    program_error::ProgramError, pubkey::Pubkey, sysvar::Sysvar,
};

/// [`crate::RestakingInstruction::WarmupOperatorVaultTicket`]
pub fn process_warmup_operator_vault_ticket(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let [config, operator, vault, operator_vault_ticket, operator_vault_admin] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    load_config(program_id, config, false)?;
    load_operator(program_id, operator, false)?;
    let config_data = config.data.borrow();
    let config = Config::try_from_slice(&config_data)?;
    load_vault(&config.vault_program, vault, false)?;
    load_operator_vault_ticket(program_id, operator_vault_ticket, operator, vault, true)?;
    load_signer(operator_vault_admin, false)?;

    // The operator vault admin shall be the signer of the transaction
    let operator_data = operator.data.borrow();
    let operator = Operator::try_from_slice(&operator_data)?;
    if operator.vault_admin.ne(operator_vault_admin.key) {
        msg!("Invalid vault admin for operator");
        return Err(ProgramError::InvalidAccountData);
    }

    // The OperatorVaultTicket shall be inactive before it can warmed up
    let mut operator_vault_ticket_data = operator_vault_ticket.data.borrow_mut();
    let operator_vault_ticket =
        OperatorVaultTicket::try_from_slice_mut(&mut operator_vault_ticket_data)?;
    if !operator_vault_ticket
        .state
        .activate(Clock::get()?.slot, config.epoch_length)
    {
        msg!("Operator is not ready to be activated");
        return Err(ProgramError::InvalidAccountData);
    }

    Ok(())
}