use borsh::BorshSerialize;
use jito_restaking_core::{
    avs::SanitizedAvs, avs_vault_ticket::AvsVaultTicket, config::SanitizedConfig,
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

/// The AVS opts-in to vaults by storing the vault in the AVS vault list. It also CPI's into
/// the vault program and adds the AVS to the vault's AVS list.
///
/// [`crate::RestakingInstruction::AvsAddVault`]
pub fn process_avs_add_vault(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let SanitizedAccounts {
        mut avs,
        vault,
        avs_vault_ticket_account,
        admin,
        payer,
        system_program,
    } = SanitizedAccounts::sanitize(program_id, accounts)?;

    avs.avs().check_vault_admin(admin.account().key)?;

    let slot = Clock::get()?.slot;

    _create_avs_vault_ticket(
        program_id,
        &avs,
        vault,
        &avs_vault_ticket_account,
        &payer,
        &system_program,
        &Rent::get()?,
        slot,
    )?;

    avs.avs_mut().increment_vault_count()?;

    avs.save()?;

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn _create_avs_vault_ticket<'a, 'info>(
    program_id: &Pubkey,
    avs: &SanitizedAvs<'a, 'info>,
    vault: &AccountInfo<'info>,
    avs_vault_ticket_account: &EmptyAccount<'a, 'info>,
    payer: &SanitizedSignerAccount<'a, 'info>,
    system_program: &SanitizedSystemProgram<'a, 'info>,
    rent: &Rent,
    slot: u64,
) -> ProgramResult {
    let (address, bump, mut seeds) =
        AvsVaultTicket::find_program_address(program_id, avs.account().key, vault.key);
    seeds.push(vec![bump]);

    assert_with_msg(
        address == *avs_vault_ticket_account.account().key,
        ProgramError::InvalidAccountData,
        "Invalid AVS vault ticket PDA",
    )?;

    let avs_vault_ticket = AvsVaultTicket::new(
        *avs.account().key,
        *vault.key,
        avs.avs().vault_count(),
        slot,
        bump,
    );

    msg!(
        "Creating AVS vault ticket: {:?}",
        avs_vault_ticket_account.account().key
    );
    let serialized = avs_vault_ticket.try_to_vec()?;
    create_account(
        payer.account(),
        avs_vault_ticket_account.account(),
        system_program.account(),
        program_id,
        rent,
        serialized.len() as u64,
        &seeds,
    )?;
    avs_vault_ticket_account.account().data.borrow_mut()[..serialized.len()]
        .copy_from_slice(&serialized);
    Ok(())
}

struct SanitizedAccounts<'a, 'info> {
    avs: SanitizedAvs<'a, 'info>,
    vault: &'a AccountInfo<'info>,
    avs_vault_ticket_account: EmptyAccount<'a, 'info>,
    admin: SanitizedSignerAccount<'a, 'info>,
    payer: SanitizedSignerAccount<'a, 'info>,
    system_program: SanitizedSystemProgram<'a, 'info>,
}

impl<'a, 'info> SanitizedAccounts<'a, 'info> {
    /// Sanitizes the accounts for the instruction: [`crate::RestakingInstruction::AvsAddVault`]
    pub fn sanitize(
        program_id: &Pubkey,
        accounts: &'a [AccountInfo<'info>],
    ) -> Result<SanitizedAccounts<'a, 'info>, ProgramError> {
        let accounts_iter = &mut accounts.iter();

        let _config =
            SanitizedConfig::sanitize(program_id, next_account_info(accounts_iter)?, false)?;
        let avs = SanitizedAvs::sanitize(program_id, next_account_info(accounts_iter)?, true)?;
        // TODO (LB): should deser vault?
        let vault = next_account_info(accounts_iter)?;
        let avs_vault_ticket_account =
            EmptyAccount::sanitize(next_account_info(accounts_iter)?, true)?;
        let admin = SanitizedSignerAccount::sanitize(next_account_info(accounts_iter)?, false)?;
        let payer = SanitizedSignerAccount::sanitize(next_account_info(accounts_iter)?, true)?;
        let system_program = SanitizedSystemProgram::sanitize(next_account_info(accounts_iter)?)?;

        Ok(SanitizedAccounts {
            avs,
            vault,
            avs_vault_ticket_account,
            admin,
            payer,
            system_program,
        })
    }
}
