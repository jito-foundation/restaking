use borsh::BorshSerialize;
use jito_restaking_core::{
    operator::SanitizedOperator, operator_vault_ticket::SanitizedOperatorVaultTicket,
};
use jito_restaking_sanitization::{
    assert_with_msg, create_account, empty_account::EmptyAccount, signer::SanitizedSignerAccount,
    system_program::SanitizedSystemProgram,
};
use jito_vault_core::{
    config::SanitizedConfig, vault::SanitizedVault, vault_operator_ticket::VaultOperatorTicket,
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

/// Instruction: [`crate::VaultInstruction::AddOperator`]
pub fn process_vault_add_operator(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let SanitizedAccounts {
        config,
        mut vault,
        operator,
        operator_vault_ticket,
        vault_operator_ticket_account,
        admin,
        payer,
        system_program,
    } = SanitizedAccounts::sanitize(program_id, accounts)?;

    vault.vault().check_operator_admin(admin.account().key)?;

    let slot = Clock::get()?.slot;
    // The operator shall support the vault for it to be added
    operator_vault_ticket
        .operator_vault_ticket()
        .check_active(slot, config.config().epoch_length())?;

    _create_vault_operator_ticket(
        program_id,
        &vault,
        &operator,
        &vault_operator_ticket_account,
        &payer,
        &system_program,
        &Rent::get()?,
        slot,
    )?;

    vault.vault_mut().increment_operator_count()?;

    vault.save()?;

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn _create_vault_operator_ticket<'a, 'info>(
    program_id: &Pubkey,
    vault: &SanitizedVault<'a, 'info>,
    operator: &SanitizedOperator<'a, 'info>,
    vault_operator_ticket_account: &EmptyAccount<'a, 'info>,
    payer: &SanitizedSignerAccount<'a, 'info>,
    system_program: &SanitizedSystemProgram<'a, 'info>,
    rent: &Rent,
    slot: u64,
) -> ProgramResult {
    let (address, bump, mut seeds) = VaultOperatorTicket::find_program_address(
        program_id,
        vault.account().key,
        operator.account().key,
    );
    seeds.push(vec![bump]);

    assert_with_msg(
        address == *vault_operator_ticket_account.account().key,
        ProgramError::InvalidAccountData,
        "Vault operator ticket is not at the correct PDA",
    )?;

    let vault_operator_ticket = VaultOperatorTicket::new(
        *vault.account().key,
        *operator.account().key,
        vault.vault().operator_count(),
        slot,
        bump,
    );

    msg!(
        "Creating vault operator ticket: {:?}",
        vault_operator_ticket_account.account().key
    );
    let serialized = vault_operator_ticket.try_to_vec()?;
    create_account(
        payer.account(),
        vault_operator_ticket_account.account(),
        system_program.account(),
        program_id,
        rent,
        serialized.len() as u64,
        &seeds,
    )?;
    vault_operator_ticket_account.account().data.borrow_mut()[..serialized.len()]
        .copy_from_slice(&serialized);
    Ok(())
}

struct SanitizedAccounts<'a, 'info> {
    config: SanitizedConfig<'a, 'info>,
    vault: SanitizedVault<'a, 'info>,
    operator: SanitizedOperator<'a, 'info>,
    operator_vault_ticket: SanitizedOperatorVaultTicket<'a, 'info>,
    vault_operator_ticket_account: EmptyAccount<'a, 'info>,
    admin: SanitizedSignerAccount<'a, 'info>,
    payer: SanitizedSignerAccount<'a, 'info>,
    system_program: SanitizedSystemProgram<'a, 'info>,
}

impl<'a, 'info> SanitizedAccounts<'a, 'info> {
    /// Sanitizes the accounts for instruction: [`crate::VaultInstruction::AddOperator`]
    fn sanitize(
        program_id: &Pubkey,
        accounts: &'a [AccountInfo<'info>],
    ) -> Result<SanitizedAccounts<'a, 'info>, ProgramError> {
        let mut accounts_iter = accounts.iter();

        let config =
            SanitizedConfig::sanitize(program_id, next_account_info(&mut accounts_iter)?, false)?;
        let vault =
            SanitizedVault::sanitize(program_id, next_account_info(&mut accounts_iter)?, false)?;
        let operator = SanitizedOperator::sanitize(
            &config.config().restaking_program(),
            next_account_info(&mut accounts_iter)?,
            false,
        )?;

        let operator_vault_ticket = SanitizedOperatorVaultTicket::sanitize(
            &config.config().restaking_program(),
            next_account_info(&mut accounts_iter)?,
            false,
            operator.account().key,
            vault.account().key,
        )?;
        let vault_operator_ticket_account =
            EmptyAccount::sanitize(next_account_info(&mut accounts_iter)?, true)?;
        let admin =
            SanitizedSignerAccount::sanitize(next_account_info(&mut accounts_iter)?, false)?;
        let payer = SanitizedSignerAccount::sanitize(next_account_info(&mut accounts_iter)?, true)?;
        let system_program =
            SanitizedSystemProgram::sanitize(next_account_info(&mut accounts_iter)?)?;

        Ok(SanitizedAccounts {
            config,
            vault,
            operator,
            operator_vault_ticket,
            vault_operator_ticket_account,
            admin,
            payer,
            system_program,
        })
    }
}
