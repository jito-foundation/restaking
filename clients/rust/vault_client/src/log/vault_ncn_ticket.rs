use jito_restaking_client_common::log::{account_header, field, section_header, PrettyDisplay};

use crate::accounts::VaultNcnTicket;

impl PrettyDisplay for VaultNcnTicket {
    fn pretty_display(&self) -> String {
        let mut output = String::new();

        output.push_str(&account_header("Vault NCN Ticket Account"));

        output.push_str(&section_header("Basic Information"));
        output.push_str(&field("Vault", self.vault));
        output.push_str(&field("NCN", self.ncn));
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
    use jito_restaking_client_common::log::PrettyDisplay;

    use crate::{accounts::VaultNcnTicket, types::SlotToggle};

    #[test]
    fn test_vault_ncn_ticket_pretty_display_structure() {
        let vault_ncn_ticket = VaultNcnTicket {
            discriminator: 12345,
            vault: Pubkey::new_unique(),
            ncn: Pubkey::new_unique(),
            index: 0,
            state: SlotToggle {
                slot_added: 1,
                slot_removed: 2,
                reserved: [0; 32],
            },
            bump: 3,
            reserved: [0; 263],
        };

        let output = vault_ncn_ticket.pretty_display();

        assert!(output.contains(&vault_ncn_ticket.vault.to_string()));
        assert!(output.contains(&vault_ncn_ticket.ncn.to_string()));
        assert!(output.contains(&vault_ncn_ticket.index.to_string()));
        assert!(output.contains(&vault_ncn_ticket.state.slot_added.to_string()));
        assert!(output.contains(&vault_ncn_ticket.state.slot_removed.to_string()));
        assert!(output.contains(&vault_ncn_ticket.bump.to_string()));
    }
}
