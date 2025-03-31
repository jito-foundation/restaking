use crate::accounts::Config;

use super::{account_header, field, section_header, PrettyDisplay};

impl PrettyDisplay for Config {
    fn pretty_display(&self) -> String {
        let mut output = String::new();

        output.push_str(&account_header("Config Account"));

        output.push_str(&section_header("Admin Authorities"));
        output.push_str(&field("Admin", self.admin));

        output.push_str(&section_header("Basic Info"));
        output.push_str(&field("Vault Program", self.vault_program));
        output.push_str(&field("Bump", self.bump));

        output.push_str(&section_header("Statistics"));
        output.push_str(&field("NCN Count", self.ncn_count));
        output.push_str(&field("Operator Count", self.operator_count));

        output.push_str(&section_header("Epoch Info"));
        output.push_str(&field("Epoch Length", self.epoch_length));

        output
    }
}
