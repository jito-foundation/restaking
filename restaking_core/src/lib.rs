pub mod config;
pub mod ncn;
pub mod ncn_operator_state;
pub mod ncn_vault_slasher_ticket;
pub mod ncn_vault_ticket;
pub mod operator;
pub mod operator_vault_ticket;

// Maximum allowed fee in basis points (100%)
pub const MAX_FEE_BPS: u16 = 10_000;
