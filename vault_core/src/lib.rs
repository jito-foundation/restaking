pub mod config;
pub mod delegation_state;
pub mod discriminators;
pub mod loader;
pub mod vault;
pub mod vault_ncn_slasher_operator_ticket;
pub mod vault_ncn_slasher_ticket;
pub mod vault_ncn_ticket;
pub mod vault_operator_delegation;
pub mod vault_staker_withdrawal_ticket;
pub mod vault_update_state_tracker;

pub const MAX_BPS: u16 = 10_000;
pub const MAX_FEE_BPS: u16 = MAX_BPS;
