use super::{account_header, field, section_header, PrettyDisplay};
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
