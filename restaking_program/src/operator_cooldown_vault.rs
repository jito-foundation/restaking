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

/// [`crate::RestakingInstruction::OperatorCooldownVault`]
pub fn process_operator_cooldown_vault(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let [config, operator, vault, operator_vault_ticket, operator_vault_admin] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    load_config(program_id, config, false)?;
    load_operator(program_id, operator, false)?;
    let mut config_data = config.data.borrow_mut();
    let config = Config::try_from_slice_mut(&mut config_data)?;
    load_vault(&config.vault_program, vault, false)?;
    load_operator_vault_ticket(program_id, operator_vault_ticket, operator, vault, true)?;
    load_signer(operator_vault_admin, false)?;

    let operator_data = operator.data.borrow();
    let operator = Operator::try_from_slice(&operator_data)?;
    if operator.vault_admin.ne(operator_vault_admin.key) {
        msg!("Invalid operator vault admin");
        return Err(ProgramError::InvalidAccountData);
    }

    let mut operator_vault_ticket_data = operator_vault_ticket.data.borrow_mut();
    let operator_vault_ticket =
        OperatorVaultTicket::try_from_slice_mut(&mut operator_vault_ticket_data)?;
    if operator_vault_ticket
        .state
        .deactivate(Clock::get()?.slot, config.epoch_length)
    {
        msg!("Vault is not ready to be deactivated");
        return Err(ProgramError::InvalidAccountData);
    }

    Ok(())
}
