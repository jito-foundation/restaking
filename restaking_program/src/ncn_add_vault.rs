use borsh::BorshSerialize;
use jito_restaking_core::{
    config::SanitizedConfig, ncn::SanitizedNcn, ncn_vault_ticket::NcnVaultTicket,
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

/// The NCN opts-in to vaults by storing the vault in the NCN vault list. It also CPI's into
/// the vault program and adds the NCN to the vault's NCN list.
///
/// [`crate::RestakingInstruction::NcnAddVault`]
pub fn process_ncn_add_vault(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let SanitizedAccounts {
        mut ncn,
        vault,
        ncn_vault_ticket_account,
        admin,
        payer,
        system_program,
    } = SanitizedAccounts::sanitize(program_id, accounts)?;

    ncn.ncn().check_vault_admin(admin.account().key)?;

    let slot = Clock::get()?.slot;

    _create_ncn_vault_ticket(
        program_id,
        &ncn,
        vault,
        &ncn_vault_ticket_account,
        &payer,
        &system_program,
        &Rent::get()?,
        slot,
    )?;

    ncn.ncn_mut().increment_vault_count()?;

    ncn.save()?;

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn _create_ncn_vault_ticket<'a, 'info>(
    program_id: &Pubkey,
    ncn: &SanitizedNcn<'a, 'info>,
    vault: &AccountInfo<'info>,
    ncn_vault_ticket_account: &EmptyAccount<'a, 'info>,
    payer: &SanitizedSignerAccount<'a, 'info>,
    system_program: &SanitizedSystemProgram<'a, 'info>,
    rent: &Rent,
    slot: u64,
) -> ProgramResult {
    let (address, bump, mut seeds) =
        NcnVaultTicket::find_program_address(program_id, ncn.account().key, vault.key);
    seeds.push(vec![bump]);

    assert_with_msg(
        address == *ncn_vault_ticket_account.account().key,
        ProgramError::InvalidAccountData,
        "Invalid NCN vault ticket PDA",
    )?;

    let ncn_vault_ticket = NcnVaultTicket::new(
        *ncn.account().key,
        *vault.key,
        ncn.ncn().vault_count(),
        slot,
        bump,
    );

    msg!(
        "Creating NCN vault ticket: {:?}",
        ncn_vault_ticket_account.account().key
    );
    let serialized = ncn_vault_ticket.try_to_vec()?;
    create_account(
        payer.account(),
        ncn_vault_ticket_account.account(),
        system_program.account(),
        program_id,
        rent,
        serialized.len() as u64,
        &seeds,
    )?;
    ncn_vault_ticket_account.account().data.borrow_mut()[..serialized.len()]
        .copy_from_slice(&serialized);
    Ok(())
}

struct SanitizedAccounts<'a, 'info> {
    ncn: SanitizedNcn<'a, 'info>,
    vault: &'a AccountInfo<'info>,
    ncn_vault_ticket_account: EmptyAccount<'a, 'info>,
    admin: SanitizedSignerAccount<'a, 'info>,
    payer: SanitizedSignerAccount<'a, 'info>,
    system_program: SanitizedSystemProgram<'a, 'info>,
}

impl<'a, 'info> SanitizedAccounts<'a, 'info> {
    /// Sanitizes the accounts for the instruction: [`crate::RestakingInstruction::NcnAddVault`]
    pub fn sanitize(
        program_id: &Pubkey,
        accounts: &'a [AccountInfo<'info>],
    ) -> Result<SanitizedAccounts<'a, 'info>, ProgramError> {
        let accounts_iter = &mut accounts.iter();

        let _config =
            SanitizedConfig::sanitize(program_id, next_account_info(accounts_iter)?, false)?;
        let ncn = SanitizedNcn::sanitize(program_id, next_account_info(accounts_iter)?, true)?;
        // TODO (LB): should deser vault?
        let vault = next_account_info(accounts_iter)?;
        let ncn_vault_ticket_account =
            EmptyAccount::sanitize(next_account_info(accounts_iter)?, true)?;
        let admin = SanitizedSignerAccount::sanitize(next_account_info(accounts_iter)?, false)?;
        let payer = SanitizedSignerAccount::sanitize(next_account_info(accounts_iter)?, true)?;
        let system_program = SanitizedSystemProgram::sanitize(next_account_info(accounts_iter)?)?;

        Ok(SanitizedAccounts {
            ncn,
            vault,
            ncn_vault_ticket_account,
            admin,
            payer,
            system_program,
        })
    }
}
