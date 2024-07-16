use borsh::{BorshDeserialize, BorshSerialize};
use bytemuck::{Pod, Zeroable};

pub mod avs;
pub mod avs_operator_ticket;
pub mod avs_vault_slasher_ticket;
pub mod avs_vault_ticket;
pub mod config;
pub mod operator;
pub mod operator_avs_ticket;
pub mod operator_vault_ticket;
pub mod result;

#[derive(Debug, Clone, PartialEq, Eq, BorshDeserialize, BorshSerialize, Copy)]
#[repr(u32)]
pub enum AccountType {
    Config,
    Avs,
    AvsOperatorTicket,
    AvsVaultSlasherTicket,
    AvsVaultTicket,
    Operator,
    OperatorAvsTicket,
    OperatorVaultTicket,
}

unsafe impl Pod for AccountType {}
unsafe impl Zeroable for AccountType {}
