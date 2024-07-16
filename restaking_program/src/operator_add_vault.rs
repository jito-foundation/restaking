use borsh::BorshSerialize;
use jito_restaking_core::{
    config::SanitizedConfig, operator::SanitizedOperator,
    operator_vault_ticket::OperatorVaultTicket,
};
use jito_restaking_sanitization::{
    assert_with_msg, create_account, empty_account::EmptyAccount, signer::SanitizedSignerAccount,
    system_program::SanitizedSystemProgram,
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

/// The node operator admin can add support for receiving delegation from a vault.
/// The vault can be used at the end of epoch + 1.
/// This method is permissioned to the node operator admin.
///
/// [`crate::RestakingInstruction::OperatorAddVault`]
pub fn process_operator_add_vault(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let SanitizedAccounts {
        mut operator,
        vault,
        operator_vault_ticket_account,
        admin,
        payer,
        system_program,
    } = SanitizedAccounts::sanitize(program_id, accounts)?;

    operator.operator().check_vault_admin(admin.account().key)?;

    let slot = Clock::get()?.slot;
    let rent = Rent::get()?;

    _create_operator_vault_ticket(
        program_id,
        &operator,
        vault,
        &operator_vault_ticket_account,
        &payer,
        &system_program,
        &rent,
        slot,
    )?;

    operator.operator_mut().increment_vault_count()?;

    operator.save()?;

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn _create_operator_vault_ticket<'a, 'info>(
    program_id: &Pubkey,
    operator: &SanitizedOperator<'a, 'info>,
    vault: &AccountInfo<'info>,
    operator_vault_ticket_account: &EmptyAccount<'a, 'info>,
    payer: &SanitizedSignerAccount<'a, 'info>,
    system_program: &SanitizedSystemProgram<'a, 'info>,
    rent: &Rent,
    slot: u64,
) -> ProgramResult {
    let (address, bump, mut seeds) =
        OperatorVaultTicket::find_program_address(program_id, operator.account().key, vault.key);
    seeds.push(vec![bump]);

    assert_with_msg(
        address == *operator_vault_ticket_account.account().key,
        ProgramError::InvalidAccountData,
        "Invalid operator vault ticket PDA",
    )?;

    let operator_vault_ticket = OperatorVaultTicket::new(
        *operator.account().key,
        *vault.key,
        operator.operator().vault_count(),
        slot,
        bump,
    );

    msg!(
        "Creating operator vault ticket: {:?}",
        operator_vault_ticket_account.account().key
    );
    let serialized = operator_vault_ticket.try_to_vec()?;
    create_account(
        payer.account(),
        operator_vault_ticket_account.account(),
        system_program.account(),
        program_id,
        rent,
        serialized.len() as u64,
        &seeds,
    )?;
    operator_vault_ticket_account.account().data.borrow_mut()[..serialized.len()]
        .copy_from_slice(&serialized);
    Ok(())
}

struct SanitizedAccounts<'a, 'info> {
    operator: SanitizedOperator<'a, 'info>,
    vault: &'a AccountInfo<'info>,
    operator_vault_ticket_account: EmptyAccount<'a, 'info>,
    admin: SanitizedSignerAccount<'a, 'info>,
    payer: SanitizedSignerAccount<'a, 'info>,
    system_program: SanitizedSystemProgram<'a, 'info>,
}

impl<'a, 'info> SanitizedAccounts<'a, 'info> {
    /// Sanitizes the accounts for the instruction: [`crate::RestakingInstruction::OperatorAddVault`]
    pub fn sanitize(
        program_id: &Pubkey,
        accounts: &'a [AccountInfo<'info>],
    ) -> Result<SanitizedAccounts<'a, 'info>, ProgramError> {
        let accounts_iter = &mut accounts.iter();

        let _config =
            SanitizedConfig::sanitize(program_id, next_account_info(accounts_iter)?, false)?;
        let operator =
            SanitizedOperator::sanitize(program_id, next_account_info(accounts_iter)?, true)?;
        let vault = next_account_info(accounts_iter)?;
        let operator_vault_ticket_account =
            EmptyAccount::sanitize(next_account_info(accounts_iter)?, true)?;
        let admin = SanitizedSignerAccount::sanitize(next_account_info(accounts_iter)?, false)?;
        let payer = SanitizedSignerAccount::sanitize(next_account_info(accounts_iter)?, false)?;
        let system_program = SanitizedSystemProgram::sanitize(next_account_info(accounts_iter)?)?;

        Ok(SanitizedAccounts {
            operator,
            vault,
            operator_vault_ticket_account,
            admin,
            payer,
            system_program,
        })
    }
}
