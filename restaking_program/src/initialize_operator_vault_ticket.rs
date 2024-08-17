use std::mem::size_of;

use jito_account_traits::{AccountDeserialize, Discriminator};
use jito_jsm_core::{
    create_account,
    loader::{load_signer, load_system_account, load_system_program},
};
use jito_restaking_core::{
    config::Config, loader::load_operator, operator::Operator,
    operator_vault_ticket::OperatorVaultTicket,
};
use jito_vault_core::loader::{load_config, load_vault};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
    pubkey::Pubkey, rent::Rent, sysvar::Sysvar,
};

/// The node operator admin can add support for receiving delegation from a vault.
/// The vault can be used at the end of epoch + 1.
/// This method is permissioned to the node operator admin.
///
/// [`crate::RestakingInstruction::InitializeOperatorVaultTicket`]
pub fn process_initialize_operator_vault_ticket(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let [config, operator_info, vault, operator_vault_ticket_account, operator_vault_admin, payer, system_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    load_config(program_id, config, false)?;
    load_operator(program_id, operator_info, true)?;
    let mut config_data = config.data.borrow_mut();
    let config = Config::try_from_slice_mut(&mut config_data)?;
    load_vault(&config.vault_program, vault, false)?;
    load_system_account(operator_vault_ticket_account, true)?;
    load_signer(operator_vault_admin, false)?;
    load_signer(payer, true)?;
    load_system_program(system_program)?;

    // The OperatorVaultTicket shall be at the canonical PDA
    let (operator_vault_ticket_pubkey, operator_vault_ticket_bump, mut operator_vault_ticket_seeds) =
        OperatorVaultTicket::find_program_address(program_id, operator_info.key, vault.key);
    operator_vault_ticket_seeds.push(vec![operator_vault_ticket_bump]);
    if operator_vault_ticket_account
        .key
        .ne(&operator_vault_ticket_pubkey)
    {
        msg!("Operator vault ticket account is not at the correct PDA");
        return Err(ProgramError::InvalidArgument);
    }

    let mut operator_data = operator_info.data.borrow_mut();
    let operator = Operator::try_from_slice_mut(&mut operator_data)?;
    if operator.vault_admin.ne(operator_vault_admin.key) {
        msg!("Invalid operator vault admin");
        return Err(ProgramError::InvalidAccountData);
    }

    msg!(
        "Initializing OperatorVaultTicket at address {}",
        operator_vault_ticket_account.key
    );
    create_account(
        payer,
        operator_vault_ticket_account,
        system_program,
        program_id,
        &Rent::get()?,
        8_u64
            .checked_add(size_of::<OperatorVaultTicket>() as u64)
            .unwrap(),
        &operator_vault_ticket_seeds,
    )?;
    let mut operator_vault_ticket_account_data =
        operator_vault_ticket_account.try_borrow_mut_data()?;
    operator_vault_ticket_account_data[0] = OperatorVaultTicket::DISCRIMINATOR;
    let operator_vault_ticket =
        OperatorVaultTicket::try_from_slice_mut(&mut operator_vault_ticket_account_data)?;
    *operator_vault_ticket = OperatorVaultTicket::new(
        *operator_info.key,
        *vault.key,
        operator.vault_count,
        operator_vault_ticket_bump,
    );

    operator.vault_count = operator
        .vault_count
        .checked_add(1)
        .ok_or(ProgramError::InvalidAccountData)?;

    Ok(())
}
