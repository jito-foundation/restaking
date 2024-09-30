use jito_bytemuck::{AccountDeserialize, Discriminator};
use jito_jsm_core::loader::{
    load_associated_token_account, load_signer, load_system_program, load_token_mint,
    load_token_program,
};
use jito_restaking_core::{
    config::Config, ncn::Ncn, ncn_operator_state::NcnOperatorState,
    ncn_vault_slasher_ticket::NcnVaultSlasherTicket, ncn_vault_ticket::NcnVaultTicket,
    operator::Operator, operator_vault_ticket::OperatorVaultTicket,
};
use solana_program::program_pack::Pack;
use solana_program::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    pubkey::Pubkey,
};
use spl_token::instruction::{close_account, transfer};
use spl_token::state::Account;

/// Harvest Tokens from any account owned by the program
/// [`crate::RestakingInstruction::HarvestTokens`]
pub fn process_harvest_tokens(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64,
) -> ProgramResult {
    let [program_account, config_ncn_or_operator, harvest_admin, mint, program_token_account, destination_base, destination_token_account, token_program, system_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    load_system_program(system_program)?;
    load_token_program(token_program)?;
    load_signer(harvest_admin, false)?;
    load_token_mint(mint)?;
    load_associated_token_account(program_token_account, program_account.key, mint.key)?;
    load_associated_token_account(destination_token_account, destination_base.key, mint.key)?;

    if program_account.owner.ne(program_id) {
        msg!("Program account is not owned by the program");
        return Err(ProgramError::InvalidAccountData);
    }

    // ---- Check the admin ---

    let config_ncn_or_operator_discriminator = config_ncn_or_operator.data.borrow()[0];

    let (admin_from_config, admin_from_ncn, admin_from_operator) =
        match config_ncn_or_operator_discriminator {
            Config::DISCRIMINATOR => {
                let config_data = config_ncn_or_operator.data.borrow();
                let config = Config::try_from_slice_unchecked(&config_data)?;

                if config.admin.ne(harvest_admin.key) {
                    msg!("Config's admin does not match the admin provided");
                    return Err(ProgramError::InvalidAccountData);
                }

                (true, false, false)
            }
            Ncn::DISCRIMINATOR => {
                let ncn_data = config_ncn_or_operator.data.borrow();
                let ncn = Ncn::try_from_slice_unchecked(&ncn_data)?;

                if ncn.harvest_admin.ne(harvest_admin.key) {
                    msg!("Ncn's harvest admin does not match the admin provided");
                    return Err(ProgramError::InvalidAccountData);
                }

                (false, true, false)
            }
            Operator::DISCRIMINATOR => {
                let operator_data = config_ncn_or_operator.data.borrow();
                let operator = Operator::try_from_slice_unchecked(&operator_data)?;

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

    let account_data = program_account.data.borrow();
    let discriminator = account_data[0];

    let (account_bump, mut account_seeds) = match discriminator {
        Config::DISCRIMINATOR => {
            if !admin_from_config {
                msg!(
                    "Only the Config's admin has the authority to harvest from the Config account"
                );
                return Err(ProgramError::InvalidAccountData);
            }

            let (_, config_bump, config_seeds) = Config::find_program_address(program_id);

            (config_bump, config_seeds)
        }
        NcnOperatorState::DISCRIMINATOR => {
            if !admin_from_ncn {
                msg!("Only the NCN's harvest admin has the authority to harvest from the NcnOperatorState account");
                return Err(ProgramError::InvalidAccountData);
            }

            let ncn_operator_state = NcnOperatorState::try_from_slice_unchecked(&account_data)?;

            let (_, ncn_operator_state_bump, ncn_operator_state_seeds) =
                NcnOperatorState::find_program_address(
                    program_id,
                    &ncn_operator_state.ncn,
                    &ncn_operator_state.operator,
                );

            (ncn_operator_state_bump, ncn_operator_state_seeds)
        }
        NcnVaultSlasherTicket::DISCRIMINATOR => {
            if !admin_from_ncn {
                msg!("Only the NCN's harvest admin has the authority to harvest from the NcnVaultSlasherTicket account");
                return Err(ProgramError::InvalidAccountData);
            }

            let ncn_vault_slasher_ticket =
                NcnVaultSlasherTicket::try_from_slice_unchecked(&account_data)?;

            let (_, ncn_vault_slasher_ticket_bump, ncn_vault_slasher_ticket_seeds) =
                NcnVaultSlasherTicket::find_program_address(
                    program_id,
                    &ncn_vault_slasher_ticket.ncn,
                    &ncn_vault_slasher_ticket.vault,
                    &ncn_vault_slasher_ticket.slasher,
                );

            (
                ncn_vault_slasher_ticket_bump,
                ncn_vault_slasher_ticket_seeds,
            )
        }
        NcnVaultTicket::DISCRIMINATOR => {
            if !admin_from_ncn {
                msg!("Only the NCN's harvest admin has the authority to harvest from the NcnVaultTicket account");
                return Err(ProgramError::InvalidAccountData);
            }

            let ncn_vault_ticket = NcnVaultTicket::try_from_slice_unchecked(&account_data)?;

            let (_, ncn_vault_ticket_bump, ncn_vault_ticket_seeds) =
                NcnVaultTicket::find_program_address(
                    program_id,
                    &ncn_vault_ticket.ncn,
                    &ncn_vault_ticket.vault,
                );

            (ncn_vault_ticket_bump, ncn_vault_ticket_seeds)
        }
        Ncn::DISCRIMINATOR => {
            if !admin_from_ncn {
                msg!("Only the NCN's harvest admin has the authority to harvest from the Ncn account");
                return Err(ProgramError::InvalidAccountData);
            }

            let ncn = Ncn::try_from_slice_unchecked(&account_data)?;

            let (_, ncn_bump, ncn_seeds) = Ncn::find_program_address(program_id, &ncn.base);

            (ncn_bump, ncn_seeds)
        }
        OperatorVaultTicket::DISCRIMINATOR => {
            if !admin_from_operator {
                msg!("Only the Operator's harvest admin has the authority to harvest from the OperatorVaultTicket account");
                return Err(ProgramError::InvalidAccountData);
            }

            let operator_vault_ticket =
                OperatorVaultTicket::try_from_slice_unchecked(&account_data)?;

            let (_, operator_vault_ticket_bump, operator_vault_ticket_seeds) =
                OperatorVaultTicket::find_program_address(
                    program_id,
                    &operator_vault_ticket.operator,
                    &operator_vault_ticket.vault,
                );

            (operator_vault_ticket_bump, operator_vault_ticket_seeds)
        }
        Operator::DISCRIMINATOR => {
            if !admin_from_operator {
                msg!("Only the Operator's harvest admin has the authority to harvest from the Operator account");
                return Err(ProgramError::InvalidAccountData);
            }

            let operator = Operator::try_from_slice_unchecked(&account_data)?;

            let (_, operator_bump, operator_seeds) =
                Operator::find_program_address(program_id, &operator.base);

            (operator_bump, operator_seeds)
        }
        _ => {
            msg!("Invalid discriminator for program account");
            return Err(ProgramError::InvalidAccountData);
        }
    };

    account_seeds.push(vec![account_bump]);
    let seed_slices: Vec<&[u8]> = account_seeds.iter().map(|seed| seed.as_slice()).collect();

    // ---- Get amount to transfer -----

    let from_token_account = Account::unpack(&program_token_account.data.borrow())?;
    let tokens_in_account = from_token_account.amount;

    if tokens_in_account < amount {
        msg!(
            "Amount to transfer exceeds the amount in the token account {}/{}",
            amount,
            tokens_in_account
        );
        return Err(ProgramError::InsufficientFunds);
    }

    // ---- Transfer Tokens ----

    msg!(
        "Transferring {} tokens from program account to destination account {:?}",
        amount,
        destination_token_account.key
    );

    invoke_signed(
        &transfer(
            token_program.key,
            program_token_account.key,
            destination_token_account.key,
            program_account.key,
            &[],
            amount,
        )?,
        &[
            mint.clone(),
            program_token_account.clone(),
            destination_token_account.clone(),
            program_account.clone(),
        ],
        &[&seed_slices],
    )?;

    if amount == tokens_in_account {
        msg!("All tokens have been harvested, closing the program's token account");
        invoke(
            &close_account(
                token_program.key,
                program_token_account.key,
                destination_base.key,
                program_account.key,
                &[],
            )?,
            &[
                program_token_account.clone(),
                destination_base.clone(),
                program_account.clone(),
                system_program.clone(),
            ],
        )?;
    }

    Ok(())
}
