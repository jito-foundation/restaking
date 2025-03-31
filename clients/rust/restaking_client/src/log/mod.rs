pub(crate) mod config;
pub(crate) mod ncn;
pub(crate) mod ncn_operator_state;
pub(crate) mod ncn_vault_slasher_ticket;
pub(crate) mod ncn_vault_ticket;
pub(crate) mod operator;
pub(crate) mod operator_vault_ticket;

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
