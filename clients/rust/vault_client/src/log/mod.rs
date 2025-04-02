pub(crate) mod config;
pub mod metadata;
pub(crate) mod vault;
pub(crate) mod vault_ncn_slasher_operator_ticket;
pub(crate) mod vault_ncn_slasher_ticket;
pub(crate) mod vault_ncn_ticket;
pub(crate) mod vault_operator_delegation;
pub(crate) mod vault_staker_withdrawal_ticket;
pub(crate) mod vault_update_state_tracker;

pub trait PrettyDisplay {
    fn pretty_display(&self) -> String;
}

fn account_header(title: &str) -> String {
    format!("\n{}\n", title)
}

fn section_header(title: &str) -> String {
    format!("\n{}\n", format!("━━━ {} ━━━", title))
}

fn field(name: &str, value: impl std::fmt::Display) -> String {
    format!("  {}: {}\n", name, value)
}
