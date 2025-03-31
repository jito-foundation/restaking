use crate::accounts::VaultNcnTicket;

use super::{account_header, field, section_header, PrettyDisplay};

impl PrettyDisplay for VaultNcnTicket {
    fn pretty_display(&self) -> String {
        let mut output = String::new();

        output.push_str(&account_header("Vault NCN Ticket Account"));

        output.push_str(&section_header("Basic Information"));
        output.push_str(&field("Vault", self.vault));
        output.push_str(&field("NCN", self.ncn));
        output.push_str(&field("Index", self.index));
        output.push_str(&field("Bump", self.bump));

        output.push_str(&section_header("State"));
        output.push_str(&field("Slot Added", self.state.slot_added));
        output.push_str(&field("Slot Removed", self.state.slot_removed));

        output
    }
}
