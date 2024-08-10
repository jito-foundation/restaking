use borsh::{BorshDeserialize, BorshSerialize};
use bytemuck::{Pod, Zeroable};

pub mod config;
pub mod operator_delegation;
pub mod result;
pub mod vault;
pub mod vault_delegation_list;
pub mod vault_ncn_slasher_operator_ticket;
pub mod vault_ncn_slasher_ticket;
pub mod vault_ncn_ticket;
pub mod vault_operator_ticket;
pub mod vault_staker_withdrawal_ticket;

#[derive(Debug, Clone, Copy, PartialEq, BorshDeserialize, BorshSerialize)]
#[repr(u32)]
enum AccountType {
    Config = 0,
    Vault = 1,
    VaultOperatorTicket = 2,
    VaultNcnSlasherTicket = 3,
    VaultNcnTicket = 4,
    VaultDelegationList = 5,
    VaultNcnSlasherOperatorTicket = 6,
    VaultStakerWithdrawalTicket = 7,
}

unsafe impl Zeroable for AccountType {}
unsafe impl Pod for AccountType {}
