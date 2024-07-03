use jito_restaking_core::avs::SanitizedAvs;
use jito_restaking_sanitization::signer::SanitizedSignerAccount;
use jito_restaking_sdk::AvsAdminRole;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program_error::ProgramError,
    pubkey::Pubkey,
};

pub fn process_avs_set_secondary_admin(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    role: AvsAdminRole,
) -> ProgramResult {
    let SanitizedAccounts {
        mut avs,
        admin,
        new_admin,
    } = SanitizedAccounts::sanitize(program_id, accounts)?;

    avs.avs().check_admin(admin.account().key)?;

    match role {
        AvsAdminRole::Operator => {
            avs.avs_mut().set_operator_admin(*new_admin.account().key);
        }
        AvsAdminRole::Vault => {
            avs.avs_mut().set_vault_admin(*new_admin.account().key);
        }
        AvsAdminRole::Slasher => {
            avs.avs_mut().set_slasher_admin(*new_admin.account().key);
        }
        AvsAdminRole::Withdraw => {
            avs.avs_mut().set_withdraw_admin(*new_admin.account().key);
        }
    }

    avs.save()?;

    Ok(())
}

struct SanitizedAccounts<'a, 'info> {
    avs: SanitizedAvs<'a, 'info>,
    admin: SanitizedSignerAccount<'a, 'info>,
    new_admin: SanitizedSignerAccount<'a, 'info>,
}

impl<'a, 'info> SanitizedAccounts<'a, 'info> {
    fn sanitize(
        program_id: &Pubkey,
        accounts: &'a [AccountInfo<'info>],
    ) -> Result<Self, ProgramError> {
        let mut accounts_iter = accounts.iter();

        let avs = SanitizedAvs::sanitize(program_id, next_account_info(&mut accounts_iter)?, true)?;
        let admin =
            SanitizedSignerAccount::sanitize(next_account_info(&mut accounts_iter)?, false)?;
        let new_admin =
            SanitizedSignerAccount::sanitize(next_account_info(&mut accounts_iter)?, false)?;

        Ok(Self {
            avs,
            admin,
            new_admin,
        })
    }
}
