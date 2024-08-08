use borsh::BorshSerialize;
use jito_restaking_core::{
    avs::SanitizedAvs, avs_operator_ticket::AvsOperatorTicket, config::SanitizedConfig,
    operator::SanitizedOperator, operator_avs_ticket::SanitizedOperatorAvsTicket,
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

/// After an operator opts-in to an AVS, the AVS operator admin can add the operator to the AVS.
/// The operator must have opted-in to the AVS before the AVS opts-in to the operator.
///
/// [`crate::RestakingInstruction::AvsAddOperator`]
pub fn process_avs_add_operator(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let SanitizedAccounts {
        config,
        mut avs,
        operator,
        avs_operator_ticket_account,
        operator_avs_ticket,
        admin,
        payer,
        system_program,
    } = SanitizedAccounts::sanitize(program_id, accounts)?;

    avs.avs().check_operator_admin(admin.account().key)?;

    let slot = Clock::get()?.slot;

    operator_avs_ticket
        .operator_avs_ticket()
        .check_active_or_cooldown(slot, config.config().epoch_length())?;

    _create_avs_operator_ticket(
        program_id,
        &avs,
        &operator,
        &avs_operator_ticket_account,
        &payer,
        &system_program,
        &Rent::get()?,
        slot,
    )?;

    avs.avs_mut().increment_operator_count()?;

    avs.save()?;

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn _create_avs_operator_ticket<'a, 'info>(
    program_id: &Pubkey,
    avs: &SanitizedAvs<'a, 'info>,
    operator: &SanitizedOperator<'a, 'info>,
    avs_operator_ticket_account: &EmptyAccount<'a, 'info>,
    payer: &SanitizedSignerAccount<'a, 'info>,
    system_program: &SanitizedSystemProgram<'a, 'info>,
    rent: &Rent,
    slot: u64,
) -> ProgramResult {
    let (address, bump, mut seeds) = AvsOperatorTicket::find_program_address(
        program_id,
        avs.account().key,
        operator.account().key,
    );
    seeds.push(vec![bump]);

    assert_with_msg(
        address == *avs_operator_ticket_account.account().key,
        ProgramError::InvalidAccountData,
        "AVS operator ticket is not at the correct PDA",
    )?;

    let avs_operator_ticket = AvsOperatorTicket::new(
        *avs.account().key,
        *operator.account().key,
        avs.avs().operator_count(),
        slot,
        bump,
    );

    msg!(
        "Creating AVS operator ticket: {:?}",
        avs_operator_ticket_account.account().key
    );
    let serialized = avs_operator_ticket.try_to_vec()?;
    create_account(
        payer.account(),
        avs_operator_ticket_account.account(),
        system_program.account(),
        program_id,
        rent,
        serialized.len() as u64,
        &seeds,
    )?;
    avs_operator_ticket_account.account().data.borrow_mut()[..serialized.len()]
        .copy_from_slice(&serialized);
    Ok(())
}

struct SanitizedAccounts<'a, 'info> {
    config: SanitizedConfig<'a, 'info>,
    avs: SanitizedAvs<'a, 'info>,
    operator: SanitizedOperator<'a, 'info>,
    avs_operator_ticket_account: EmptyAccount<'a, 'info>,
    operator_avs_ticket: SanitizedOperatorAvsTicket<'a, 'info>,
    admin: SanitizedSignerAccount<'a, 'info>,
    payer: SanitizedSignerAccount<'a, 'info>,
    system_program: SanitizedSystemProgram<'a, 'info>,
}

impl<'a, 'info> SanitizedAccounts<'a, 'info> {
    /// [`crate::RestakingInstruction::AvsAddOperator`]
    fn sanitize(
        program_id: &Pubkey,
        accounts: &'a [AccountInfo<'info>],
    ) -> Result<SanitizedAccounts<'a, 'info>, ProgramError> {
        let accounts_iter = &mut accounts.iter();

        let config =
            SanitizedConfig::sanitize(program_id, next_account_info(accounts_iter)?, false)?;
        let avs = SanitizedAvs::sanitize(program_id, next_account_info(accounts_iter)?, true)?;
        let operator =
            SanitizedOperator::sanitize(program_id, next_account_info(accounts_iter)?, false)?;
        let avs_operator_ticket_account =
            EmptyAccount::sanitize(next_account_info(accounts_iter)?, true)?;
        let operator_avs_ticket = SanitizedOperatorAvsTicket::sanitize(
            program_id,
            next_account_info(accounts_iter)?,
            false,
            operator.account().key,
            avs.account().key,
        )?;
        let admin = SanitizedSignerAccount::sanitize(next_account_info(accounts_iter)?, false)?;
        let payer = SanitizedSignerAccount::sanitize(next_account_info(accounts_iter)?, true)?;
        let system_program = SanitizedSystemProgram::sanitize(next_account_info(accounts_iter)?)?;

        Ok(SanitizedAccounts {
            config,
            avs,
            operator,
            avs_operator_ticket_account,
            operator_avs_ticket,
            admin,
            payer,
            system_program,
        })
    }
}
