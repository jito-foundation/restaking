use jito_restaking_core::{
    avs::SanitizedAvs, avs_vault_list::SanitizedAvsVaultList, config::SanitizedConfig,
};
use jito_restaking_sanitization::signer::SanitizedSignerAccount;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::Clock,
    entrypoint::ProgramResult,
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvar::Sysvar,
};

/// [`crate::RestakingInstruction::AvsRemoveVault`]
pub fn process_avs_remove_vault(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let SanitizedAccounts {
        avs,
        mut avs_vault_list,
        admin,
        vault,
    } = SanitizedAccounts::sanitize(program_id, accounts)?;

    avs.avs().check_vault_admin(admin.account().key)?;

    let slot = Clock::get()?.slot;
    avs_vault_list
        .avs_vault_list_mut()
        .remove_vault(*vault.key, slot)?;

    avs_vault_list.save()?;

    Ok(())
}

struct SanitizedAccounts<'a, 'info> {
    // config: SanitizedConfig<'a, 'info>,
    avs: SanitizedAvs<'a, 'info>,
    avs_vault_list: SanitizedAvsVaultList<'a, 'info>,
    admin: SanitizedSignerAccount<'a, 'info>,
    vault: &'a AccountInfo<'info>,
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

        Ok(SanitizedAccounts {
            // config,
            avs,
            avs_vault_list,
            admin,
            vault,
        })
    }
}
