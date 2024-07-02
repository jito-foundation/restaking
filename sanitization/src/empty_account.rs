use solana_program::{account_info::AccountInfo, program_error::ProgramError};

use crate::assert_with_msg;

pub struct EmptyAccount<'a, 'info> {
    account: &'a AccountInfo<'info>,
}

impl<'a, 'info> EmptyAccount<'a, 'info> {
    pub fn sanitize(
        account: &'a AccountInfo<'info>,
        expect_writable: bool,
    ) -> Result<EmptyAccount<'a, 'info>, ProgramError> {
        if expect_writable {
            assert_with_msg(
                account.is_writable,
                ProgramError::InvalidAccountData,
                "Invalid writable flag for empty account",
            )?;
        }

        assert_with_msg(
            account.data_is_empty(),
            ProgramError::InvalidAccountData,
            "Invalid empty account data",
        )?;

        Ok(EmptyAccount { account })
    }

    pub const fn account(&self) -> &AccountInfo<'info> {
        self.account
    }
}
