use solana_program::{account_info::AccountInfo, program_pack::Pack, pubkey::Pubkey};
use spl_token::state::Account;

use crate::result::{SanitizationError, SanitizationResult};

pub struct SanitizedTokenAccount<'a, 'info> {
    inner: &'a AccountInfo<'info>,
    token_account: Account,
}

impl<'a, 'info> SanitizedTokenAccount<'a, 'info> {
    pub fn sanitize(
        account: &'a AccountInfo<'info>,
        mint: &Pubkey,
        owner: &Pubkey,
    ) -> SanitizationResult<SanitizedTokenAccount<'a, 'info>> {
        if *account.owner != spl_token::id() {
            return Err(SanitizationError::TokenAccountInvalidProgramOwner);
        }

        let token_account = Account::unpack(&account.data.borrow())
            .map_err(|_| SanitizationError::TokenAccountInvalidAccountData)?;

        if token_account.mint != *mint {
            return Err(SanitizationError::TokenAccountInvalidMint);
        }

        if token_account.owner != *owner {
            return Err(SanitizationError::TokenAccountInvalidOwner);
        }

        Ok(SanitizedTokenAccount {
            inner: account,
            token_account,
        })
    }

    pub const fn token_account(&self) -> &Account {
        &self.token_account
    }

    pub const fn account(&self) -> &AccountInfo<'info> {
        self.inner
    }

    /// Reload needs to be called after CPIs to ensure the data is up-to-date
    pub fn reload(&mut self) -> SanitizationResult<()> {
        self.token_account = Account::unpack(&self.inner.data.borrow())
            .map_err(|_| SanitizationError::TokenAccountInvalidAccountData)?;
        Ok(())
    }
}
