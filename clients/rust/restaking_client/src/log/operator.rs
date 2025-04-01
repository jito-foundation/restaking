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

#[cfg(test)]
mod tests {
    use anchor_lang::prelude::Pubkey;

    use crate::{accounts::Operator, log::PrettyDisplay};

    #[test]
    fn test_operator_pretty_display_structure() {
        let operator = Operator {
            discriminator: 12345,
            base: Pubkey::new_unique(),
            admin: Pubkey::new_unique(),
            ncn_admin: Pubkey::new_unique(),
            vault_admin: Pubkey::new_unique(),
            delegate_admin: Pubkey::new_unique(),
            metadata_admin: Pubkey::new_unique(),
            voter: Pubkey::new_unique(),
            index: 1,
            ncn_count: 2,
            vault_count: 3,
            operator_fee_bps: 4,
            bump: 5,
            reserved_space: [0; 261],
        };

        let output = operator.pretty_display();

        assert!(output.contains(&operator.base.to_string()));
        assert!(output.contains(&operator.admin.to_string()));
        assert!(output.contains(&operator.ncn_admin.to_string()));
        assert!(output.contains(&operator.vault_admin.to_string()));
        assert!(output.contains(&operator.delegate_admin.to_string()));
        assert!(output.contains(&operator.metadata_admin.to_string()));
        assert!(output.contains(&operator.voter.to_string()));
        assert!(output.contains(&operator.index.to_string()));
        assert!(output.contains(&operator.ncn_count.to_string()));
        assert!(output.contains(&operator.vault_count.to_string()));
        assert!(output.contains(&operator.operator_fee_bps.to_string()));
        assert!(output.contains(&operator.bump.to_string()));
    }
}
