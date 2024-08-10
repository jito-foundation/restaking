use std::mem::size_of;

use jito_account_traits::{AccountDeserialize, Discriminator};
use jito_jsm_core::{
    create_account,
    loader::{load_signer, load_system_account, load_system_program},
};
use jito_restaking_core::loader::{load_ncn, load_operator};
use jito_vault_core::{
    config::Config,
    loader::{load_config, load_vault, load_vault_ncn_slasher_ticket},
    vault_ncn_slasher_operator_ticket::VaultNcnSlasherOperatorTicket,
};
use solana_program::{
    account_info::AccountInfo, clock::Clock, entrypoint::ProgramResult, msg,
    program_error::ProgramError, pubkey::Pubkey, rent::Rent, sysvar::Sysvar,
};

/// Instruction: [`crate::VaultInstruction::InitializeVaultNcnSlasherOperatorTicket`]
pub fn process_initialize_vault_ncn_slasher_operator_ticket(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let [config, vault, ncn, slasher, operator, vault_ncn_slasher_ticket, vault_ncn_slasher_operator_ticket, payer, system_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    load_config(program_id, config, false)?;
    load_vault(program_id, vault, false)?;
    let config_data = config.data.borrow();
    let config = Config::try_from_slice(&config_data)?;
    load_ncn(&config.restaking_program, ncn, false)?;
    load_operator(&config.restaking_program, operator, false)?;
    load_vault_ncn_slasher_ticket(
        program_id,
        vault_ncn_slasher_ticket,
        vault,
        ncn,
        slasher,
        false,
    )?;
    load_system_account(vault_ncn_slasher_operator_ticket, true)?;
    load_signer(payer, false)?;
    load_system_program(system_program)?;

    let ncn_epoch = Clock::get()?.slot.checked_div(config.epoch_length).unwrap();

    let (
        vault_ncn_slasher_operator_ticket_pubkey,
        vault_ncn_slasher_operator_ticket_bump,
        mut vault_ncn_slasher_operator_ticket_seeds,
    ) = VaultNcnSlasherOperatorTicket::find_program_address(
        program_id,
        vault.key,
        ncn.key,
        slasher.key,
        operator.key,
        ncn_epoch,
    );
    vault_ncn_slasher_operator_ticket_seeds.push(vec![vault_ncn_slasher_operator_ticket_bump]);
    if vault_ncn_slasher_operator_ticket
        .key
        .ne(&vault_ncn_slasher_operator_ticket_pubkey)
    {
        msg!("Vault NCN slasher operator ticket is not at the correct PDA");
        return Err(ProgramError::InvalidAccountData);
    }

    msg!(
        "Initializing vault NCN slasher operator ticket at address {}",
        vault_ncn_slasher_operator_ticket.key
    );
    create_account(
        payer,
        vault_ncn_slasher_operator_ticket,
        system_program,
        program_id,
        &Rent::get()?,
        8_u64
            .checked_add(size_of::<VaultNcnSlasherOperatorTicket>() as u64)
            .unwrap(),
        &vault_ncn_slasher_operator_ticket_seeds,
    )?;

    let mut vault_ncn_slasher_operator_ticket_data =
        vault_ncn_slasher_operator_ticket.try_borrow_mut_data()?;
    vault_ncn_slasher_operator_ticket_data[0] = VaultNcnSlasherOperatorTicket::DISCRIMINATOR;
    let vault_ncn_slasher_operator_ticket = VaultNcnSlasherOperatorTicket::try_from_slice_mut(
        &mut vault_ncn_slasher_operator_ticket_data,
    )?;
    *vault_ncn_slasher_operator_ticket = VaultNcnSlasherOperatorTicket::new(
        *vault.key,
        *ncn.key,
        *slasher.key,
        *operator.key,
        ncn_epoch,
        vault_ncn_slasher_operator_ticket_bump,
    );

    Ok(())
}
