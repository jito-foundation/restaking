use super::{account_header, field, section_header, PrettyDisplay};
use crate::accounts::OperatorVaultTicket;

impl PrettyDisplay for OperatorVaultTicket {
    fn pretty_display(&self) -> String {
        let mut output = String::new();

        output.push_str(&account_header("Operator Vault Ticket Account"));

        output.push_str(&section_header("Basic Info"));
        output.push_str(&field("Operator", self.operator));
        output.push_str(&field("Vault", self.vault));
        output.push_str(&field("Index", self.index));
        output.push_str(&field("Bump", self.bump));

        output.push_str(&section_header("State"));
        output.push_str(&field("Opt-In Added", self.state.slot_added));
        output.push_str(&field("Opt-In Removed", self.state.slot_removed));

        output
    }
}
