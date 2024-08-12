use jito_account_traits::AccountDeserialize;
use jito_jsm_core::loader::load_signer;
use jito_restaking_core::{
    config::Config,
    loader::{load_config, load_ncn, load_ncn_vault_ticket},
    ncn::Ncn,
    ncn_vault_ticket::NcnVaultTicket,
};
use jito_restaking_sdk::error::RestakingError;
use jito_vault_core::loader::load_vault;
use solana_program::{
    account_info::AccountInfo, clock::Clock, entrypoint::ProgramResult, msg,
    program_error::ProgramError, pubkey::Pubkey, sysvar::Sysvar,
};

/// [`crate::RestakingInstruction::WarmupNcnVaultTicket`]
pub fn process_warmup_ncn_vault_ticket(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let [config, ncn, vault, ncn_vault_ticket, ncn_vault_admin] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    load_config(program_id, config, false)?;
    load_ncn(program_id, ncn, false)?;
    let config_data = config.data.borrow_mut();
    let config = Config::try_from_slice(&config_data)?;
    load_vault(&config.vault_program, vault, false)?;
    load_ncn_vault_ticket(program_id, ncn_vault_ticket, ncn, vault, true)?;
    load_signer(ncn_vault_admin, false)?;

    // The NCN vault admin shall be the signer of the transaction
    let ncn_data = ncn.data.borrow();
    let ncn = Ncn::try_from_slice(&ncn_data)?;
    if ncn.vault_admin.ne(ncn_vault_admin.key) {
        msg!("Invalid vault admin for NCN");
        return Err(RestakingError::NcnVaultAdminInvalid.into());
    }

    // The NcnVaultTicket shall be inactive before it can warmed up
    let mut ncn_vault_ticket_data = ncn_vault_ticket.data.borrow_mut();
    let ncn_vault_ticket = NcnVaultTicket::try_from_slice_mut(&mut ncn_vault_ticket_data)?;
    if !ncn_vault_ticket
        .state
        .activate(Clock::get()?.slot, config.epoch_length)
    {
        msg!("Vault is not ready to be activated");
        return Err(RestakingError::NcnVaultTicketFailedWarmup.into());
    }

    Ok(())
}
