use jito_restaking_core::{
    avs::SanitizedAvs, avs_vault_list::SanitizedAvsVaultList, config::SanitizedConfig,
};
use jito_restaking_sanitization::{
    signer::SanitizedSignerAccount, system_program::SanitizedSystemProgram,
};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::Clock,
    entrypoint::ProgramResult,
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
        avs,
        mut avs_vault_list,
        admin,
        vault,
        payer,
    } = SanitizedAccounts::sanitize(program_id, accounts)?;

    avs.avs().check_vault_admin(admin.account().key)?;

    let slot = Clock::get()?.slot;
    avs_vault_list
        .avs_vault_list_mut()
        .add_vault(*vault.key, slot)?;

    let rent = Rent::get()?;
    avs_vault_list.save_with_realloc(&rent, payer.account())?;

    Ok(())
}

struct SanitizedAccounts<'a, 'info> {
    // config: SanitizedConfig<'a, 'info>,
    avs: SanitizedAvs<'a, 'info>,
    avs_vault_list: SanitizedAvsVaultList<'a, 'info>,
    admin: SanitizedSignerAccount<'a, 'info>,
    vault: &'a AccountInfo<'info>,
    payer: SanitizedSignerAccount<'a, 'info>,
    // system_program: SanitizedSystemProgram<'a, 'info>,
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
        let avs = SanitizedAvs::sanitize(program_id, next_account_info(accounts_iter)?, false)?;
        let avs_vault_list = SanitizedAvsVaultList::sanitize(
            program_id,
            next_account_info(accounts_iter)?,
            true,
            avs.account().key,
        )?;
        let admin = SanitizedSignerAccount::sanitize(next_account_info(accounts_iter)?, false)?;

        // TODO (LB): should run more verification on the vault here?
        //  program owner? deserialize it/check header?
        let vault = next_account_info(accounts_iter)?;

        let payer = SanitizedSignerAccount::sanitize(next_account_info(accounts_iter)?, true)?;
        let _system_program = SanitizedSystemProgram::sanitize(next_account_info(accounts_iter)?)?;

        Ok(SanitizedAccounts {
            // config,
            avs,
            avs_vault_list,
            admin,
            vault,
            payer,
            // system_program,
        })
    }
}
