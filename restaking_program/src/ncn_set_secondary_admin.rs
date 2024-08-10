use jito_restaking_core::ncn::SanitizedNcn;
use jito_restaking_sanitization::signer::SanitizedSignerAccount;
use jito_restaking_sdk::NcnAdminRole;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program_error::ProgramError,
    pubkey::Pubkey,
};

pub fn process_ncn_set_secondary_admin(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    role: NcnAdminRole,
) -> ProgramResult {
    let SanitizedAccounts {
        mut ncn,
        admin,
        new_admin,
    } = SanitizedAccounts::sanitize(program_id, accounts)?;

    ncn.ncn().check_admin(admin.account().key)?;

    match role {
        NcnAdminRole::Operator => {
            ncn.ncn_mut().set_operator_admin(*new_admin.account().key);
        }
        NcnAdminRole::Vault => {
            ncn.ncn_mut().set_vault_admin(*new_admin.account().key);
        }
        NcnAdminRole::Slasher => {
            ncn.ncn_mut().set_slasher_admin(*new_admin.account().key);
        }
        NcnAdminRole::Withdraw => {
            ncn.ncn_mut().set_withdraw_admin(*new_admin.account().key);
        }
    }

    ncn.save()?;

    Ok(())
}

struct SanitizedAccounts<'a, 'info> {
    ncn: SanitizedNcn<'a, 'info>,
    admin: SanitizedSignerAccount<'a, 'info>,
    new_admin: SanitizedSignerAccount<'a, 'info>,
}

impl<'a, 'info> SanitizedAccounts<'a, 'info> {
    fn sanitize(
        program_id: &Pubkey,
        accounts: &'a [AccountInfo<'info>],
    ) -> Result<Self, ProgramError> {
        let mut accounts_iter = accounts.iter();

        let ncn = SanitizedNcn::sanitize(program_id, next_account_info(&mut accounts_iter)?, true)?;
        let admin =
            SanitizedSignerAccount::sanitize(next_account_info(&mut accounts_iter)?, false)?;
        let new_admin =
            SanitizedSignerAccount::sanitize(next_account_info(&mut accounts_iter)?, false)?;

        Ok(Self {
            ncn,
            admin,
            new_admin,
        })
    }
}
