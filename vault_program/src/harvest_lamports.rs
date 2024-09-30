use jito_bytemuck::{AccountDeserialize, Discriminator};
use jito_jsm_core::loader::load_signer;

use jito_vault_core::{
    config::Config, vault::Vault, vault_ncn_slasher_operator_ticket::VaultNcnSlasherOperatorTicket,
    vault_ncn_slasher_ticket::VaultNcnSlasherTicket, vault_ncn_ticket::VaultNcnTicket,
    vault_operator_delegation::VaultOperatorDelegation,
    vault_staker_withdrawal_ticket::VaultStakerWithdrawalTicket,
    vault_update_state_tracker::VaultUpdateStateTracker,
};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
    pubkey::Pubkey, rent::Rent, sysvar::Sysvar,
};

/// Harvest Lamports from any account owned by the program
/// [`crate::RestakingInstruction::HarvestLamports`]
pub fn process_harvest_lamports(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let [program_account, config_or_vault, harvest_admin, destination] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    load_signer(harvest_admin, false)?;

    if program_account.owner.ne(program_id) {
        msg!("Program account is not owned by the program");
        return Err(ProgramError::InvalidAccountData);
    }

    // ---- Check the admin ---

    let config_or_vault_discriminator = config_or_vault.data.borrow()[0];

    let (admin_from_config, admin_from_vault) = match config_or_vault_discriminator {
        Config::DISCRIMINATOR => {
            let config_data = config_or_vault.data.borrow();
            let config = Config::try_from_slice_unchecked(&config_data)?;

            if config.admin.ne(harvest_admin.key) {
                msg!("Config's admin does not match the admin provided");
                return Err(ProgramError::InvalidAccountData);
            }

            (true, false)
        }
        Vault::DISCRIMINATOR => {
            let vault_data = config_or_vault.data.borrow();
            let vault = Vault::try_from_slice_unchecked(&vault_data)?;

            if vault.harvest_admin.ne(harvest_admin.key) {
                msg!("Vault's harvest admin does not match the admin provided");
                return Err(ProgramError::InvalidAccountData);
            }

            (false, true)
        }
        _ => {
            msg!("Invalid discriminator for config, ncn or operator account");
            return Err(ProgramError::InvalidAccountData);
        }
    };

    // ---- Check the admin matches the correct program account -----

    let discriminator = program_account.data.borrow()[0];

    match discriminator {
        Config::DISCRIMINATOR => {
            if !admin_from_config {
                msg!(
                    "Only the Config's admin has the authority to harvest from the Config account"
                );
                return Err(ProgramError::InvalidAccountData);
            }
        }
        VaultNcnSlasherOperatorTicket::DISCRIMINATOR => {
            if !admin_from_vault {
                msg!("Only the Vault's harvest admin has the authority to harvest from the VaultNcnSlasherOperatorTicket account");
                return Err(ProgramError::InvalidAccountData);
            }
        }
        VaultNcnSlasherTicket::DISCRIMINATOR => {
            if !admin_from_vault {
                msg!("Only the Vaults's harvest admin has the authority to harvest from the VaultNcnSlasherTicket account");
                return Err(ProgramError::InvalidAccountData);
            }
        }
        VaultNcnTicket::DISCRIMINATOR => {
            if !admin_from_vault {
                msg!("Only the Vault's harvest admin has the authority to harvest from the VaultNcnTicket account");
                return Err(ProgramError::InvalidAccountData);
            }
        }
        VaultOperatorDelegation::DISCRIMINATOR => {
            if !admin_from_vault {
                msg!("Only the Vault's harvest admin has the authority to harvest from the VaultOperatorDelegation account");
                return Err(ProgramError::InvalidAccountData);
            }
        }
        VaultStakerWithdrawalTicket::DISCRIMINATOR => {
            if !admin_from_vault {
                msg!("Only the Vault's harvest admin has the authority to harvest from the VaultStakerWithdrawalTicket account");
                return Err(ProgramError::InvalidAccountData);
            }
        }
        VaultUpdateStateTracker::DISCRIMINATOR => {
            if !admin_from_vault {
                msg!("Only the Vault's harvest admin has the authority to harvest from the VaultUpdateStateTracker account");
                return Err(ProgramError::InvalidAccountData);
            }
        }
        _ => {
            msg!("Invalid discriminator for program account");
            return Err(ProgramError::InvalidAccountData);
        }
    };

    // ---- Get amount to transfer -----
    let data_size = program_account.data.borrow().len() as u64;
    let rent = Rent::get()?.minimum_balance(data_size as usize);
    let amount_to_transfer = program_account
        .lamports()
        .checked_sub(rent)
        .ok_or(ProgramError::InsufficientFunds)?;

    // ---- Transfer lamports ----

    msg!(
        "Transferring {} lamports from program account to destination account {:?}",
        amount_to_transfer,
        destination.key
    );

    **program_account.try_borrow_mut_lamports()? = {
        program_account
            .lamports
            .borrow()
            .checked_sub(amount_to_transfer)
            .ok_or(ProgramError::InsufficientFunds)?
    };
    **destination.try_borrow_mut_lamports()? = {
        destination
            .lamports
            .borrow()
            .checked_add(amount_to_transfer)
            .ok_or(ProgramError::ArithmeticOverflow)?
    };

    Ok(())
}
