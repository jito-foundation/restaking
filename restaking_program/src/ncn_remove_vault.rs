use jito_account_traits::AccountDeserialize;
use jito_jsm_core::loader::load_signer;
use jito_restaking_core::config::Config;
use jito_restaking_core::loader::{load_ncn, load_ncn_vault_ticket};
use jito_restaking_core::ncn::Ncn;
use jito_restaking_core::ncn_vault_ticket::NcnVaultTicket;
use jito_vault_core::loader::load_config;
use solana_program::{
    account_info::AccountInfo, clock::Clock, entrypoint::ProgramResult, msg,
    program_error::ProgramError, pubkey::Pubkey, sysvar::Sysvar,
};

/// [`crate::RestakingInstruction::NcnRemoveVault`]
pub fn process_ncn_remove_vault(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let [config, ncn, vault, ncn_vault_ticket, ncn_vault_admin] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    load_config(program_id, config, false)?;
    load_ncn(program_id, ncn, false)?;
    load_ncn_vault_ticket(program_id, ncn_vault_ticket, ncn, vault, true)?;
    load_signer(ncn_vault_admin, false)?;

    let ncn = Ncn::try_from_slice(&ncn.data.borrow())?;
    if ncn.vault_admin.ne(&ncn_vault_admin.key) {
        msg!("Invalid vault admin for NCN");
        return Err(ProgramError::InvalidAccountData);
    }

    let config = Config::try_from_slice(&config.data.borrow())?;

    let ncn_vault_ticket =
        NcnVaultTicket::try_from_slice_mut(&mut ncn_vault_ticket.data.borrow_mut())?;

    if !ncn_vault_ticket
        .state
        .deactivate(Clock::get()?.slot, config.epoch_length)
    {
        msg!("Vault is not ready to be deactivated");
        return Err(ProgramError::InvalidAccountData);
    }

    Ok(())
}
