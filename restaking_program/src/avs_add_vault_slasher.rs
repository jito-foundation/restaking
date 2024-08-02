use borsh::BorshSerialize;
use jito_restaking_core::{
    avs::SanitizedAvs, avs_vault_slasher_ticket::AvsVaultSlasherTicket,
    avs_vault_ticket::SanitizedAvsVaultTicket, config::SanitizedConfig,
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

pub fn process_avs_add_vault_slasher(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    max_slashable_per_epoch: u64,
) -> ProgramResult {
    let SanitizedAccounts {
        config,
        mut avs,
        vault,
        slasher,
        avs_vault_ticket,
        avs_vault_slasher_ticket_account,
        admin,
        payer,
        system_program,
    } = SanitizedAccounts::sanitize(program_id, accounts)?;

    avs.avs().check_slasher_admin(admin.account().key)?;

    let slot = Clock::get()?.slot;
    avs_vault_ticket
        .avs_vault_ticket()
        .check_active(slot, config.config().epoch_length())?;

    _create_avs_vault_slasher_ticket(
        program_id,
        &avs,
        vault,
        slasher,
        &avs_vault_slasher_ticket_account,
        &payer,
        &system_program,
        &Rent::get()?,
        slot,
        max_slashable_per_epoch,
    )?;

    avs.avs_mut().increment_slasher_count()?;

    avs.save()?;

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn _create_avs_vault_slasher_ticket<'a, 'info>(
    program_id: &Pubkey,
    avs: &SanitizedAvs<'a, 'info>,
    vault: &AccountInfo<'info>,
    slasher: &AccountInfo<'info>,
    avs_vault_slasher_ticket_account: &EmptyAccount<'a, 'info>,
    payer: &SanitizedSignerAccount<'a, 'info>,
    system_program: &SanitizedSystemProgram<'a, 'info>,
    rent: &Rent,
    slot: u64,
    max_slashable_per_epoch: u64,
) -> ProgramResult {
    let (address, bump, mut seeds) = AvsVaultSlasherTicket::find_program_address(
        program_id,
        avs.account().key,
        vault.key,
        slasher.key,
    );
    seeds.push(vec![bump]);

    assert_with_msg(
        address == *avs_vault_slasher_ticket_account.account().key,
        ProgramError::InvalidAccountData,
        "Invalid AVS vault slasher ticket PDA",
    )?;

    let avs_vault_slasher_ticket = AvsVaultSlasherTicket::new(
        *avs.account().key,
        *vault.key,
        *slasher.key,
        max_slashable_per_epoch,
        avs.avs().slasher_count(),
        slot,
        bump,
    );

    msg!(
        "Creating AVS vault slasher ticket: {:?}",
        avs_vault_slasher_ticket_account.account().key
    );
    let serialized = avs_vault_slasher_ticket.try_to_vec()?;
    create_account(
        payer.account(),
        avs_vault_slasher_ticket_account.account(),
        system_program.account(),
        program_id,
        rent,
        serialized.len() as u64,
        &seeds,
    )?;
    avs_vault_slasher_ticket_account.account().data.borrow_mut()[..serialized.len()]
        .copy_from_slice(&serialized);
    Ok(())
}

struct SanitizedAccounts<'a, 'info> {
    config: SanitizedConfig<'a, 'info>,
    avs: SanitizedAvs<'a, 'info>,
    vault: &'a AccountInfo<'info>,
    slasher: &'a AccountInfo<'info>,
    avs_vault_ticket: SanitizedAvsVaultTicket<'a, 'info>,
    avs_vault_slasher_ticket_account: EmptyAccount<'a, 'info>,
    admin: SanitizedSignerAccount<'a, 'info>,
    payer: SanitizedSignerAccount<'a, 'info>,
    system_program: SanitizedSystemProgram<'a, 'info>,
}

impl<'a, 'info> SanitizedAccounts<'a, 'info> {
    /// Sanitizes the accounts for the instruction: [`crate::RestakingInstruction::AvsAddVaultSlasher`]
    fn sanitize(
        program_id: &Pubkey,
        accounts: &'a [AccountInfo<'info>],
    ) -> Result<SanitizedAccounts<'a, 'info>, ProgramError> {
        let accounts_iter = &mut accounts.iter();

        let config =
            SanitizedConfig::sanitize(program_id, next_account_info(accounts_iter)?, false)?;
        let avs = SanitizedAvs::sanitize(program_id, next_account_info(accounts_iter)?, false)?;
        let vault = next_account_info(accounts_iter)?;
        let slasher = next_account_info(accounts_iter)?;
        let avs_vault_ticket = SanitizedAvsVaultTicket::sanitize(
            program_id,
            next_account_info(accounts_iter)?,
            false,
            avs.account().key,
            vault.key,
        )?;
        let avs_vault_slasher_ticket_account =
            EmptyAccount::sanitize(next_account_info(accounts_iter)?, true)?;
        let admin = SanitizedSignerAccount::sanitize(next_account_info(accounts_iter)?, false)?;
        let payer = SanitizedSignerAccount::sanitize(next_account_info(accounts_iter)?, true)?;
        let system_program = SanitizedSystemProgram::sanitize(next_account_info(accounts_iter)?)?;

        Ok(SanitizedAccounts {
            config,
            avs,
            vault,
            slasher,
            avs_vault_ticket,
            avs_vault_slasher_ticket_account,
            admin,
            payer,
            system_program,
        })
    }
}
