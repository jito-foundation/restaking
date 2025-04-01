use super::{account_header, field, section_header, PrettyDisplay};
use crate::accounts::VaultNcnSlasherTicket;

impl PrettyDisplay for VaultNcnSlasherTicket {
    fn pretty_display(&self) -> String {
        let mut output = String::new();

        output.push_str(&account_header("Vault NCN Slasher Ticket Account"));

        output.push_str(&section_header("Basic Information"));
        output.push_str(&field("Vault", self.vault));
        output.push_str(&field("NCN", self.ncn));
        output.push_str(&field("Slasher", self.slasher));
        output.push_str(&field(
            "Max Slashable per Epoch",
            self.max_slashable_per_epoch,
        ));
        output.push_str(&field("Index", self.index));
        output.push_str(&field("Bump", self.bump));

        output.push_str(&section_header("State"));
        output.push_str(&field("Slot Added", self.state.slot_added));
        output.push_str(&field("Slot Removed", self.state.slot_removed));

        output
    }
}

#[cfg(test)]
mod tests {
    use anchor_lang::prelude::Pubkey;

    use crate::{accounts::VaultNcnSlasherTicket, log::PrettyDisplay, types::SlotToggle};

    #[test]
    fn test_vault_ncn_slasher_ticket_pretty_display_structure() {
        let vault_ncn_slasher_ticket = VaultNcnSlasherTicket {
            discriminator: 12345,
            vault: Pubkey::new_unique(),
            ncn: Pubkey::new_unique(),
            slasher: Pubkey::new_unique(),
            max_slashable_per_epoch: 1,
            index: 2,
            state: SlotToggle {
                slot_added: 3,
                slot_removed: 4,
                reserved: [0; 32],
            },
            bump: 5,
            reserved: [0; 263],
        };

        let output = vault_ncn_slasher_ticket.pretty_display();

        assert!(output.contains(&vault_ncn_slasher_ticket.vault.to_string()));
        assert!(output.contains(&vault_ncn_slasher_ticket.ncn.to_string()));
        assert!(output.contains(&vault_ncn_slasher_ticket.slasher.to_string()));
        assert!(output.contains(&vault_ncn_slasher_ticket.max_slashable_per_epoch.to_string()));
        assert!(output.contains(&vault_ncn_slasher_ticket.index.to_string()));
        assert!(output.contains(&vault_ncn_slasher_ticket.state.slot_added.to_string()));
        assert!(output.contains(&vault_ncn_slasher_ticket.state.slot_removed.to_string()));
        assert!(output.contains(&vault_ncn_slasher_ticket.bump.to_string()));
    }
}
