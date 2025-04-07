use jito_restaking_client_common::log::{account_header, field, section_header, PrettyDisplay};

use crate::accounts::NcnOperatorState;

impl PrettyDisplay for NcnOperatorState {
    fn pretty_display(&self) -> String {
        let mut output = String::new();

        output.push_str(&account_header("Ncn Operator State Account"));

        output.push_str(&section_header("Basic Information"));
        output.push_str(&field("NCN", self.ncn));
        output.push_str(&field("Operator", self.operator));
        output.push_str(&field("Index", self.index));
        output.push_str(&field("Bump", self.bump));

        output.push_str(&section_header("NCN State"));
        output.push_str(&field("NCN Opt-In Added", self.ncn_opt_in_state.slot_added));
        output.push_str(&field(
            "NCN Opt-In Removed",
            self.ncn_opt_in_state.slot_removed,
        ));

        output.push_str(&section_header("Operator State"));
        output.push_str(&field(
            "Operator Opt-In Added",
            self.operator_opt_in_state.slot_added,
        ));
        output.push_str(&field(
            "Operator Opt-In Removed",
            self.operator_opt_in_state.slot_removed,
        ));

        output
    }
}

#[cfg(test)]
mod tests {
    use anchor_lang::prelude::Pubkey;
    use jito_restaking_client_common::log::PrettyDisplay;

    use crate::{accounts::NcnOperatorState, types::SlotToggle};

    #[test]
    fn test_ncn_operator_state_pretty_display_structure() {
        let ncn_operator_state = NcnOperatorState {
            discriminator: 12345,
            ncn: Pubkey::new_unique(),
            operator: Pubkey::new_unique(),
            index: 0,
            ncn_opt_in_state: SlotToggle {
                slot_added: 0,
                slot_removed: 1,
                reserved: [0; 32],
            },
            operator_opt_in_state: SlotToggle {
                slot_added: 0,
                slot_removed: 1,
                reserved: [0; 32],
            },
            bump: 254,
            reserved: [0; 263],
        };

        let output = ncn_operator_state.pretty_display();

        assert!(output.contains(&ncn_operator_state.ncn.to_string()));
        assert!(output.contains(&ncn_operator_state.operator.to_string()));
        assert!(output.contains(&ncn_operator_state.index.to_string()));
        assert!(output.contains(&ncn_operator_state.ncn_opt_in_state.slot_added.to_string()));
        assert!(output.contains(&ncn_operator_state.ncn_opt_in_state.slot_removed.to_string()));
        assert!(output.contains(
            &ncn_operator_state
                .operator_opt_in_state
                .slot_added
                .to_string()
        ));
        assert!(output.contains(
            &ncn_operator_state
                .operator_opt_in_state
                .slot_removed
                .to_string()
        ));
        assert!(output.contains(&ncn_operator_state.bump.to_string()));
    }
}
