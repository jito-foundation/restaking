//! This code was AUTOGENERATED using the kinobi library.
//! Please DO NOT EDIT THIS FILE, instead use visitors
//! to add features, then rerun kinobi to update it.
//!
//! <https://github.com/kinobi-so/kinobi>
//!

use num_derive::FromPrimitive;
use borsh::BorshSerialize;
use borsh::BorshDeserialize;

#[derive(BorshSerialize, BorshDeserialize, Clone, Debug, Eq, PartialEq, Copy, PartialOrd, Hash, FromPrimitive)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum WithdrawalAllocationMethod {
Greedy,
}


