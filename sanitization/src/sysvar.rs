use solana_program::{
    account_info::AccountInfo,
    sysvar::{self},
};

use crate::result::{SanitizationError, SanitizationResult};

#[derive(Debug)]
pub struct SanitizedSysvar<'a, 'info> {
    account: &'a AccountInfo<'info>,
}

impl<'a, 'info> SanitizedSysvar<'a, 'info> {
    /// Sanitizes the TokenProgram so it can be used in a safe context
    pub fn sanitize(
        account: &'a AccountInfo<'info>,
    ) -> SanitizationResult<SanitizedSysvar<'a, 'info>> {
        if account.key != &sysvar::id() {
            return Err(SanitizationError::SysvarInvalidAddress);
        }

        Ok(SanitizedSysvar { account })
    }

    pub const fn account(&self) -> &AccountInfo<'info> {
        self.account
    }
}

#[cfg(test)]
mod tests {
    use assert_matches::assert_matches;
    use solana_program::{
        account_info::AccountInfo, clock::Epoch, pubkey::Pubkey, system_program, sysvar,
    };

    use crate::{result::SanitizationError, sysvar::SanitizedSysvar};

    #[test]
    fn test_wrong_address_fails() {
        let mut data: Vec<_> = vec![0];
        let key = Pubkey::new_unique();
        let mut lamports = 0;

        let bad_program_id = Pubkey::new_unique();
        let account_info = AccountInfo::new(
            &key,
            false,
            false,
            &mut lamports,
            &mut data,
            &bad_program_id,
            false,
            Epoch::MAX,
        );
        let err = SanitizedSysvar::sanitize(&account_info).unwrap_err();
        assert_matches!(err, SanitizationError::SysvarInvalidAddress);
    }

    #[test]
    fn test_correct_address_ok() {
        let mut data: Vec<_> = vec![0];
        let mut lamports = 0;

        let program_id = sysvar::id();
        let system_program = system_program::id();
        let account_info = AccountInfo::new(
            &program_id,
            false,
            false,
            &mut lamports,
            &mut data,
            &system_program,
            false,
            Epoch::MAX,
        );
        assert!(SanitizedSysvar::sanitize(&account_info).is_ok());
    }
}
