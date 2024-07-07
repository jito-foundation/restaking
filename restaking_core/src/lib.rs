use borsh::{BorshDeserialize, BorshSerialize};

pub mod avs;
pub mod avs_operator_list;
pub mod avs_slasher_list;
pub mod avs_vault_list;
pub mod config;
pub mod operator;
pub mod operator_avs_list;
pub mod operator_vault_list;
pub mod result;
pub mod vault;

#[derive(Debug, Clone, PartialEq, Eq, BorshDeserialize, BorshSerialize)]
#[repr(u32)]
pub enum AccountType {
    Config = 0,
    Avs = 1,
    AvsOperatorList = 2,
    AvsVaultList = 3,
    AvsSlasherList = 4,
    Operator = 5,
    OperatorAvsList = 6,
    OperatorVaultList = 7,
}
