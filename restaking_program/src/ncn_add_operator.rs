use borsh::BorshSerialize;
use jito_restaking_core::{
    config::SanitizedConfig, ncn::SanitizedNcn, ncn_operator_ticket::NcnOperatorTicket,
    operator::SanitizedOperator, operator_ncn_ticket::SanitizedOperatorNcnTicket,
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

/// After an operator opts-in to an NCN, the NCN operator admin can add the operator to the NCN.
/// The operator must have opted-in to the NCN before the NCN opts-in to the operator.
///
/// [`crate::RestakingInstruction::NcnAddOperator`]
pub fn process_ncn_add_operator(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let SanitizedAccounts {
        config,
        mut ncn,
        operator,
        ncn_operator_ticket_account,
        operator_ncn_ticket,
        admin,
        payer,
        system_program,
    } = SanitizedAccounts::sanitize(program_id, accounts)?;

    ncn.ncn().check_operator_admin(admin.account().key)?;

    let slot = Clock::get()?.slot;

    operator_ncn_ticket
        .operator_ncn_ticket()
        .check_active_or_cooldown(slot, config.config().epoch_length())?;

    _create_ncn_operator_ticket(
        program_id,
        &ncn,
        &operator,
        &ncn_operator_ticket_account,
        &payer,
        &system_program,
        &Rent::get()?,
        slot,
    )?;

    ncn.ncn_mut().increment_operator_count()?;

    ncn.save()?;

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn _create_ncn_operator_ticket<'a, 'info>(
    program_id: &Pubkey,
    ncn: &SanitizedNcn<'a, 'info>,
    operator: &SanitizedOperator<'a, 'info>,
    ncn_operator_ticket_account: &EmptyAccount<'a, 'info>,
    payer: &SanitizedSignerAccount<'a, 'info>,
    system_program: &SanitizedSystemProgram<'a, 'info>,
    rent: &Rent,
    slot: u64,
) -> ProgramResult {
    let (address, bump, mut seeds) = NcnOperatorTicket::find_program_address(
        program_id,
        ncn.account().key,
        operator.account().key,
    );
    seeds.push(vec![bump]);

    assert_with_msg(
        address == *ncn_operator_ticket_account.account().key,
        ProgramError::InvalidAccountData,
        "NCN operator ticket is not at the correct PDA",
    )?;

    let ncn_operator_ticket = NcnOperatorTicket::new(
        *ncn.account().key,
        *operator.account().key,
        ncn.ncn().operator_count(),
        slot,
        bump,
    );

    msg!(
        "Creating NCN operator ticket: {:?}",
        ncn_operator_ticket_account.account().key
    );
    let serialized = ncn_operator_ticket.try_to_vec()?;
    create_account(
        payer.account(),
        ncn_operator_ticket_account.account(),
        system_program.account(),
        program_id,
        rent,
        serialized.len() as u64,
        &seeds,
    )?;
    ncn_operator_ticket_account.account().data.borrow_mut()[..serialized.len()]
        .copy_from_slice(&serialized);
    Ok(())
}

struct SanitizedAccounts<'a, 'info> {
    config: SanitizedConfig<'a, 'info>,
    ncn: SanitizedNcn<'a, 'info>,
    operator: SanitizedOperator<'a, 'info>,
    ncn_operator_ticket_account: EmptyAccount<'a, 'info>,
    operator_ncn_ticket: SanitizedOperatorNcnTicket<'a, 'info>,
    admin: SanitizedSignerAccount<'a, 'info>,
    payer: SanitizedSignerAccount<'a, 'info>,
    system_program: SanitizedSystemProgram<'a, 'info>,
}

impl<'a, 'info> SanitizedAccounts<'a, 'info> {
    /// [`crate::RestakingInstruction::NcnAddOperator`]
    fn sanitize(
        program_id: &Pubkey,
        accounts: &'a [AccountInfo<'info>],
    ) -> Result<SanitizedAccounts<'a, 'info>, ProgramError> {
        let accounts_iter = &mut accounts.iter();

        let config =
            SanitizedConfig::sanitize(program_id, next_account_info(accounts_iter)?, false)?;
        let ncn = SanitizedNcn::sanitize(program_id, next_account_info(accounts_iter)?, true)?;
        let operator =
            SanitizedOperator::sanitize(program_id, next_account_info(accounts_iter)?, false)?;
        let ncn_operator_ticket_account =
            EmptyAccount::sanitize(next_account_info(accounts_iter)?, true)?;
        let operator_ncn_ticket = SanitizedOperatorNcnTicket::sanitize(
            program_id,
            next_account_info(accounts_iter)?,
            false,
            operator.account().key,
            ncn.account().key,
        )?;
        let admin = SanitizedSignerAccount::sanitize(next_account_info(accounts_iter)?, false)?;
        let payer = SanitizedSignerAccount::sanitize(next_account_info(accounts_iter)?, true)?;
        let system_program = SanitizedSystemProgram::sanitize(next_account_info(accounts_iter)?)?;

        Ok(SanitizedAccounts {
            config,
            ncn,
            operator,
            ncn_operator_ticket_account,
            operator_ncn_ticket,
            admin,
            payer,
            system_program,
        })
    }
}
