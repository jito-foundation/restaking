use jito_restaking_client_common::log::{account_header, field, section_header, PrettyDisplay};

use crate::accounts::VaultNcnSlasherOperatorTicket;

impl PrettyDisplay for VaultNcnSlasherOperatorTicket {
    fn pretty_display(&self) -> String {
        let mut output = String::new();

        output.push_str(&account_header("Vault NCN Slasher Operator Ticket Account"));

        output.push_str(&section_header("Basic Information"));
        output.push_str(&field("Vault", self.vault));
        output.push_str(&field("NCN", self.ncn));
        output.push_str(&field("Slasher", self.slasher));
        output.push_str(&field("Operator", self.operator));
        output.push_str(&field("Epoch", self.epoch));
        output.push_str(&field("Slashed", self.slashed));
        output.push_str(&field("Bump", self.bump));

        output
    }
}

#[cfg(test)]
mod tests {
    use anchor_lang::prelude::Pubkey;
    use jito_restaking_client_common::log::PrettyDisplay;

    use crate::accounts::VaultNcnSlasherOperatorTicket;

    #[test]
    fn test_vault_ncn_slasher_operator_ticket_pretty_display_structure() {
        let vault_ncn_slasher_operator_ticket = VaultNcnSlasherOperatorTicket {
            discriminator: 12345,
            vault: Pubkey::new_unique(),
            ncn: Pubkey::new_unique(),
            slasher: Pubkey::new_unique(),
            operator: Pubkey::new_unique(),
            epoch: 0,
            slashed: 1,
            bump: 2,
            reserved: [0; 263],
        };

        let output = vault_ncn_slasher_operator_ticket.pretty_display();

        assert!(output.contains(&vault_ncn_slasher_operator_ticket.vault.to_string()));
        assert!(output.contains(&vault_ncn_slasher_operator_ticket.ncn.to_string()));
        assert!(output.contains(&vault_ncn_slasher_operator_ticket.slasher.to_string()));
        assert!(output.contains(&vault_ncn_slasher_operator_ticket.operator.to_string()));
        assert!(output.contains(&vault_ncn_slasher_operator_ticket.epoch.to_string()));
        assert!(output.contains(&vault_ncn_slasher_operator_ticket.slashed.to_string()));
        assert!(output.contains(&vault_ncn_slasher_operator_ticket.bump.to_string()));
    }
}
