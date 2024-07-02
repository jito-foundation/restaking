use borsh::{BorshDeserialize, BorshSerialize};

pub mod config;
pub mod result;
pub mod vault;
pub mod vault_avs_list;
pub mod vault_operator_list;
pub mod vault_slasher_list;

#[derive(Debug, Clone, Copy, PartialEq, BorshDeserialize, BorshSerialize)]
#[repr(u32)]
enum AccountType {
    Config = 0,
    Vault = 1,
    VaultAvsList = 2,
    VaultOperatorList = 3,
    VaultSlasherList = 4,
}
