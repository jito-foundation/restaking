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
