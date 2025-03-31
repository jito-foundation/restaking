use crate::accounts::VaultUpdateStateTracker;

use super::{account_header, field, section_header, PrettyDisplay};

impl PrettyDisplay for VaultUpdateStateTracker {
    fn pretty_display(&self) -> String {
        let mut output = String::new();

        output.push_str(&account_header("Vault Update State Tracker Account"));

        output.push_str(&section_header("Basic Information"));
        output.push_str(&field("Vault", self.vault));
        output.push_str(&field("NCN Epoch", self.ncn_epoch));
        output.push_str(&field("Last Updated Index", self.last_updated_index));
        output.push_str(&field(
            "Withdrawal Allocation Method",
            self.withdrawal_allocation_method,
        ));

        output.push_str(&section_header("Delegation State"));
        output.push_str(&field(
            "Enqueued for Cooldown Amount",
            self.delegation_state.enqueued_for_cooldown_amount,
        ));
        output.push_str(&field(
            "Coolindown Amount",
            self.delegation_state.cooling_down_amount,
        ));
        output.push_str(&field("Staked Amount", self.delegation_state.staked_amount));

        output
    }
}
