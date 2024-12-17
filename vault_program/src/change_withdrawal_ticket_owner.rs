use jito_bytemuck::AccountDeserialize;
use jito_jsm_core::loader::load_signer;
use jito_vault_core::{
    config::Config, vault::Vault, vault_staker_withdrawal_ticket::VaultStakerWithdrawalTicket,
};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
    pubkey::Pubkey,
};

pub fn process_change_withdrawal_ticket_owner(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let [config, vault_info, vault_staker_withdrawal_ticket, old_owner, new_owner] = accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    Config::load(program_id, config, false)?;
    Vault::load(program_id, vault_info, false)?;
    let vault_data = vault_info.data.borrow();
    let vault = Vault::try_from_slice_unchecked(&vault_data)?;
    VaultStakerWithdrawalTicket::load(program_id, vault_staker_withdrawal_ticket, true)?;
    let mut vault_staker_withdrawal_ticket_data = vault_staker_withdrawal_ticket.data.borrow_mut();
    let vault_staker_withdrawal_ticket = VaultStakerWithdrawalTicket::try_from_slice_unchecked_mut(
        &mut vault_staker_withdrawal_ticket_data,
    )?;
    load_signer(old_owner, false)?;

    vault.check_is_paused()?;

    vault_staker_withdrawal_ticket.check_staker(old_owner.key)?;
    vault_staker_withdrawal_ticket.staker = *new_owner.key;

    Ok(())
}
