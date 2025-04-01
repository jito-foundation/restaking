use super::{account_header, field, section_header, PrettyDisplay};
use crate::accounts::VaultUpdateStateTracker;

impl PrettyDisplay for VaultUpdateStateTracker {
    fn pretty_display(&self) -> String {
        let mut output = String::new();

        output.push_str(&account_header("Vault Update State Tracker Account"));

        output.push_str(&section_header("Basic Information"));
        output.push_str(&field("Vault", self.vault));
        output.push_str(&field("NCN Epoch", self.ncn_epoch));
        output.push_str(&field("Last Updated Index", self.last_updated_index));

        let withdrawal_allocation_method = match self.withdrawal_allocation_method {
            0 => "Greedy",
            _ => "",
        };
        output.push_str(&field(
            "Withdrawal Allocation Method",
            withdrawal_allocation_method,
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

#[cfg(test)]
mod tests {
    use anchor_lang::prelude::Pubkey;

    use crate::{accounts::VaultUpdateStateTracker, log::PrettyDisplay, types::DelegationState};

    #[test]
    fn test_vault_update_state_tracker_pretty_display_structure() {
        let vault_update_state_tracker = VaultUpdateStateTracker {
            discriminator: 12345,
            vault: Pubkey::new_unique(),
            ncn_epoch: 1,
            last_updated_index: 2,
            delegation_state: DelegationState {
                staked_amount: 3,
                enqueued_for_cooldown_amount: 4,
                cooling_down_amount: 5,
                reserved: [0; 256],
            },
            withdrawal_allocation_method: 6,
            reserved: [0; 263],
        };

        let output = vault_update_state_tracker.pretty_display();

        assert!(output.contains(&vault_update_state_tracker.vault.to_string()));
        assert!(output.contains(&vault_update_state_tracker.ncn_epoch.to_string()));
        assert!(output.contains(&vault_update_state_tracker.last_updated_index.to_string()));
        assert!(output.contains(
            &vault_update_state_tracker
                .delegation_state
                .staked_amount
                .to_string()
        ));
        assert!(output.contains(
            &vault_update_state_tracker
                .delegation_state
                .enqueued_for_cooldown_amount
                .to_string()
        ));
        assert!(output.contains(
            &vault_update_state_tracker
                .delegation_state
                .cooling_down_amount
                .to_string()
        ));
        assert!(output.contains(
            &vault_update_state_tracker
                .withdrawal_allocation_method
                .to_string()
        ));
    }
}
