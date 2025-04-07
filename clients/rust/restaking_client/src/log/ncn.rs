use jito_restaking_client_common::log::{account_header, field, section_header, PrettyDisplay};

use crate::accounts::Ncn;

impl PrettyDisplay for Ncn {
    fn pretty_display(&self) -> String {
        let mut output = String::new();

        output.push_str(&account_header("Ncn Account"));

        output.push_str(&section_header("Basic Information"));
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

#[cfg(test)]
mod tests {
    use anchor_lang::prelude::Pubkey;
    use jito_restaking_client_common::log::PrettyDisplay;

    use crate::accounts::Ncn;

    #[test]
    fn test_ncn_pretty_display_structure() {
        let ncn = Ncn {
            discriminator: 12345,
            base: Pubkey::new_unique(),
            admin: Pubkey::new_unique(),
            operator_admin: Pubkey::new_unique(),
            vault_admin: Pubkey::new_unique(),
            slasher_admin: Pubkey::new_unique(),
            delegate_admin: Pubkey::new_unique(),
            metadata_admin: Pubkey::new_unique(),
            weight_table_admin: Pubkey::new_unique(),
            ncn_program_admin: Pubkey::new_unique(),
            index: 0,
            operator_count: 1,
            vault_count: 2,
            slasher_count: 3,
            bump: 254,
            reserved: [0; 263],
        };

        let output = ncn.pretty_display();

        assert!(output.contains(&ncn.base.to_string()));
        assert!(output.contains(&ncn.admin.to_string()));
        assert!(output.contains(&ncn.operator_admin.to_string()));
        assert!(output.contains(&ncn.vault_admin.to_string()));
        assert!(output.contains(&ncn.slasher_admin.to_string()));
        assert!(output.contains(&ncn.delegate_admin.to_string()));
        assert!(output.contains(&ncn.metadata_admin.to_string()));
        assert!(output.contains(&ncn.weight_table_admin.to_string()));
        assert!(output.contains(&ncn.ncn_program_admin.to_string()));
        assert!(output.contains(&ncn.index.to_string()));
        assert!(output.contains(&ncn.operator_count.to_string()));
        assert!(output.contains(&ncn.vault_count.to_string()));
        assert!(output.contains(&ncn.slasher_count.to_string()));
        assert!(output.contains(&ncn.bump.to_string()));
    }
}
