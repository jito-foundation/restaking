use jito_bytemuck::AccountDeserialize;
use jito_jsm_core::loader::load_signer;
use jito_restaking_core::{config::Config, ncn::Ncn, ncn_vault_ticket::NcnVaultTicket};
use jito_restaking_sdk::error::RestakingError;
use solana_program::{
    account_info::AccountInfo, clock::Clock, entrypoint::ProgramResult, msg,
    program_error::ProgramError, pubkey::Pubkey, sysvar::Sysvar,
};

/// [`crate::RestakingInstruction::CooldownNcnVaultTicket`]
pub fn process_cooldown_ncn_vault_ticket(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let [config, ncn, vault, ncn_vault_ticket, ncn_vault_admin] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    Config::load(program_id, config, false)?;
    Ncn::load(program_id, ncn, false)?;
    NcnVaultTicket::load(program_id, ncn_vault_ticket, ncn, vault, true)?;
    load_signer(ncn_vault_admin, false)?;

    // The NCN vault admin shall be the signer of the transaction
    let ncn_data = ncn.data.borrow();
    let ncn = Ncn::try_from_slice_unchecked(&ncn_data)?;
    if ncn.vault_admin.ne(ncn_vault_admin.key) {
        msg!("Invalid vault admin for NCN");
        return Err(RestakingError::NcnVaultAdminInvalid.into());
    }

    // The NcnVaultTicket shall be active before it can be cooled down
    let config_data = config.data.borrow();
    let config = Config::try_from_slice_unchecked(&config_data)?;
    let mut ncn_vault_ticket_data = ncn_vault_ticket.data.borrow_mut();
    let ncn_vault_ticket =
        NcnVaultTicket::try_from_slice_unchecked_mut(&mut ncn_vault_ticket_data)?;
    if !ncn_vault_ticket
        .state
        .deactivate(Clock::get()?.slot, config.epoch_length())?
    {
        msg!("Vault is not ready to be deactivated");
        return Err(RestakingError::NcnVaultTicketFailedCooldown.into());
    }

    msg!(
        "COOLDOWN NCN_VAULT_TICKET: NCN {} deactivating Vault {}",
        ncn_vault_ticket.ncn,
        ncn_vault_ticket.vault,
    );

    Ok(())
}
