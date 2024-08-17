use jito_account_traits::AccountDeserialize;
use jito_jsm_core::loader::load_signer;
use jito_restaking_core::{
    config::Config, ncn::Ncn, ncn_vault_slasher_ticket::NcnVaultSlasherTicket,
    ncn_vault_ticket::NcnVaultTicket,
};
use jito_restaking_sdk::error::RestakingError;
use jito_vault_core::vault::Vault;
use solana_program::{
    account_info::AccountInfo, clock::Clock, entrypoint::ProgramResult, msg,
    program_error::ProgramError, pubkey::Pubkey, sysvar::Sysvar,
};

/// [`crate::RestakingInstruction::WarmupNcnVaultSlasherTicket`]
pub fn process_warmup_ncn_vault_slasher_ticket(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let [config, ncn, vault, slasher, ncn_vault_ticket, ncn_vault_slasher_ticket, admin] = accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    Config::load(program_id, config, false)?;
    Ncn::load(program_id, ncn, false)?;
    let config_data = config.data.borrow_mut();
    let config = Config::try_from_slice_unchecked(&config_data)?;
    Vault::load(&config.vault_program, vault, false)?;
    NcnVaultTicket::load(program_id, ncn_vault_ticket, ncn, vault, false)?;
    NcnVaultSlasherTicket::load(
        program_id,
        ncn_vault_slasher_ticket,
        ncn,
        vault,
        slasher,
        true,
    )?;
    load_signer(admin, false)?;

    // The NCN slasher admin shall be the signer of the transaction
    let ncn_data = ncn.data.borrow();
    let ncn = Ncn::try_from_slice_unchecked(&ncn_data)?;
    if ncn.slasher_admin.ne(admin.key) {
        msg!("Invalid slasher admin for NCN");
        return Err(RestakingError::NcnSlasherAdminInvalid.into());
    }

    // The NcnVaultSlasherTicket shall be inactive before it can warmed up
    let mut ncn_vault_slasher_ticket_data = ncn_vault_slasher_ticket.data.borrow_mut();
    let ncn_vault_slasher_ticket =
        NcnVaultSlasherTicket::try_from_slice_unchecked_mut(&mut ncn_vault_slasher_ticket_data)?;
    if !ncn_vault_slasher_ticket
        .state
        .activate(Clock::get()?.slot, config.epoch_length)
    {
        msg!("Slasher is not ready to be activated");
        return Err(RestakingError::NcnVaultSlasherTicketFailedWarmup.into());
    }

    Ok(())
}
