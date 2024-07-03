use solana_program::account_info::AccountInfo;

use crate::result::{SanitizationError, SanitizationResult};

pub struct EmptyAccount<'a, 'info> {
    account: &'a AccountInfo<'info>,
}

impl<'a, 'info> EmptyAccount<'a, 'info> {
    pub fn sanitize(
        account: &'a AccountInfo<'info>,
        expect_writable: bool,
    ) -> SanitizationResult<EmptyAccount<'a, 'info>> {
        if expect_writable && !account.is_writable {
            return Err(SanitizationError::EmptyAccountNotWritable);
        }
        if !account.data_is_empty() {
            return Err(SanitizationError::EmptyAccountNotEmpty);
        }

        Ok(EmptyAccount { account })
    }

    pub const fn account(&self) -> &AccountInfo<'info> {
        self.account
    }
}
