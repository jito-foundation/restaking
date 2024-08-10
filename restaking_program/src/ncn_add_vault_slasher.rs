use borsh::BorshSerialize;
use jito_restaking_core::{
    config::SanitizedConfig, ncn::SanitizedNcn, ncn_vault_slasher_ticket::NcnVaultSlasherTicket,
    ncn_vault_ticket::SanitizedNcnVaultTicket,
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

pub fn process_ncn_add_vault_slasher(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    max_slashable_per_epoch: u64,
) -> ProgramResult {
    let SanitizedAccounts {
        config,
        mut ncn,
        vault,
        slasher,
        ncn_vault_ticket,
        ncn_vault_slasher_ticket_account,
        admin,
        payer,
        system_program,
    } = SanitizedAccounts::sanitize(program_id, accounts)?;

    ncn.ncn().check_slasher_admin(admin.account().key)?;

    let slot = Clock::get()?.slot;
    ncn_vault_ticket
        .ncn_vault_ticket()
        .check_active_or_cooldown(slot, config.config().epoch_length())?;

    _create_ncn_vault_slasher_ticket(
        program_id,
        &ncn,
        vault,
        slasher,
        &ncn_vault_slasher_ticket_account,
        &payer,
        &system_program,
        &Rent::get()?,
        slot,
        max_slashable_per_epoch,
    )?;

    ncn.ncn_mut().increment_slasher_count()?;

    ncn.save()?;

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn _create_ncn_vault_slasher_ticket<'a, 'info>(
    program_id: &Pubkey,
    ncn: &SanitizedNcn<'a, 'info>,
    vault: &AccountInfo<'info>,
    slasher: &AccountInfo<'info>,
    ncn_vault_slasher_ticket_account: &EmptyAccount<'a, 'info>,
    payer: &SanitizedSignerAccount<'a, 'info>,
    system_program: &SanitizedSystemProgram<'a, 'info>,
    rent: &Rent,
    slot: u64,
    max_slashable_per_epoch: u64,
) -> ProgramResult {
    let (address, bump, mut seeds) = NcnVaultSlasherTicket::find_program_address(
        program_id,
        ncn.account().key,
        vault.key,
        slasher.key,
    );
    seeds.push(vec![bump]);

    assert_with_msg(
        address == *ncn_vault_slasher_ticket_account.account().key,
        ProgramError::InvalidAccountData,
        "Invalid NCN vault slasher ticket PDA",
    )?;

    let ncn_vault_slasher_ticket = NcnVaultSlasherTicket::new(
        *ncn.account().key,
        *vault.key,
        *slasher.key,
        max_slashable_per_epoch,
        ncn.ncn().slasher_count(),
        slot,
        bump,
    );

    msg!(
        "Creating NCN vault slasher ticket: {:?}",
        ncn_vault_slasher_ticket_account.account().key
    );
    let serialized = ncn_vault_slasher_ticket.try_to_vec()?;
    create_account(
        payer.account(),
        ncn_vault_slasher_ticket_account.account(),
        system_program.account(),
        program_id,
        rent,
        serialized.len() as u64,
        &seeds,
    )?;
    ncn_vault_slasher_ticket_account.account().data.borrow_mut()[..serialized.len()]
        .copy_from_slice(&serialized);
    Ok(())
}

struct SanitizedAccounts<'a, 'info> {
    config: SanitizedConfig<'a, 'info>,
    ncn: SanitizedNcn<'a, 'info>,
    vault: &'a AccountInfo<'info>,
    slasher: &'a AccountInfo<'info>,
    ncn_vault_ticket: SanitizedNcnVaultTicket<'a, 'info>,
    ncn_vault_slasher_ticket_account: EmptyAccount<'a, 'info>,
    admin: SanitizedSignerAccount<'a, 'info>,
    payer: SanitizedSignerAccount<'a, 'info>,
    system_program: SanitizedSystemProgram<'a, 'info>,
}

impl<'a, 'info> SanitizedAccounts<'a, 'info> {
    /// Sanitizes the accounts for the instruction: [`crate::RestakingInstruction::ncnAddVaultSlasher`]
    fn sanitize(
        program_id: &Pubkey,
        accounts: &'a [AccountInfo<'info>],
    ) -> Result<SanitizedAccounts<'a, 'info>, ProgramError> {
        let accounts_iter = &mut accounts.iter();

        let config =
            SanitizedConfig::sanitize(program_id, next_account_info(accounts_iter)?, false)?;
        let ncn = SanitizedNcn::sanitize(program_id, next_account_info(accounts_iter)?, false)?;
        let vault = next_account_info(accounts_iter)?;
        let slasher = next_account_info(accounts_iter)?;
        let ncn_vault_ticket = SanitizedNcnVaultTicket::sanitize(
            program_id,
            next_account_info(accounts_iter)?,
            false,
            ncn.account().key,
            vault.key,
        )?;
        let ncn_vault_slasher_ticket_account =
            EmptyAccount::sanitize(next_account_info(accounts_iter)?, true)?;
        let admin = SanitizedSignerAccount::sanitize(next_account_info(accounts_iter)?, false)?;
        let payer = SanitizedSignerAccount::sanitize(next_account_info(accounts_iter)?, true)?;
        let system_program = SanitizedSystemProgram::sanitize(next_account_info(accounts_iter)?)?;

        Ok(SanitizedAccounts {
            config,
            ncn,
            vault,
            slasher,
            ncn_vault_ticket,
            ncn_vault_slasher_ticket_account,
            admin,
            payer,
            system_program,
        })
    }
}
