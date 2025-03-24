//! This code was AUTOGENERATED using the kinobi library.
//! Please DO NOT EDIT THIS FILE, instead use visitors
//! to add features, then rerun kinobi to update it.
//!
//! <https://github.com/kinobi-so/kinobi>
//!

use solana_program::pubkey::Pubkey;
use crate::generated::types::DelegationState;
use borsh::BorshSerialize;
use borsh::BorshDeserialize;


#[derive(BorshSerialize, BorshDeserialize, Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct VaultOperatorDelegation {
pub discriminator: u64,
#[cfg_attr(feature = "serde", serde(with = "serde_with::As::<serde_with::DisplayFromStr>"))]
pub vault: Pubkey,
#[cfg_attr(feature = "serde", serde(with = "serde_with::As::<serde_with::DisplayFromStr>"))]
pub operator: Pubkey,
pub delegation_state: DelegationState,
pub last_update_slot: u64,
pub index: u64,
pub bump: u8,
#[cfg_attr(feature = "serde", serde(with = "serde_with::As::<serde_with::Bytes>"))]
pub reserved: [u8; 263],
}


impl VaultOperatorDelegation {
  
  
  
  #[inline(always)]
  pub fn from_bytes(data: &[u8]) -> Result<Self, std::io::Error> {
    let mut data = data;
    Self::deserialize(&mut data)
  }
}

impl<'a> TryFrom<&solana_program::account_info::AccountInfo<'a>> for VaultOperatorDelegation {
  type Error = std::io::Error;

  fn try_from(account_info: &solana_program::account_info::AccountInfo<'a>) -> Result<Self, Self::Error> {
      let mut data: &[u8] = &(*account_info.data).borrow();
      Self::deserialize(&mut data)
  }
}

#[cfg(feature = "anchor")]
impl anchor_lang::AccountDeserialize for VaultOperatorDelegation {
    fn try_deserialize_unchecked(buf: &mut &[u8]) -> anchor_lang::Result<Self> {
      Ok(Self::deserialize(buf)?)
    }
}

#[cfg(feature = "anchor")]
impl anchor_lang::AccountSerialize for VaultOperatorDelegation {}

#[cfg(feature = "anchor")]
impl anchor_lang::Owner for VaultOperatorDelegation {
    fn owner() -> Pubkey {
      crate::JITO_VAULT_ID
    }
}

#[cfg(feature = "anchor-idl-build")]
impl anchor_lang::IdlBuild for VaultOperatorDelegation {}


#[cfg(feature = "anchor-idl-build")]
impl anchor_lang::Discriminator for VaultOperatorDelegation {
  const DISCRIMINATOR: &'static [u8] = &[0; 8];
}

