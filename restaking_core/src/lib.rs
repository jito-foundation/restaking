use borsh::{BorshDeserialize, BorshSerialize};

pub mod avs;
pub mod config;
pub mod node_operator;
mod vault;

#[derive(Debug, Clone, PartialEq, Eq, BorshDeserialize, BorshSerialize)]
#[repr(u32)]
pub enum AccountType {
    Config = 0,
    Avs = 1,
    AvsOperatorList = 2,
    AvsVaultList = 3,
    AvsSlasherList = 4,
    NodeOperator = 5,
    NodeOperatorAvsList = 6,
    NodeOperatorVaultList = 7,
}
