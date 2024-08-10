use borsh::BorshSerialize;
use jito_restaking_core::{
    config::SanitizedConfig, ncn::SanitizedNcn, operator::SanitizedOperator,
    operator_ncn_ticket::OperatorNcnTicket,
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

/// The node operator admin can add support for running an NCN.
/// This method is permissioned to the node operator admin.
///
/// [`crate::RestakingInstruction::OperatorAddNcn`]
pub fn process_operator_add_ncn(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let SanitizedAccounts {
        mut operator,
        ncn,
        operator_ncn_ticket_account,
        admin,
        payer,
        system_program,
    } = SanitizedAccounts::sanitize(program_id, accounts)?;

    operator.operator.check_ncn_admin(admin.account().key)?;

    let slot = Clock::get()?.slot;
    let rent = Rent::get()?;
    _create_operator_ncn_ticket(
        program_id,
        &operator,
        &ncn,
        &operator_ncn_ticket_account,
        &payer,
        &system_program,
        &rent,
        slot,
    )?;

    operator.operator_mut().increment_ncn_count()?;

    operator.save()?;

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn _create_operator_ncn_ticket<'a, 'info>(
    program_id: &Pubkey,
    operator: &SanitizedOperator<'a, 'info>,
    ncn: &SanitizedNcn<'a, 'info>,
    operator_ncn_ticket_account: &EmptyAccount<'a, 'info>,
    payer: &SanitizedSignerAccount<'a, 'info>,
    system_program: &SanitizedSystemProgram<'a, 'info>,
    rent: &Rent,
    slot: u64,
) -> ProgramResult {
    let (address, bump, mut seeds) = OperatorNcnTicket::find_program_address(
        program_id,
        operator.account().key,
        ncn.account().key,
    );
    seeds.push(vec![bump]);

    assert_with_msg(
        address == *operator_ncn_ticket_account.account().key,
        ProgramError::InvalidAccountData,
        "Invalid operator NCN ticket PDA",
    )?;

    let operator_ncn_ticket = OperatorNcnTicket::new(
        *operator.account().key,
        *ncn.account().key,
        operator.operator.ncn_count(),
        slot,
        bump,
    );

    msg!(
        "Creating operator NCN ticket: {:?}",
        operator_ncn_ticket_account.account().key
    );
    let serialized = operator_ncn_ticket.try_to_vec()?;
    create_account(
        payer.account(),
        operator_ncn_ticket_account.account(),
        system_program.account(),
        program_id,
        rent,
        serialized.len() as u64,
        &seeds,
    )?;
    operator_ncn_ticket_account.account().data.borrow_mut()[..serialized.len()]
        .copy_from_slice(&serialized);
    Ok(())
}

struct SanitizedAccounts<'a, 'info> {
    operator: SanitizedOperator<'a, 'info>,
    ncn: SanitizedNcn<'a, 'info>,
    operator_ncn_ticket_account: EmptyAccount<'a, 'info>,
    admin: SanitizedSignerAccount<'a, 'info>,
    payer: SanitizedSignerAccount<'a, 'info>,
    system_program: SanitizedSystemProgram<'a, 'info>,
}

impl<'a, 'info> SanitizedAccounts<'a, 'info> {
    /// Sanitizes the accounts for the instruction: [`crate::RestakingInstruction::OperatorAddNcn`]
    fn sanitize(
        program_id: &Pubkey,
        accounts: &'a [AccountInfo<'info>],
    ) -> Result<SanitizedAccounts<'a, 'info>, ProgramError> {
        let mut accounts_iter = accounts.iter();

        let _config =
            SanitizedConfig::sanitize(program_id, next_account_info(&mut accounts_iter)?, false)?;
        let operator =
            SanitizedOperator::sanitize(program_id, next_account_info(&mut accounts_iter)?, true)?;
        let ncn =
            SanitizedNcn::sanitize(program_id, next_account_info(&mut accounts_iter)?, false)?;
        let operator_ncn_ticket_account =
            EmptyAccount::sanitize(next_account_info(&mut accounts_iter)?, true)?;
        let admin =
            SanitizedSignerAccount::sanitize(next_account_info(&mut accounts_iter)?, false)?;
        let payer = SanitizedSignerAccount::sanitize(next_account_info(&mut accounts_iter)?, true)?;
        let system_program =
            SanitizedSystemProgram::sanitize(next_account_info(&mut accounts_iter)?)?;

        Ok(SanitizedAccounts {
            operator,
            ncn,
            operator_ncn_ticket_account,
            admin,
            payer,
            system_program,
        })
    }
}
