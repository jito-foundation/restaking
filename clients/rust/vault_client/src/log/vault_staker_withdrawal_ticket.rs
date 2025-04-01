use super::{account_header, field, section_header, PrettyDisplay};
use crate::accounts::VaultStakerWithdrawalTicket;

impl PrettyDisplay for VaultStakerWithdrawalTicket {
    fn pretty_display(&self) -> String {
        let mut output = String::new();

        output.push_str(&account_header("Vault Staker Withdrawal Ticket Account"));

        output.push_str(&section_header("Basic Information"));
        output.push_str(&field("Vault", self.vault));
        output.push_str(&field("Staker", self.staker));
        output.push_str(&field("Base", self.base));
        output.push_str(&field("VRT Amount", self.vrt_amount));
        output.push_str(&field("Slot Unstaked", self.slot_unstaked));
        output.push_str(&field("Bump", self.bump));

        output
    }
}

#[cfg(test)]
mod tests {
    use anchor_lang::prelude::Pubkey;

    use crate::{accounts::VaultStakerWithdrawalTicket, log::PrettyDisplay};

    #[test]
    fn test_vault_staker_withdrawal_ticket_pretty_display_structure() {
        let vault_staker_withdrawal_ticket = VaultStakerWithdrawalTicket {
            discriminator: 12345,
            vault: Pubkey::new_unique(),
            staker: Pubkey::new_unique(),
            base: Pubkey::new_unique(),
            vrt_amount: 0,
            slot_unstaked: 1,
            bump: 2,
            reserved: [0; 263],
        };

        let output = vault_staker_withdrawal_ticket.pretty_display();

        assert!(output.contains(&vault_staker_withdrawal_ticket.vault.to_string()));
        assert!(output.contains(&vault_staker_withdrawal_ticket.staker.to_string()));
        assert!(output.contains(&vault_staker_withdrawal_ticket.base.to_string()));
        assert!(output.contains(&vault_staker_withdrawal_ticket.vrt_amount.to_string()));
        assert!(output.contains(&vault_staker_withdrawal_ticket.slot_unstaked.to_string()));
        assert!(output.contains(&vault_staker_withdrawal_ticket.bump.to_string()));
    }
}
