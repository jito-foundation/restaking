use solana_program::account_info::AccountInfo;

use crate::result::{SanitizationError, SanitizationResult};

#[derive(Debug)]
pub struct SanitizedMetadataProgram<'a, 'info> {
    account: &'a AccountInfo<'info>,
}

impl<'a, 'info> SanitizedMetadataProgram<'a, 'info> {
    /// Sanitizes the MetadataProgram so it can be used in a safe context
    pub fn sanitize(
        account: &'a AccountInfo<'info>,
    ) -> SanitizationResult<SanitizedMetadataProgram<'a, 'info>> {
        if account.key != &mpl_token_metadata::ID {
            return Err(SanitizationError::MetadataProgramInvalidAddress);
        }

        Ok(SanitizedMetadataProgram { account })
    }

    pub const fn account(&self) -> &AccountInfo<'info> {
        self.account
    }
}

#[cfg(test)]
mod tests {
    use assert_matches::assert_matches;
    use solana_program::{account_info::AccountInfo, clock::Epoch, pubkey::Pubkey, system_program};

    use crate::{metadata_program::SanitizedMetadataProgram, result::SanitizationError};

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
        let err = SanitizedMetadataProgram::sanitize(&account_info).unwrap_err();
        assert_matches!(err, SanitizationError::MetadataProgramInvalidAddress);
    }

    #[test]
    fn test_correct_address_ok() {
        let mut data: Vec<_> = vec![0];
        let mut lamports = 0;

        let program_id = mpl_token_metadata::ID;
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
        assert!(SanitizedMetadataProgram::sanitize(&account_info).is_ok());
    }
}
