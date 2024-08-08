use borsh::BorshSerialize;
use jito_restaking_core::{
    avs::SanitizedAvs, avs_vault_slasher_ticket::SanitizedAvsVaultSlasherTicket,
};
use jito_restaking_sanitization::{
    assert_with_msg, create_account, empty_account::EmptyAccount, signer::SanitizedSignerAccount,
    system_program::SanitizedSystemProgram,
};
use jito_vault_core::{
    config::SanitizedConfig, vault::SanitizedVault, vault_avs_slasher_ticket::VaultAvsSlasherTicket,
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

/// Processes the register slasher instruction: [`crate::VaultInstruction::AddSlasher`]
pub fn process_add_slasher(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let SanitizedAccounts {
        config,
        mut vault,
        avs,
        slasher,
        avs_slasher_ticket,
        vault_avs_slasher_ticket_account,
        admin,
        payer,
        system_program,
    } = SanitizedAccounts::sanitize(program_id, accounts)?;

    vault.vault().check_slasher_admin(admin.account().key)?;
    let slot = Clock::get()?.slot;

    avs_slasher_ticket
        .avs_vault_slasher_ticket()
        .check_active_or_cooldown(slot, config.config().epoch_length())?;

    let max_slashable_per_epoch = avs_slasher_ticket
        .avs_vault_slasher_ticket()
        .max_slashable_per_epoch();

    _create_vault_avs_slasher_ticket(
        program_id,
        &vault,
        &avs,
        slasher,
        &vault_avs_slasher_ticket_account,
        &payer,
        &system_program,
        &Rent::get()?,
        slot,
        max_slashable_per_epoch,
    )?;

    vault.vault_mut().increment_slasher_count()?;

    vault.save()?;

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn _create_vault_avs_slasher_ticket<'a, 'info>(
    program_id: &Pubkey,
    vault: &SanitizedVault<'a, 'info>,
    avs: &SanitizedAvs<'a, 'info>,
    slasher: &AccountInfo<'info>,
    vault_avs_slasher_ticket_account: &EmptyAccount<'a, 'info>,
    payer: &SanitizedSignerAccount<'a, 'info>,
    system_program: &SanitizedSystemProgram<'a, 'info>,
    rent: &Rent,
    slot: u64,
    max_slashable_per_epoch: u64,
) -> ProgramResult {
    let (address, bump, mut seeds) = VaultAvsSlasherTicket::find_program_address(
        program_id,
        vault.account().key,
        avs.account().key,
        slasher.key,
    );
    seeds.push(vec![bump]);

    assert_with_msg(
        address == *vault_avs_slasher_ticket_account.account().key,
        ProgramError::InvalidAccountData,
        "Vault AVS slasher ticket is not at the correct PDA",
    )?;

    let vault_avs_slasher_ticket = VaultAvsSlasherTicket::new(
        *vault.account().key,
        *avs.account().key,
        *slasher.key,
        max_slashable_per_epoch,
        vault.vault().slasher_count(),
        slot,
        bump,
    );

    msg!(
        "Creating vault AVS slasher ticket: {:?}",
        vault_avs_slasher_ticket_account.account().key
    );
    let serialized = vault_avs_slasher_ticket.try_to_vec()?;
    create_account(
        payer.account(),
        vault_avs_slasher_ticket_account.account(),
        system_program.account(),
        program_id,
        rent,
        serialized.len() as u64,
        &seeds,
    )?;
    vault_avs_slasher_ticket_account.account().data.borrow_mut()[..serialized.len()]
        .copy_from_slice(&serialized);
    Ok(())
}

struct SanitizedAccounts<'a, 'info> {
    config: SanitizedConfig<'a, 'info>,
    vault: SanitizedVault<'a, 'info>,
    avs: SanitizedAvs<'a, 'info>,
    slasher: &'a AccountInfo<'info>,
    avs_slasher_ticket: SanitizedAvsVaultSlasherTicket<'a, 'info>,
    vault_avs_slasher_ticket_account: EmptyAccount<'a, 'info>,
    admin: SanitizedSignerAccount<'a, 'info>,
    payer: SanitizedSignerAccount<'a, 'info>,
    system_program: SanitizedSystemProgram<'a, 'info>,
}

impl<'a, 'info> SanitizedAccounts<'a, 'info> {
    fn sanitize(
        program_id: &Pubkey,
        accounts: &'a [AccountInfo<'info>],
    ) -> Result<SanitizedAccounts<'a, 'info>, ProgramError> {
        let account_iter = &mut accounts.iter();

        let config =
            SanitizedConfig::sanitize(program_id, next_account_info(account_iter)?, false)?;
        let vault = SanitizedVault::sanitize(program_id, next_account_info(account_iter)?, false)?;
        let avs = SanitizedAvs::sanitize(
            &config.config().restaking_program(),
            next_account_info(account_iter)?,
            false,
        )?;
        let slasher = next_account_info(account_iter)?;
        let avs_slasher_ticket = SanitizedAvsVaultSlasherTicket::sanitize(
            &config.config().restaking_program(),
            next_account_info(account_iter)?,
            false,
            avs.account().key,
            vault.account().key,
            slasher.key,
        )?;
        let vault_avs_slasher_ticket_account =
            EmptyAccount::sanitize(next_account_info(account_iter)?, true)?;
        let admin = SanitizedSignerAccount::sanitize(next_account_info(account_iter)?, false)?;
        let payer = SanitizedSignerAccount::sanitize(next_account_info(account_iter)?, true)?;
        let system_program = SanitizedSystemProgram::sanitize(next_account_info(account_iter)?)?;

        Ok(SanitizedAccounts {
            config,
            vault,
            avs,
            slasher,
            avs_slasher_ticket,
            vault_avs_slasher_ticket_account,
            admin,
            payer,
            system_program,
        })
    }
}
