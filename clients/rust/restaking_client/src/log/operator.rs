use super::{account_header, field, section_header, PrettyDisplay};
use crate::accounts::Operator;

impl PrettyDisplay for Operator {
    fn pretty_display(&self) -> String {
        let mut output = String::new();

        output.push_str(&account_header("Operator Account"));

        output.push_str(&section_header("Basic Information"));
        output.push_str(&field("Base", self.base));
        output.push_str(&field("Index", self.index));
        output.push_str(&field("Bump", self.bump));

        output.push_str(&section_header("Admin Authorities"));
        output.push_str(&field("Admin", self.admin));
        output.push_str(&field("NCN Admin", self.ncn_admin));
        output.push_str(&field("Vault Admin", self.vault_admin));
        output.push_str(&field("Delegate Admin", self.delegate_admin));
        output.push_str(&field("Metadata Admin", self.metadata_admin));
        output.push_str(&field("Voter", self.voter));

        output.push_str(&section_header("Statistics"));
        output.push_str(&field("NCN Count", self.ncn_count));
        output.push_str(&field("Vault Count", self.vault_count));
        output.push_str(&field("Operator Fee BPS", self.operator_fee_bps));

        output
    }
}
