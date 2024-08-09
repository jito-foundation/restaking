use borsh::BorshSerialize;
use jito_restaking_core::{ncn::SanitizedNcn, ncn_vault_ticket::SanitizedNcnVaultTicket};
use jito_restaking_sanitization::{
    assert_with_msg, create_account, empty_account::EmptyAccount, signer::SanitizedSignerAccount,
    system_program::SanitizedSystemProgram,
};
use jito_vault_core::{
    config::SanitizedConfig, vault::SanitizedVault, vault_ncn_ticket::VaultNcnTicket,
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

/// Adds an NCN to the vault NCN list, which means delegation applied to operators staking to the NCN
/// will be applied.
///
/// # Behavior
/// * The vault admin shall have the ability to add support for a new NCN
/// if the NCN is actively supporting the vault
///
/// Instruction: [`crate::VaultInstruction::AddNcn`]
pub fn process_vault_add_ncn(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let SanitizedAccounts {
        config,
        mut vault,
        ncn,
        ncn_vault_ticket,
        vault_ncn_ticket,
        admin,
        payer,
        system_program,
    } = SanitizedAccounts::sanitize(program_id, accounts)?;

    vault.vault().check_ncn_admin(admin.account().key)?;

    let slot = Clock::get()?.slot;
    ncn_vault_ticket
        .ncn_vault_ticket()
        .check_active_or_cooldown(slot, config.config().epoch_length())?;

    _create_vault_ncn_ticket(
        program_id,
        &vault,
        &ncn,
        &vault_ncn_ticket,
        &payer,
        &system_program,
        &Rent::get()?,
        slot,
    )?;

    vault.vault_mut().increment_ncn_count()?;

    vault.save()?;

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn _create_vault_ncn_ticket<'a, 'info>(
    program_id: &Pubkey,
    vault: &SanitizedVault<'a, 'info>,
    ncn: &SanitizedNcn<'a, 'info>,
    vault_ncn_ticket_account: &EmptyAccount<'a, 'info>,
    payer: &SanitizedSignerAccount<'a, 'info>,
    system_program: &SanitizedSystemProgram<'a, 'info>,
    rent: &Rent,
    slot: u64,
) -> ProgramResult {
    let (address, bump, mut seeds) =
        VaultNcnTicket::find_program_address(program_id, vault.account().key, ncn.account().key);
    seeds.push(vec![bump]);

    assert_with_msg(
        address == *vault_ncn_ticket_account.account().key,
        ProgramError::InvalidAccountData,
        "Vault NCN ticket is not at the correct PDA",
    )?;

    let vault_ncn_ticket = VaultNcnTicket::new(
        *vault.account().key,
        *ncn.account().key,
        vault.vault().ncn_count(),
        slot,
        bump,
    );

    let serialized = vault_ncn_ticket.try_to_vec()?;
    msg!(
        "Creating vault NCN ticket: {:?} with space: {}",
        vault_ncn_ticket_account.account().key,
        serialized.len()
    );
    create_account(
        payer.account(),
        vault_ncn_ticket_account.account(),
        system_program.account(),
        program_id,
        rent,
        serialized.len() as u64,
        &seeds,
    )?;
    vault_ncn_ticket_account.account().data.borrow_mut()[..serialized.len()]
        .copy_from_slice(&serialized);
    Ok(())
}

struct SanitizedAccounts<'a, 'info> {
    config: SanitizedConfig<'a, 'info>,
    vault: SanitizedVault<'a, 'info>,
    ncn: SanitizedNcn<'a, 'info>,
    ncn_vault_ticket: SanitizedNcnVaultTicket<'a, 'info>,
    vault_ncn_ticket: EmptyAccount<'a, 'info>,
    admin: SanitizedSignerAccount<'a, 'info>,
    payer: SanitizedSignerAccount<'a, 'info>,
    system_program: SanitizedSystemProgram<'a, 'info>,
}

impl<'a, 'info> SanitizedAccounts<'a, 'info> {
    /// Sanitizes the accounts for the instruction: [`crate::VaultInstruction::AddNcn`]
    fn sanitize(
        program_id: &Pubkey,
        accounts: &'a [AccountInfo<'info>],
    ) -> Result<SanitizedAccounts<'a, 'info>, ProgramError> {
        let mut accounts_iter = accounts.iter();

        let config =
            SanitizedConfig::sanitize(program_id, next_account_info(&mut accounts_iter)?, false)?;
        let vault =
            SanitizedVault::sanitize(program_id, next_account_info(&mut accounts_iter)?, true)?;
        let ncn = SanitizedNcn::sanitize(
            &config.config().restaking_program(),
            next_account_info(&mut accounts_iter)?,
            false,
        )?;
        let ncn_vault_ticket = SanitizedNcnVaultTicket::sanitize(
            &config.config().restaking_program(),
            next_account_info(&mut accounts_iter)?,
            false,
            ncn.account().key,
            vault.account().key,
        )?;
        let vault_ncn_ticket =
            EmptyAccount::sanitize(next_account_info(&mut accounts_iter)?, true)?;
        let admin =
            SanitizedSignerAccount::sanitize(next_account_info(&mut accounts_iter)?, false)?;
        let payer = SanitizedSignerAccount::sanitize(next_account_info(&mut accounts_iter)?, true)?;
        let system_program =
            SanitizedSystemProgram::sanitize(next_account_info(&mut accounts_iter)?)?;

        Ok(SanitizedAccounts {
            config,
            vault,
            ncn,
            ncn_vault_ticket,
            vault_ncn_ticket,
            admin,
            payer,
            system_program,
        })
    }
}
