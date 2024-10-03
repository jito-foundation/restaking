use jito_bytemuck::{AccountDeserialize, Discriminator};
use jito_jsm_core::loader::{
    load_associated_token_account, load_signer, load_system_program, load_token_mint,
    load_token_program,
};
use jito_vault_core::{
    config::Config, vault::Vault, vault_ncn_slasher_operator_ticket::VaultNcnSlasherOperatorTicket,
    vault_ncn_slasher_ticket::VaultNcnSlasherTicket, vault_ncn_ticket::VaultNcnTicket,
    vault_operator_delegation::VaultOperatorDelegation,
    vault_staker_withdrawal_ticket::VaultStakerWithdrawalTicket,
    vault_update_state_tracker::VaultUpdateStateTracker,
};
use solana_program::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
};
use spl_token::{
    instruction::{close_account, transfer},
    state::Account,
};

/// Harvest Lamports from any account owned by the program
/// [`crate::RestakingInstruction::HarvestLamports`]
pub fn process_harvest_tokens(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64,
) -> ProgramResult {
    let [program_account, config_or_vault, harvest_admin, mint, program_token_account, destination_base, destination_token_account, token_program, system_program] =
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

    let config_or_vault_discriminator = config_or_vault.data.borrow()[0];

    let (admin_from_config, admin_from_vault) = match config_or_vault_discriminator {
        Config::DISCRIMINATOR => {
            let mut config_data = config_or_vault.data.borrow_mut();
            let config = Config::try_from_slice_unchecked_mut(&mut config_data)?;

            if config.admin.ne(harvest_admin.key) {
                msg!("Config's admin does not match the admin provided");
                return Err(ProgramError::InvalidAccountData);
            }

            (true, false)
        }
        Vault::DISCRIMINATOR => {
            let mut vault_data = config_or_vault.data.borrow_mut();
            let vault = Vault::try_from_slice_unchecked_mut(&mut vault_data)?;

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
        VaultNcnSlasherOperatorTicket::DISCRIMINATOR => {
            if !admin_from_vault {
                msg!("Only the Vault's harvest admin has the authority to harvest from the VaultNcnSlasherOperatorTicket account");
                return Err(ProgramError::InvalidAccountData);
            }

            let vault_ncn_slasher_operator_ticket =
                VaultNcnSlasherOperatorTicket::try_from_slice_unchecked(&account_data)?;

            let (
                _,
                vault_ncn_slasher_operator_ticket_bump,
                vault_ncn_slasher_operator_ticket_seeds,
            ) = VaultNcnSlasherOperatorTicket::find_program_address(
                program_id,
                &vault_ncn_slasher_operator_ticket.vault,
                &vault_ncn_slasher_operator_ticket.ncn,
                &vault_ncn_slasher_operator_ticket.slasher,
                &vault_ncn_slasher_operator_ticket.operator,
                vault_ncn_slasher_operator_ticket.epoch(),
            );

            (
                vault_ncn_slasher_operator_ticket_bump,
                vault_ncn_slasher_operator_ticket_seeds,
            )
        }
        VaultNcnSlasherTicket::DISCRIMINATOR => {
            if !admin_from_vault {
                msg!("Only the Vaults's harvest admin has the authority to harvest from the VaultNcnSlasherTicket account");
                return Err(ProgramError::InvalidAccountData);
            }

            let vault_ncn_slasher_ticket =
                VaultNcnSlasherTicket::try_from_slice_unchecked(&account_data)?;

            let (_, vault_ncn_slasher_ticket_bump, vault_ncn_slasher_ticket_seeds) =
                VaultNcnSlasherTicket::find_program_address(
                    program_id,
                    &vault_ncn_slasher_ticket.vault,
                    &vault_ncn_slasher_ticket.ncn,
                    &vault_ncn_slasher_ticket.slasher,
                );

            (
                vault_ncn_slasher_ticket_bump,
                vault_ncn_slasher_ticket_seeds,
            )
        }
        VaultNcnTicket::DISCRIMINATOR => {
            if !admin_from_vault {
                msg!("Only the Vault's harvest admin has the authority to harvest from the VaultNcnTicket account");
                return Err(ProgramError::InvalidAccountData);
            }

            let vault_ncn_ticket = VaultNcnTicket::try_from_slice_unchecked(&account_data)?;

            let (_, vault_ncn_ticket_bump, vault_ncn_ticket_seeds) =
                VaultNcnTicket::find_program_address(
                    program_id,
                    &vault_ncn_ticket.vault,
                    &vault_ncn_ticket.ncn,
                );

            (vault_ncn_ticket_bump, vault_ncn_ticket_seeds)
        }
        VaultOperatorDelegation::DISCRIMINATOR => {
            if !admin_from_vault {
                msg!("Only the Vault's harvest admin has the authority to harvest from the VaultOperatorDelegation account");
                return Err(ProgramError::InvalidAccountData);
            }

            let vault_operator_delegation =
                VaultOperatorDelegation::try_from_slice_unchecked(&account_data)?;

            let (_, vault_operator_delegation_bump, vault_operator_delegation_seeds) =
                VaultOperatorDelegation::find_program_address(
                    program_id,
                    &vault_operator_delegation.vault,
                    &vault_operator_delegation.operator,
                );

            (
                vault_operator_delegation_bump,
                vault_operator_delegation_seeds,
            )
        }
        VaultStakerWithdrawalTicket::DISCRIMINATOR => {
            if !admin_from_vault {
                msg!("Only the Vault's harvest admin has the authority to harvest from the VaultStakerWithdrawalTicket account");
                return Err(ProgramError::InvalidAccountData);
            }

            let vault_staker_withdrawal_ticket =
                VaultStakerWithdrawalTicket::try_from_slice_unchecked(&account_data)?;

            let (_, vault_staker_withdrawal_ticket_bump, vault_staker_withdrawal_ticket_seeds) =
                VaultStakerWithdrawalTicket::find_program_address(
                    program_id,
                    &vault_staker_withdrawal_ticket.vault,
                    &vault_staker_withdrawal_ticket.staker,
                );

            (
                vault_staker_withdrawal_ticket_bump,
                vault_staker_withdrawal_ticket_seeds,
            )
        }
        VaultUpdateStateTracker::DISCRIMINATOR => {
            if !admin_from_vault {
                msg!("Only the Vault's harvest admin has the authority to harvest from the VaultUpdateStateTracker account");
                return Err(ProgramError::InvalidAccountData);
            }

            let vault_update_state_tracker =
                VaultUpdateStateTracker::try_from_slice_unchecked(&account_data)?;

            let (_, vault_update_state_tracker_bump, vault_update_state_tracker_seeds) =
                VaultUpdateStateTracker::find_program_address(
                    program_id,
                    &vault_update_state_tracker.vault,
                    vault_update_state_tracker.ncn_epoch(),
                );

            (
                vault_update_state_tracker_bump,
                vault_update_state_tracker_seeds,
            )
        }
        Vault::DISCRIMINATOR => {
            if !admin_from_vault {
                msg!("Only the Vault's harvest admin has the authority to harvest from the Vault account");
                return Err(ProgramError::InvalidAccountData);
            }

            let vault = Vault::try_from_slice_unchecked(&account_data)?;

            // Super important to prevent the vault's supported mint from being withdrawn
            if vault.supported_mint.eq(mint.key) {
                msg!("You cannot withdraw the vault's supported mint");
                return Err(ProgramError::InvalidAccountData);
            }

            let (_, vault_bump, vault_seeds) = Vault::find_program_address(program_id, &vault.base);

            (vault_bump, vault_seeds)
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
