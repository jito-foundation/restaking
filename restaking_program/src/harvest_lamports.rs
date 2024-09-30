use jito_bytemuck::{AccountDeserialize, Discriminator};
use jito_jsm_core::loader::load_signer;
use jito_restaking_core::{
    config::Config, ncn::Ncn, ncn_operator_state::NcnOperatorState,
    ncn_vault_slasher_ticket::NcnVaultSlasherTicket, ncn_vault_ticket::NcnVaultTicket,
    operator::Operator, operator_vault_ticket::OperatorVaultTicket,
};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
    pubkey::Pubkey, rent::Rent, sysvar::Sysvar,
};

/// Harvest Lamports from any account owned by the program
/// [`crate::RestakingInstruction::HarvestLamports`]
pub fn process_harvest_lamports(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let [program_account, config_ncn_or_operator, harvest_admin, destination] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    load_signer(harvest_admin, false)?;

    if program_account.owner.ne(program_id) {
        msg!("Program account is not owned by the program");
        return Err(ProgramError::InvalidAccountData);
    }

    // ---- Check the admin ---

    let config_ncn_or_operator_discriminator = config_ncn_or_operator.data.borrow()[0];

    let (admin_from_config, admin_from_ncn, admin_from_operator) =
        match config_ncn_or_operator_discriminator {
            Config::DISCRIMINATOR => {
                let mut config_data = config_ncn_or_operator.data.borrow();
                let config = Config::try_from_slice_unchecked(&mut config_data)?;

                if config.admin.ne(harvest_admin.key) {
                    msg!("Config's admin does not match the admin provided");
                    return Err(ProgramError::InvalidAccountData);
                }

                (true, false, false)
            }
            Ncn::DISCRIMINATOR => {
                let mut ncn_data = config_ncn_or_operator.data.borrow();
                let ncn = Ncn::try_from_slice_unchecked(&mut ncn_data)?;

                if ncn.harvest_admin.ne(harvest_admin.key) {
                    msg!("Ncn's harvest admin does not match the admin provided");
                    return Err(ProgramError::InvalidAccountData);
                }

                (false, true, false)
            }
            Operator::DISCRIMINATOR => {
                let mut operator_data = config_ncn_or_operator.data.borrow();
                let operator = Operator::try_from_slice_unchecked(&mut operator_data)?;

                if operator.harvest_admin.ne(harvest_admin.key) {
                    msg!("Operator's harvest admin does not match the admin provided");
                    return Err(ProgramError::InvalidAccountData);
                }

                (false, false, true)
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
        NcnOperatorState::DISCRIMINATOR => {
            if !admin_from_ncn && !admin_from_operator {
                msg!("Only the NCN or the Config's harvest admin has the authority to harvest from the NcnOperatorState account");
                return Err(ProgramError::InvalidAccountData);
            }
        }
        NcnVaultSlasherTicket::DISCRIMINATOR => {
            if !admin_from_ncn {
                msg!("Only the NCN's harvest admin has the authority to harvest from the NcnVaultSlasherTicket account");
                return Err(ProgramError::InvalidAccountData);
            }
        }
        NcnVaultTicket::DISCRIMINATOR => {
            if !admin_from_ncn {
                msg!("Only the NCN's harvest admin has the authority to harvest from the NcnVaultTicket account");
                return Err(ProgramError::InvalidAccountData);
            }
        }
        Ncn::DISCRIMINATOR => {
            if !admin_from_ncn {
                msg!("Only the NCN's harvest admin has the authority to harvest from the Ncn account");
                return Err(ProgramError::InvalidAccountData);
            }
        }
        OperatorVaultTicket::DISCRIMINATOR => {
            if !admin_from_operator {
                msg!("Only the Operator's harvest admin has the authority to harvest from the OperatorVaultTicket account");
                return Err(ProgramError::InvalidAccountData);
            }
        }
        Operator::DISCRIMINATOR => {
            if !admin_from_operator {
                msg!("Only the Operator's harvest admin has the authority to harvest from the Operator account");
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
