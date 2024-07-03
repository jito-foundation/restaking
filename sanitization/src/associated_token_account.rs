use solana_program::{account_info::AccountInfo, program_pack::Pack, pubkey::Pubkey};
use spl_associated_token_account::get_associated_token_address;
use spl_token::state::Account;

use crate::result::{SanitizationError, SanitizationResult};

pub struct SanitizedAssociatedTokenAccount<'a, 'info> {
    inner: &'a AccountInfo<'info>,
    token_account: Account,
}

impl<'a, 'info> SanitizedAssociatedTokenAccount<'a, 'info> {
    pub fn sanitize(
        account: &'a AccountInfo<'info>,
        mint: &Pubkey,
        owner: &Pubkey,
    ) -> SanitizationResult<SanitizedAssociatedTokenAccount<'a, 'info>> {
        let expected_ata = get_associated_token_address(owner, mint);

        if *account.key != expected_ata {
            return Err(SanitizationError::AssociatedTokenAccountInvalidAddress);
        }
        if account.owner != &spl_token::id() {
            return Err(SanitizationError::AssociatedTokenAccountInvalidOwner);
        }
        if account.data_len() != Account::LEN {
            return Err(SanitizationError::AssociatedTokenAccountInvalidAccountData);
        }

        let token_account = Account::unpack(&account.data.borrow())
            .map_err(|_| SanitizationError::AssociatedTokenAccountInvalidAccountData)?;

        Ok(SanitizedAssociatedTokenAccount {
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
            .map_err(|_| SanitizationError::AssociatedTokenAccountFailedReload)?;
        Ok(())
    }
}
