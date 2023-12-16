use solana_program::{account_info::AccountInfo, program_error::ProgramError};

use crate::assert_with_msg;

#[derive(Debug)]
pub struct SanitizedSignerAccount<'a, 'info> {
    account: &'a AccountInfo<'info>,
}

impl<'a, 'info> SanitizedSignerAccount<'a, 'info> {
    /// Sanitizes the SignerAccount so it can be used in a safe context
    pub fn sanitize(
        account: &'a AccountInfo<'info>,
        expect_writable: bool,
    ) -> Result<SanitizedSignerAccount<'a, 'info>, ProgramError> {
        assert_with_msg(
            account.is_signer,
            ProgramError::InvalidAccountData,
            "Signer account is not a signer",
        )?;

        if expect_writable {
            assert_with_msg(
                account.is_writable,
                ProgramError::InvalidAccountData,
                "Invalid writable flag for signer",
            )?;
        }

        Ok(SanitizedSignerAccount { account })
    }

    pub const fn account(&self) -> &AccountInfo<'info> {
        self.account
    }
}

#[cfg(test)]
mod tests {
    use assert_matches::assert_matches;
    use solana_program::{
        account_info::AccountInfo, clock::Epoch, program_error::ProgramError, pubkey::Pubkey,
    };

    use crate::signer::SanitizedSignerAccount;

    #[test]
    fn test_not_signer_fails() {
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
        let err = SanitizedSignerAccount::sanitize(&account_info, false).unwrap_err();

        assert_matches!(err, ProgramError::InvalidAccountData);
    }

    #[test]
    fn test_not_signer_wrong_writable_fails() {
        let mut data: Vec<_> = vec![0];
        let key = Pubkey::new_unique();
        let mut lamports = 0;

        let bad_program_id = Pubkey::new_unique();
        let account_info = AccountInfo::new(
            &key,
            true,
            false,
            &mut lamports,
            &mut data,
            &bad_program_id,
            false,
            Epoch::MAX,
        );
        let err = SanitizedSignerAccount::sanitize(&account_info, true).unwrap_err();
        assert_matches!(err, ProgramError::InvalidAccountData);
    }
}
