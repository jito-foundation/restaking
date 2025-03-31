use super::{account_header, field, section_header, PrettyDisplay};
use crate::accounts::VaultNcnSlasherOperatorTicket;

impl PrettyDisplay for VaultNcnSlasherOperatorTicket {
    fn pretty_display(&self) -> String {
        let mut output = String::new();

        output.push_str(&account_header("Vault NCN Slasher Operator Ticket Account"));

        output.push_str(&section_header("Basic Information"));
        output.push_str(&field("Vault", self.vault));
        output.push_str(&field("NCN", self.ncn));
        output.push_str(&field("Slasher", self.slasher));
        output.push_str(&field("Operator", self.operator));
        output.push_str(&field("Epoch", self.epoch));
        output.push_str(&field("Slashed", self.slashed));
        output.push_str(&field("Bump", self.bump));

        output
    }
}
