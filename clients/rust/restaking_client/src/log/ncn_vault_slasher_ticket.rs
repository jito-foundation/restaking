use jito_restaking_client_common::log::{account_header, field, section_header, PrettyDisplay};

use crate::accounts::NcnVaultSlasherTicket;

impl PrettyDisplay for NcnVaultSlasherTicket {
    fn pretty_display(&self) -> String {
        let mut output = String::new();

        output.push_str(&account_header("Ncn Vault Slasher Ticket Account"));

        output.push_str(&section_header("Basic Information"));
        output.push_str(&field("NCN", self.ncn));
        output.push_str(&field("Vault", self.vault));
        output.push_str(&field("Slasher", self.slasher));
        output.push_str(&field(
            "Max Slashable Per Epoch",
            self.max_slashable_per_epoch,
        ));
        output.push_str(&field("Index", self.index));
        output.push_str(&field("Bump", self.bump));

        output.push_str(&section_header("State"));
        output.push_str(&field("Opt-In Added", self.state.slot_added));
        output.push_str(&field("Opt-In Removed", self.state.slot_removed));

        output
    }
}

#[cfg(test)]
mod tests {
    use anchor_lang::prelude::Pubkey;
    use jito_restaking_client_common::log::PrettyDisplay;

    use crate::{accounts::NcnVaultSlasherTicket, types::SlotToggle};

    #[test]
    fn test_ncn_vault_slasher_ticket_pretty_display_structure() {
        let ncn_vault_slasher_ticket = NcnVaultSlasherTicket {
            discriminator: 12345,
            ncn: Pubkey::new_unique(),
            vault: Pubkey::new_unique(),
            slasher: Pubkey::new_unique(),
            max_slashable_per_epoch: 0,
            index: 1,
            state: SlotToggle {
                slot_added: 0,
                slot_removed: 1,
                reserved: [0; 32],
            },
            bump: 2,
            reserved: [0; 263],
        };

        let output = ncn_vault_slasher_ticket.pretty_display();

        assert!(output.contains(&ncn_vault_slasher_ticket.ncn.to_string()));
        assert!(output.contains(&ncn_vault_slasher_ticket.vault.to_string()));
        assert!(output.contains(&ncn_vault_slasher_ticket.slasher.to_string()));
        assert!(output.contains(&ncn_vault_slasher_ticket.max_slashable_per_epoch.to_string()));
        assert!(output.contains(&ncn_vault_slasher_ticket.index.to_string()));
        assert!(output.contains(&ncn_vault_slasher_ticket.state.slot_added.to_string()));
        assert!(output.contains(&ncn_vault_slasher_ticket.state.slot_removed.to_string()));
        assert!(output.contains(&ncn_vault_slasher_ticket.bump.to_string()));
    }
}
