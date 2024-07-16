use borsh::BorshSerialize;
use jito_restaking_core::{avs::SanitizedAvs, avs_vault_ticket::SanitizedAvsVaultTicket};
use jito_restaking_sanitization::{
    assert_with_msg, create_account, empty_account::EmptyAccount, signer::SanitizedSignerAccount,
    system_program::SanitizedSystemProgram,
};
use jito_vault_core::{
    config::SanitizedConfig, vault::SanitizedVault, vault_avs_ticket::VaultAvsTicket,
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

/// Adds an AVS to the vault AVS list, which means delegation applied to operators staking to the AVS
/// will be applied.
///
/// # Behavior
/// * The vault admin shall have the ability to add support for a new AVS
/// if the AVS is actively supporting the vault
///
/// Instruction: [`crate::VaultInstruction::AddAvs`]
pub fn process_vault_add_avs(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let SanitizedAccounts {
        mut vault,
        avs,
        avs_vault_ticket,
        vault_avs_ticket,
        admin,
        payer,
        system_program,
    } = SanitizedAccounts::sanitize(program_id, accounts)?;

    vault.vault().check_avs_admin(admin.account().key)?;

    let slot = Clock::get()?.slot;
    avs_vault_ticket.avs_vault_ticket().check_active(slot)?;

    _create_vault_avs_ticket(
        program_id,
        &vault,
        &avs,
        &vault_avs_ticket,
        &payer,
        &system_program,
        &Rent::get()?,
        slot,
    )?;

    vault.vault_mut().increment_avs_count()?;

    vault.save()?;

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn _create_vault_avs_ticket<'a, 'info>(
    program_id: &Pubkey,
    vault: &SanitizedVault<'a, 'info>,
    avs: &SanitizedAvs<'a, 'info>,
    vault_avs_ticket_account: &EmptyAccount<'a, 'info>,
    payer: &SanitizedSignerAccount<'a, 'info>,
    system_program: &SanitizedSystemProgram<'a, 'info>,
    rent: &Rent,
    slot: u64,
) -> ProgramResult {
    let (address, bump, mut seeds) =
        VaultAvsTicket::find_program_address(program_id, vault.account().key, avs.account().key);
    seeds.push(vec![bump]);

    assert_with_msg(
        address == *vault_avs_ticket_account.account().key,
        ProgramError::InvalidAccountData,
        "Vault AVS ticket is not at the correct PDA",
    )?;

    let vault_avs_ticket = VaultAvsTicket::new(
        *vault.account().key,
        *avs.account().key,
        vault.vault().avs_count(),
        slot,
        bump,
    );

    let serialized = vault_avs_ticket.try_to_vec()?;
    msg!(
        "Creating vault AVS ticket: {:?} with space: {}",
        vault_avs_ticket_account.account().key,
        serialized.len()
    );
    create_account(
        payer.account(),
        vault_avs_ticket_account.account(),
        system_program.account(),
        program_id,
        rent,
        serialized.len() as u64,
        &seeds,
    )?;
    vault_avs_ticket_account.account().data.borrow_mut()[..serialized.len()]
        .copy_from_slice(&serialized);
    Ok(())
}

struct SanitizedAccounts<'a, 'info> {
    vault: SanitizedVault<'a, 'info>,
    avs: SanitizedAvs<'a, 'info>,
    avs_vault_ticket: SanitizedAvsVaultTicket<'a, 'info>,
    vault_avs_ticket: EmptyAccount<'a, 'info>,
    admin: SanitizedSignerAccount<'a, 'info>,
    payer: SanitizedSignerAccount<'a, 'info>,
    system_program: SanitizedSystemProgram<'a, 'info>,
}

impl<'a, 'info> SanitizedAccounts<'a, 'info> {
    /// Sanitizes the accounts for the instruction: [`crate::VaultInstruction::AddAvs`]
    fn sanitize(
        program_id: &Pubkey,
        accounts: &'a [AccountInfo<'info>],
    ) -> Result<SanitizedAccounts<'a, 'info>, ProgramError> {
        let mut accounts_iter = accounts.iter();

        let config =
            SanitizedConfig::sanitize(program_id, next_account_info(&mut accounts_iter)?, false)?;
        let vault =
            SanitizedVault::sanitize(program_id, next_account_info(&mut accounts_iter)?, true)?;
        let avs = SanitizedAvs::sanitize(
            &config.config().restaking_program(),
            next_account_info(&mut accounts_iter)?,
            false,
        )?;
        let avs_vault_ticket = SanitizedAvsVaultTicket::sanitize(
            &config.config().restaking_program(),
            next_account_info(&mut accounts_iter)?,
            false,
            avs.account().key,
            vault.account().key,
        )?;
        let vault_avs_ticket =
            EmptyAccount::sanitize(next_account_info(&mut accounts_iter)?, true)?;
        let admin =
            SanitizedSignerAccount::sanitize(next_account_info(&mut accounts_iter)?, false)?;
        let payer = SanitizedSignerAccount::sanitize(next_account_info(&mut accounts_iter)?, true)?;
        let system_program =
            SanitizedSystemProgram::sanitize(next_account_info(&mut accounts_iter)?)?;

        Ok(SanitizedAccounts {
            vault,
            avs,
            avs_vault_ticket,
            vault_avs_ticket,
            admin,
            payer,
            system_program,
        })
    }
}
