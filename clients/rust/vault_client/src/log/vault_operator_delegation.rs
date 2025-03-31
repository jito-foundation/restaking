use crate::accounts::VaultOperatorDelegation;

use super::{account_header, field, section_header, PrettyDisplay};

impl PrettyDisplay for VaultOperatorDelegation {
    fn pretty_display(&self) -> String {
        let mut output = String::new();

        output.push_str(&account_header("Vault Operator Delegation Account"));

        output.push_str(&section_header("Basic Information"));
        output.push_str(&field("Vault", self.vault));
        output.push_str(&field("Operator", self.operator));
        output.push_str(&field("Last Update Slot", self.last_update_slot));
        output.push_str(&field("Index", self.index));
        output.push_str(&field("Bump", self.bump));

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
