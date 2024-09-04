use jito_bytemuck::AccountDeserialize;
use jito_jsm_core::loader::load_signer;
use jito_restaking_core::{
    config::Config, ncn::Ncn, ncn_vault_slasher_ticket::NcnVaultSlasherTicket,
};
use jito_restaking_sdk::error::RestakingError;
use jito_vault_core::vault::Vault;
use solana_program::{
    account_info::AccountInfo, clock::Clock, entrypoint::ProgramResult, msg,
    program_error::ProgramError, pubkey::Pubkey, sysvar::Sysvar,
};

/// [`crate::RestakingInstruction::CooldownNcnVaultSlasherTicket`]
pub fn process_cooldown_ncn_vault_slasher_ticket(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let [config, ncn, vault, slasher, ncn_vault_slasher_ticket, ncn_slasher_admin] = accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    Config::load(program_id, config, false)?;
    Ncn::load(program_id, ncn, false)?;
    let config_data = config.data.borrow();
    let config = Config::try_from_slice_unchecked(&config_data)?;
    Vault::load(&config.vault_program, vault, false)?;
    NcnVaultSlasherTicket::load(
        program_id,
        ncn_vault_slasher_ticket,
        ncn,
        vault,
        slasher,
        true,
    )?;
    load_signer(ncn_slasher_admin, false)?;

    // The NCN slasher admin shall be the signer of the transaction
    let ncn_data = ncn.data.borrow();
    let ncn = Ncn::try_from_slice_unchecked(&ncn_data)?;
    if ncn.slasher_admin.ne(ncn_slasher_admin.key) {
        msg!("Invalid slasher admin for NCN");
        return Err(RestakingError::NcnSlasherAdminInvalid.into());
    }

    // The NcnVaultSlasherTicket shall be active before it can be cooled down
    let mut ncn_vault_slasher_ticket_data = ncn_vault_slasher_ticket.data.borrow_mut();
    let ncn_vault_slasher_ticket =
        NcnVaultSlasherTicket::try_from_slice_unchecked_mut(&mut ncn_vault_slasher_ticket_data)?;
    if !ncn_vault_slasher_ticket
        .state
        .deactivate(Clock::get()?.slot, config.epoch_length())
    {
        msg!("Slasher is not ready to be deactivated");
        return Err(RestakingError::NcnVaultSlasherTicketFailedCooldown.into());
    }

    Ok(())
}
