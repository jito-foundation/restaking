use super::{account_header, field, section_header, PrettyDisplay};
use crate::accounts::Config;

impl PrettyDisplay for Config {
    fn pretty_display(&self) -> String {
        let mut output = String::new();

        output.push_str(&account_header("Config Account"));

        output.push_str(&section_header("Admin Authorities"));
        output.push_str(&field("Admin", self.admin));

        output.push_str(&section_header("Basic Information"));
        output.push_str(&field("Vault Program", self.vault_program));
        output.push_str(&field("Bump", self.bump));

        output.push_str(&section_header("Statistics"));
        output.push_str(&field("NCN Count", self.ncn_count));
        output.push_str(&field("Operator Count", self.operator_count));

        output.push_str(&section_header("Epoch Information"));
        output.push_str(&field("Epoch Length", self.epoch_length));

        output
    }
}

#[cfg(test)]
mod tests {
    use anchor_lang::prelude::Pubkey;

    use crate::{accounts::Config, log::PrettyDisplay};

    #[test]
    fn test_config_pretty_display_structure() {
        let config = Config {
            discriminator: 12345,
            admin: Pubkey::new_unique(),
            vault_program: Pubkey::new_unique(),
            ncn_count: 5,
            operator_count: 10,
            epoch_length: 432000,
            bump: 254,
            reserved: [0; 263],
        };

        let output = config.pretty_display();

        assert!(output.contains(&config.admin.to_string()));
        assert!(output.contains(&config.vault_program.to_string()));
        assert!(output.contains(&config.bump.to_string()));
        assert!(output.contains(&config.epoch_length.to_string()));
        assert!(output.contains(&config.ncn_count.to_string()));
        assert!(output.contains(&config.operator_count.to_string()));
    }
}
