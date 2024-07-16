use borsh::{BorshDeserialize, BorshSerialize};

pub mod config;
pub mod result;
pub mod vault;
pub mod vault_avs_ticket;
pub mod vault_delegation_list;
pub mod vault_operator_ticket;
pub mod vault_slasher_ticket;

#[derive(Debug, Clone, Copy, PartialEq, BorshDeserialize, BorshSerialize)]
#[repr(u32)]
enum AccountType {
    Config,
    Vault,
    VaultOperatorTicket,
    VaultAvsSlasherTicket,
    VaultAvsTicket,
    VaultDelegationList,
}
