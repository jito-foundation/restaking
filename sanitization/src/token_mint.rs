use solana_program::{account_info::AccountInfo, program_pack::Pack};
use spl_token::state::Mint;

use crate::result::{SanitizationError, SanitizationResult};

#[derive(Debug)]
pub struct SanitizedTokenMint<'a, 'info> {
    account: &'a AccountInfo<'info>,
    mint: Mint,
}

impl<'a, 'info> SanitizedTokenMint<'a, 'info> {
    /// Sanitizes the TokenMint so it can be used in a safe context
    pub fn sanitize(
        account: &'a AccountInfo<'info>,
        expect_writable: bool,
    ) -> SanitizationResult<SanitizedTokenMint<'a, 'info>> {
        if expect_writable && !account.is_writable {
            return Err(SanitizationError::TokenMintExpectedWritable);
        }

        let mint = Mint::unpack(&account.data.borrow())
            .map_err(|_| SanitizationError::TokenMintInvalidAccountData)?;

        if account.owner != &spl_token::id() {
            return Err(SanitizationError::TokenMintInvalidProgramOwner);
        }

        Ok(SanitizedTokenMint { account, mint })
    }

    pub const fn account(&self) -> &AccountInfo<'info> {
        self.account
    }

    pub const fn mint(&self) -> &Mint {
        &self.mint
    }

    /// Reload needs to be called after CPIs to ensure the data is up-to-date
    pub fn reload(&mut self) -> SanitizationResult<()> {
        self.mint = Mint::unpack(&self.account.data.borrow())
            .map_err(|_| SanitizationError::TokenMintInvalidAccountData)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use assert_matches::assert_matches;
    use solana_program::{
        account_info::AccountInfo, clock::Epoch, program_pack::Pack, pubkey::Pubkey,
    };
    use spl_token::state::Mint;

    use crate::{result::SanitizationError, token_mint::SanitizedTokenMint};

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
        let err = SanitizedTokenMint::sanitize(&account_info, false).unwrap_err();
        assert_matches!(err, SanitizationError::TokenMintInvalidAccountData);
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
        let err = SanitizedTokenMint::sanitize(&account_info, false).unwrap_err();

        assert_matches!(err, SanitizationError::TokenMintInvalidProgramOwner);
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
        let err = SanitizedTokenMint::sanitize(&account_info, false).unwrap_err();
        assert_matches!(err, SanitizationError::TokenMintInvalidAccountData);
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
        let err = SanitizedTokenMint::sanitize(&account_info, false).unwrap_err();
        assert_matches!(err, SanitizationError::TokenMintInvalidAccountData);
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
        SanitizedTokenMint::sanitize(&account_info, false).unwrap();
    }
}
