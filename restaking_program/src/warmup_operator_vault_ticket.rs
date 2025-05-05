use jito_bytemuck::AccountDeserialize;
use jito_jsm_core::loader::load_signer;
use jito_restaking_core::{
    config::Config, operator::Operator, operator_vault_ticket::OperatorVaultTicket,
};
use jito_restaking_sdk::error::RestakingError;
use jito_vault_core::vault::Vault;
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
    Config::load(program_id, config, false)?;
    Operator::load(program_id, operator, false)?;
    let config_data = config.data.borrow();
    let config = Config::try_from_slice_unchecked(&config_data)?;
    Vault::load(&config.vault_program, vault, false)?;
    OperatorVaultTicket::load(program_id, operator_vault_ticket, operator, vault, true)?;
    load_signer(operator_vault_admin, false)?;

    // The operator vault admin shall be the signer of the transaction
    let operator_data = operator.data.borrow();
    let operator = Operator::try_from_slice_unchecked(&operator_data)?;
    if operator.vault_admin.ne(operator_vault_admin.key) {
        msg!("Invalid vault admin for operator");
        return Err(RestakingError::OperatorVaultAdminInvalid.into());
    }

    // The OperatorVaultTicket shall be inactive before it can warmed up
    let mut operator_vault_ticket_data = operator_vault_ticket.data.borrow_mut();
    let operator_vault_ticket =
        OperatorVaultTicket::try_from_slice_unchecked_mut(&mut operator_vault_ticket_data)?;
    if !operator_vault_ticket
        .state
        .activate(Clock::get()?.slot, config.epoch_length())?
    {
        msg!("Operator is not ready to be activated");
        return Err(RestakingError::OperatorVaultTicketFailedWarmup.into());
    }

    msg!(
        "WARMUP OPERATOR_VAULT_TICKET: Operator {} activating Vault {}",
        operator_vault_ticket.operator,
        operator_vault_ticket.vault,
    );

    Ok(())
}
