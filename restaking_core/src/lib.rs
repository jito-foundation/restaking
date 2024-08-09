use borsh::{BorshDeserialize, BorshSerialize};
use bytemuck::{Pod, Zeroable};

pub mod config;
pub mod ncn;
pub mod ncn_operator_ticket;
pub mod ncn_vault_slasher_ticket;
pub mod ncn_vault_ticket;
pub mod operator;
pub mod operator_ncn_ticket;
pub mod operator_vault_ticket;
pub mod result;

#[derive(Debug, Clone, PartialEq, Eq, BorshDeserialize, BorshSerialize, Copy)]
#[repr(u32)]
pub enum AccountType {
    Config,
    Ncn,
    NcnOperatorTicket,
    NcnVaultSlasherTicket,
    NcnVaultTicket,
    Operator,
    OperatorNcnTicket,
    OperatorVaultTicket,
}

unsafe impl Pod for AccountType {}
unsafe impl Zeroable for AccountType {}
