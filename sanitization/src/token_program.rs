use solana_program::account_info::AccountInfo;

use crate::result::{SanitizationError, SanitizationResult};

#[derive(Debug)]
pub struct SanitizedTokenProgram<'a, 'info> {
    account: &'a AccountInfo<'info>,
}

impl<'a, 'info> SanitizedTokenProgram<'a, 'info> {
    /// Sanitizes the TokenProgram so it can be used in a safe context
    pub fn sanitize(
        account: &'a AccountInfo<'info>,
    ) -> SanitizationResult<SanitizedTokenProgram<'a, 'info>> {
        if account.key != &spl_token::id() {
            return Err(SanitizationError::TokenProgramInvalidAddress);
        }

        Ok(SanitizedTokenProgram { account })
    }

    pub const fn account(&self) -> &AccountInfo<'info> {
        self.account
    }
}

#[cfg(test)]
mod tests {
    use assert_matches::assert_matches;
    use solana_program::{account_info::AccountInfo, clock::Epoch, pubkey::Pubkey, system_program};

    use crate::{result::SanitizationError, token_program::SanitizedTokenProgram};

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
        let err = SanitizedTokenProgram::sanitize(&account_info).unwrap_err();
        assert_matches!(err, SanitizationError::TokenProgramInvalidAddress);
    }

    #[test]
    fn test_correct_address_ok() {
        let mut data: Vec<_> = vec![0];
        let mut lamports = 0;

        let program_id = spl_token::id();
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
        SanitizedTokenProgram::sanitize(&account_info).unwrap();
    }
}
