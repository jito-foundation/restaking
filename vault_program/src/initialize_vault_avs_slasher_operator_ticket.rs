use borsh::BorshSerialize;
use jito_restaking_core::{avs::SanitizedAvs, operator::SanitizedOperator};
use jito_restaking_sanitization::{
    assert_with_msg, create_account, empty_account::EmptyAccount, signer::SanitizedSignerAccount,
    system_program::SanitizedSystemProgram,
};
use jito_vault_core::{
    config::SanitizedConfig, vault::SanitizedVault,
    vault_avs_slasher_operator_ticket::VaultAvsSlasherOperatorTicket,
    vault_avs_slasher_ticket::SanitizedVaultAvsSlasherTicket,
};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::Clock,
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
    rent::Rent,
    sysvar::Sysvar,
};

/// Instruction: [`crate::VaultInstruction::InitializeVaultAvsSlasherOperatorTicket`]
pub fn process_initialize_vault_avs_slasher_operator_ticket(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let SanitizedAccounts {
        config,
        vault,
        avs,
        operator,
        slasher,
        vault_avs_slasher_ticket,
        vault_avs_slasher_operator_ticket,
        payer,
        system_program,
    } = SanitizedAccounts::sanitize(program_id, accounts)?;

    let slot = Clock::get()?.slot;
    vault_avs_slasher_ticket
        .vault_avs_slasher_ticket()
        .check_active_or_cooldown(slot, config.config().epoch_length())?;

    let epoch = slot.checked_div(config.config().epoch_length()).unwrap();

    _create_vault_avs_slasher_operator_ticket(
        program_id,
        &vault_avs_slasher_operator_ticket,
        &vault,
        &avs,
        &operator,
        slasher,
        &payer,
        &system_program,
        &Rent::get()?,
        epoch,
    )?;

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn _create_vault_avs_slasher_operator_ticket<'a, 'info>(
    program_id: &Pubkey,
    vault_avs_slasher_operator_ticket_account: &EmptyAccount<'a, 'info>,
    vault: &SanitizedVault<'a, 'info>,
    avs: &SanitizedAvs<'a, 'info>,
    operator: &SanitizedOperator<'a, 'info>,
    slasher: &'a AccountInfo<'info>,
    payer: &SanitizedSignerAccount<'a, 'info>,
    system_program: &SanitizedSystemProgram<'a, 'info>,
    rent: &Rent,
    epoch: u64,
) -> ProgramResult {
    let (address, bump, mut seeds) = VaultAvsSlasherOperatorTicket::find_program_address(
        program_id,
        vault.account().key,
        avs.account().key,
        slasher.key,
        operator.account().key,
        epoch,
    );
    seeds.push(vec![bump]);

    assert_with_msg(
        address == *vault_avs_slasher_operator_ticket_account.account().key,
        ProgramError::InvalidAccountData,
        "Invalid vault AVS slasher operator ticket PDA",
    )?;

    let vault_avs_slasher_operator_ticket = VaultAvsSlasherOperatorTicket::new(
        *vault.account().key,
        *avs.account().key,
        *slasher.key,
        *operator.account().key,
        epoch,
        0,
        bump,
    );

    msg!(
        "Creating vault AVS slasher operator ticket: {:?}",
        vault_avs_slasher_operator_ticket_account.account().key
    );
    let serialized = vault_avs_slasher_operator_ticket.try_to_vec()?;
    create_account(
        payer.account(),
        vault_avs_slasher_operator_ticket_account.account(),
        system_program.account(),
        program_id,
        rent,
        serialized.len() as u64,
        &seeds,
    )?;
    vault_avs_slasher_operator_ticket_account
        .account()
        .data
        .borrow_mut()[..serialized.len()]
        .copy_from_slice(&serialized);
    Ok(())
}

struct SanitizedAccounts<'a, 'info> {
    config: SanitizedConfig<'a, 'info>,
    vault: SanitizedVault<'a, 'info>,
    avs: SanitizedAvs<'a, 'info>,
    operator: SanitizedOperator<'a, 'info>,
    slasher: &'a AccountInfo<'info>,
    vault_avs_slasher_ticket: SanitizedVaultAvsSlasherTicket<'a, 'info>,
    vault_avs_slasher_operator_ticket: EmptyAccount<'a, 'info>,
    payer: SanitizedSignerAccount<'a, 'info>,
    system_program: SanitizedSystemProgram<'a, 'info>,
}

impl<'a, 'info> SanitizedAccounts<'a, 'info> {
    /// Sanitizes the accounts for the instruction: [`crate::VaultInstruction::InitializeVaultAvsSlasherOperatorTicket`]
    pub fn sanitize(
        program_id: &Pubkey,
        accounts: &'a [AccountInfo<'info>],
    ) -> Result<SanitizedAccounts<'a, 'info>, ProgramError> {
        let accounts_iter = &mut accounts.iter();

        let config =
            SanitizedConfig::sanitize(program_id, next_account_info(accounts_iter)?, false)?;
        let vault = SanitizedVault::sanitize(program_id, next_account_info(accounts_iter)?, false)?;
        let avs = SanitizedAvs::sanitize(
            &config.config().restaking_program(),
            next_account_info(accounts_iter)?,
            false,
        )?;
        let slasher = next_account_info(accounts_iter)?;
        let operator = SanitizedOperator::sanitize(
            &config.config().restaking_program(),
            next_account_info(accounts_iter)?,
            false,
        )?;
        let vault_avs_slasher_ticket = SanitizedVaultAvsSlasherTicket::sanitize(
            program_id,
            next_account_info(accounts_iter)?,
            false,
            vault.account().key,
            avs.account().key,
            slasher.key,
        )?;
        let vault_avs_slasher_operator_ticket =
            EmptyAccount::sanitize(next_account_info(accounts_iter)?, true)?;
        let payer = SanitizedSignerAccount::sanitize(next_account_info(accounts_iter)?, true)?;
        let system_program = SanitizedSystemProgram::sanitize(next_account_info(accounts_iter)?)?;

        Ok(SanitizedAccounts {
            config,
            vault,
            avs,
            operator,
            slasher,
            vault_avs_slasher_ticket,
            vault_avs_slasher_operator_ticket,
            payer,
            system_program,
        })
    }
}
