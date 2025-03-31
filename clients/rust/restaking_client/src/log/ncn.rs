use crate::accounts::Ncn;

use super::{account_header, field, section_header, PrettyDisplay};

impl PrettyDisplay for Ncn {
    fn pretty_display(&self) -> String {
        let mut output = String::new();

        output.push_str(&account_header("Ncn Account"));

        output.push_str(&section_header("Basic Info"));
        output.push_str(&field("Base", self.base));
        output.push_str(&field("Index", self.index));
        output.push_str(&field("Bump", self.bump));

        output.push_str(&section_header("Admin Authorities"));
        output.push_str(&field("Admin", self.admin));
        output.push_str(&field("Operator Admin", self.operator_admin));
        output.push_str(&field("Vault Admin", self.vault_admin));
        output.push_str(&field("Slasher Admin", self.slasher_admin));
        output.push_str(&field("Delegate Admin", self.delegate_admin));
        output.push_str(&field("Metadata Admin", self.metadata_admin));
        output.push_str(&field("Weight Table Admin", self.weight_table_admin));
        output.push_str(&field("NCN Program Admin", self.ncn_program_admin));

        output.push_str(&section_header("Statistics"));
        output.push_str(&field("Operator Count", self.operator_count));
        output.push_str(&field("Vault Count", self.vault_count));
        output.push_str(&field("Slasher Count", self.slasher_count));

        output
    }
}
