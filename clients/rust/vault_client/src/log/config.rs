use super::{account_header, field, section_header, PrettyDisplay};
use crate::accounts::Config;

impl PrettyDisplay for Config {
    fn pretty_display(&self) -> String {
        let mut output = String::new();

        output.push_str(&account_header("Config Account"));

        output.push_str(&section_header("Admin Authorities"));
        output.push_str(&field("Admin", self.admin));
        output.push_str(&field("Program Fee Wallet", self.program_fee_wallet));
        output.push_str(&field("Fee Admin", self.fee_admin));

        output.push_str(&section_header("Basic Information"));
        output.push_str(&field("Restaking Program", self.restaking_program));
        output.push_str(&field("Bump", self.bump));

        output.push_str(&section_header("Statistics"));
        output.push_str(&field("Vault Count", self.num_vaults));
        output.push_str(&field(
            "Deposit Withdrawal Fee Cap BPS",
            self.deposit_withdrawal_fee_cap_bps,
        ));
        output.push_str(&field(
            "Fee Rate of Change BPS",
            self.fee_rate_of_change_bps,
        ));
        output.push_str(&field("Fee Bump BPS", self.fee_bump_bps));
        output.push_str(&field("Program Fee BPS", self.program_fee_bps));

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
            restaking_program: Pubkey::new_unique(),
            epoch_length: 0,
            num_vaults: 1,
            deposit_withdrawal_fee_cap_bps: 2,
            fee_rate_of_change_bps: 3,
            fee_bump_bps: 4,
            program_fee_bps: 5,
            program_fee_wallet: Pubkey::new_unique(),
            fee_admin: Pubkey::new_unique(),
            bump: 248,
            reserved: [0; 229],
        };

        let output = config.pretty_display();

        assert!(output.contains(&config.admin.to_string()));
        assert!(output.contains(&config.restaking_program.to_string()));
        assert!(output.contains(&config.epoch_length.to_string()));
        assert!(output.contains(&config.num_vaults.to_string()));
        assert!(output.contains(&config.deposit_withdrawal_fee_cap_bps.to_string()));
        assert!(output.contains(&config.fee_rate_of_change_bps.to_string()));
        assert!(output.contains(&config.program_fee_bps.to_string()));
        assert!(output.contains(&config.program_fee_wallet.to_string()));
        assert!(output.contains(&config.fee_admin.to_string()));
        assert!(output.contains(&config.bump.to_string()));
    }
}
