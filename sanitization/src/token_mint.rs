use solana_program::{account_info::AccountInfo, program_error::ProgramError, program_pack::Pack};
use spl_token::state::Mint;

use crate::assert_with_msg;

#[derive(Debug)]
pub struct SanitizedTokenMint<'a, 'info> {
    account: &'a AccountInfo<'info>,
    mint: Mint,
}

impl<'a, 'info> SanitizedTokenMint<'a, 'info> {
    /// Sanitizes the TokenMint so it can be used in a safe context
    pub fn sanitize(
        account: &'a AccountInfo<'info>,
    ) -> Result<SanitizedTokenMint<'a, 'info>, ProgramError> {
        let mint = Mint::unpack(&account.data.borrow());

        assert_with_msg(
            mint.is_ok(),
            ProgramError::InvalidAccountData,
            "Invalid token mint data",
        )?;

        assert_with_msg(
            account.owner == &spl_token::id(),
            ProgramError::InvalidAccountData,
            "Invalid token mint owner",
        )?;

        Ok(SanitizedTokenMint {
            account,
            mint: mint.unwrap(),
        })
    }

    pub const fn account(&self) -> &AccountInfo<'info> {
        self.account
    }

    pub const fn mint(&self) -> &Mint {
        &self.mint
    }

    /// Reload needs to be called after CPIs to ensure the data is up-to-date
    pub fn reload(&mut self) -> Result<(), ProgramError> {
        self.mint = Mint::unpack(&self.account.data.borrow())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use assert_matches::assert_matches;
    use solana_program::{
        account_info::AccountInfo, clock::Epoch, program_error::ProgramError, program_pack::Pack,
        pubkey::Pubkey,
    };
    use spl_token::state::Mint;

    use crate::token_mint::SanitizedTokenMint;

    #[test]
    fn test_incorrect_owner_fails() {
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
        let err = SanitizedTokenMint::sanitize(&account_info).unwrap_err();
        assert_matches!(err, ProgramError::InvalidAccountData);
    }

    #[test]
    fn test_incorrect_owner_bad_mint_fails() {
        let key = Pubkey::new_unique();
        let mut lamports = 0;

        let bad_program_id = Pubkey::new_unique();
        let mint = Mint {
            mint_authority: Default::default(),
            supply: 0,
            decimals: 0,
            is_initialized: true,
            freeze_authority: Default::default(),
        };
        let mut data: Vec<_> = vec![0; Mint::LEN];
        mint.pack_into_slice(&mut data);

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
        let err = SanitizedTokenMint::sanitize(&account_info).unwrap_err();

        assert_matches!(err, ProgramError::InvalidAccountData);
    }

    #[test]
    fn test_correct_owner_bad_mint_fails() {
        let key = Pubkey::new_unique();
        let mut lamports = 0;

        let program_id = spl_token::id();
        let mut data: Vec<_> = vec![0; Mint::LEN];

        let account_info = AccountInfo::new(
            &key,
            false,
            false,
            &mut lamports,
            &mut data,
            &program_id,
            false,
            Epoch::MAX,
        );
        let err = SanitizedTokenMint::sanitize(&account_info).unwrap_err();
        assert_matches!(err, ProgramError::InvalidAccountData);
    }

    #[test]
    fn test_correct_owner_uninitialized_mint_fails() {
        let key = Pubkey::new_unique();
        let mut lamports = 0;

        let program_id = spl_token::id();
        let mint = Mint {
            mint_authority: Default::default(),
            supply: 0,
            decimals: 0,
            is_initialized: false,
            freeze_authority: Default::default(),
        };
        let mut data: Vec<_> = vec![0; Mint::LEN];
        mint.pack_into_slice(&mut data);

        let account_info = AccountInfo::new(
            &key,
            false,
            false,
            &mut lamports,
            &mut data,
            &program_id,
            false,
            Epoch::MAX,
        );
        let err = SanitizedTokenMint::sanitize(&account_info).unwrap_err();
        assert_matches!(err, ProgramError::InvalidAccountData);
    }

    #[test]
    fn test_correct_owner_ok_mint_ok() {
        let key = Pubkey::new_unique();
        let mut lamports = 0;

        let program_id = spl_token::id();
        let mint = Mint {
            mint_authority: Default::default(),
            supply: 0,
            decimals: 0,
            is_initialized: true,
            freeze_authority: Default::default(),
        };
        let mut data: Vec<_> = vec![0; Mint::LEN];
        mint.pack_into_slice(&mut data);

        let account_info = AccountInfo::new(
            &key,
            false,
            false,
            &mut lamports,
            &mut data,
            &program_id,
            false,
            Epoch::MAX,
        );
        SanitizedTokenMint::sanitize(&account_info).unwrap();
    }
}
