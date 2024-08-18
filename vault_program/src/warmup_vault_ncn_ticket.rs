use jito_account_traits::AccountDeserialize;
use jito_jsm_core::loader::load_signer;
use jito_restaking_core::{ncn::Ncn, ncn_vault_ticket::NcnVaultTicket};
use jito_vault_core::{config::Config, vault::Vault, vault_ncn_ticket::VaultNcnTicket};
use jito_vault_sdk::error::VaultError;
use solana_program::{
    account_info::AccountInfo, clock::Clock, entrypoint::ProgramResult, msg,
    program_error::ProgramError, pubkey::Pubkey, sysvar::Sysvar,
};

/// Instruction: [`crate::VaultInstruction::WarmupVaultNcnTicket`]
pub fn process_warmup_vault_ncn_ticket(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let [config, vault, ncn, ncn_vault_ticket, vault_ncn_ticket, vault_ncn_admin] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    Config::load(program_id, config, false)?;
    Vault::load(program_id, vault, false)?;
    let config_data = config.data.borrow();
    let config = Config::try_from_slice_unchecked(&config_data)?;
    Ncn::load(&config.restaking_program, ncn, false)?;
    NcnVaultTicket::load(
        &config.restaking_program,
        ncn_vault_ticket,
        ncn,
        vault,
        false,
    )?;
    VaultNcnTicket::load(program_id, vault_ncn_ticket, vault, ncn, true)?;
    load_signer(vault_ncn_admin, false)?;

    // The Vault NCN admin shall be the signer of the transaction
    let vault_data = vault.data.borrow();
    let vault = Vault::try_from_slice_unchecked(&vault_data)?;
    if vault.ncn_admin.ne(vault_ncn_admin.key) {
        msg!("Invalid ncn admin for vault");
        return Err(VaultError::VaultNcnAdminInvalid.into());
    }

    // The vault shall be up-to-date before warming up the NCN
    if vault.is_update_needed(Clock::get()?.slot, config.epoch_length) {
        msg!("Vault update is needed");
        return Err(VaultError::VaultUpdateNeeded.into());
    }

    // The VaultNcnTicket shall be ready to be activated
    let mut vault_ncn_ticket_data = vault_ncn_ticket.data.borrow_mut();
    let vault_ncn_ticket =
        VaultNcnTicket::try_from_slice_unchecked_mut(&mut vault_ncn_ticket_data)?;
    if !vault_ncn_ticket
        .state
        .activate(Clock::get()?.slot, config.epoch_length)
    {
        msg!("VaultNcnTicket is not ready to be activated");
        return Err(VaultError::VaultNcnTicketFailedWarmup.into());
    }

    Ok(())
}
