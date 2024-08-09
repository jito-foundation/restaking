use jito_restaking_core::ncn::SanitizedNcn;
use jito_restaking_sanitization::signer::SanitizedSignerAccount;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program_error::ProgramError,
    pubkey::Pubkey,
};

pub fn process_ncn_set_admin(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let SanitizedAccounts {
        mut ncn,
        old_admin,
        new_admin,
    } = SanitizedAccounts::sanitize(program_id, accounts)?;

    ncn.ncn().check_admin(old_admin.account().key)?;
    ncn.ncn_mut().set_admin(*new_admin.account().key);

    ncn.save()?;

    Ok(())
}

struct SanitizedAccounts<'a, 'info> {
    ncn: SanitizedNcn<'a, 'info>,
    old_admin: SanitizedSignerAccount<'a, 'info>,
    new_admin: SanitizedSignerAccount<'a, 'info>,
}

impl<'a, 'info> SanitizedAccounts<'a, 'info> {
    fn sanitize(
        program_id: &Pubkey,
        accounts: &'a [AccountInfo<'info>],
    ) -> Result<Self, ProgramError> {
        let mut accounts_iter = accounts.iter();

        let ncn = SanitizedNcn::sanitize(program_id, next_account_info(&mut accounts_iter)?, true)?;
        let old_admin =
            SanitizedSignerAccount::sanitize(next_account_info(&mut accounts_iter)?, false)?;
        let new_admin =
            SanitizedSignerAccount::sanitize(next_account_info(&mut accounts_iter)?, false)?;

        Ok(Self {
            ncn,
            old_admin,
            new_admin,
        })
    }
}
