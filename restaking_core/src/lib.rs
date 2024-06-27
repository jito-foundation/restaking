use borsh::{BorshDeserialize, BorshSerialize};

pub mod avs;
pub mod avs_operator_list;
pub mod avs_slasher_list;
pub mod avs_vault_list;
pub mod config;
pub mod node_operator;
pub mod vault;

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
