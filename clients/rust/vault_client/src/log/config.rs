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

        output.push_str(&section_header("Basic Info"));
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
        output.push_str(&field("Program Fee BPS", self.fee_bump_bps));

        output.push_str(&section_header("Epoch Info"));
        output.push_str(&field("Epoch Length", self.epoch_length));

        output
    }
}
