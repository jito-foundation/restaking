use jito_restaking_client_common::log::{account_header, field, section_header, PrettyDisplay};

use crate::accounts::VaultOperatorDelegation;

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
            "Cooling Down Amount",
            self.delegation_state.cooling_down_amount,
        ));
        output.push_str(&field("Staked Amount", self.delegation_state.staked_amount));

        output
    }
}

#[cfg(test)]
mod tests {
    use anchor_lang::prelude::Pubkey;
    use jito_restaking_client_common::log::PrettyDisplay;

    use crate::{accounts::VaultOperatorDelegation, types::DelegationState};

    #[test]
    fn test_vault_operator_delegation_pretty_display_structure() {
        let vault_operator_delegation = VaultOperatorDelegation {
            discriminator: 12345,
            vault: Pubkey::new_unique(),
            operator: Pubkey::new_unique(),
            delegation_state: DelegationState {
                staked_amount: 1,
                enqueued_for_cooldown_amount: 2,
                cooling_down_amount: 3,
                reserved: [0; 256],
            },
            last_update_slot: 4,
            index: 5,
            bump: 6,
            reserved: [0; 263],
        };

        let output = vault_operator_delegation.pretty_display();

        assert!(output.contains(&vault_operator_delegation.vault.to_string()));
        assert!(output.contains(&vault_operator_delegation.operator.to_string()));
        assert!(output.contains(
            &vault_operator_delegation
                .delegation_state
                .staked_amount
                .to_string()
        ));
        assert!(output.contains(
            &vault_operator_delegation
                .delegation_state
                .enqueued_for_cooldown_amount
                .to_string()
        ));
        assert!(output.contains(
            &vault_operator_delegation
                .delegation_state
                .cooling_down_amount
                .to_string()
        ));
        assert!(output.contains(&vault_operator_delegation.last_update_slot.to_string()));
        assert!(output.contains(&vault_operator_delegation.index.to_string()));
        assert!(output.contains(&vault_operator_delegation.bump.to_string()));
    }
}
