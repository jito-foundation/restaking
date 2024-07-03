use solana_program::{account_info::AccountInfo, system_program};

use crate::result::{SanitizationError, SanitizationResult};

#[derive(Debug)]
pub struct SanitizedSystemProgram<'a, 'info> {
    account: &'a AccountInfo<'info>,
}

impl<'a, 'info> SanitizedSystemProgram<'a, 'info> {
    /// Sanitizes the SystemProgram so it can be used in a safe context
    pub fn sanitize(
        account: &'a AccountInfo<'info>,
    ) -> SanitizationResult<SanitizedSystemProgram<'a, 'info>> {
        if account.key != &system_program::id() {
            return Err(SanitizationError::SystemProgramInvalidAddress);
        }

        Ok(SanitizedSystemProgram { account })
    }

    pub const fn account(&self) -> &AccountInfo<'info> {
        self.account
    }
}

#[cfg(test)]
mod tests {
    use assert_matches::assert_matches;
    use solana_program::{account_info::AccountInfo, clock::Epoch, pubkey::Pubkey, system_program};

    use crate::{result::SanitizationError, system_program::SanitizedSystemProgram};

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
        let err = SanitizedSystemProgram::sanitize(&account_info).unwrap_err();
        assert_matches!(err, SanitizationError::SystemProgramInvalidAddress);
    }

    #[test]
    fn test_correct_address_ok() {
        let mut data: Vec<_> = vec![0];
        let mut lamports = 0;

        let program_id = system_program::id();
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
        SanitizedSystemProgram::sanitize(&account_info).unwrap();
    }
}
