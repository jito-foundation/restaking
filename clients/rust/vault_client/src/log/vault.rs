use crate::accounts::Vault;

use super::{account_header, field, section_header, PrettyDisplay};

impl PrettyDisplay for Vault {
    fn pretty_display(&self) -> String {
        let mut output = String::new();

        output.push_str(&account_header("Vault Account"));

        output.push_str(&section_header("Basic Information"));
        output.push_str(&field("Base", self.base));
        output.push_str(&field("Vault Index", self.vault_index));
        output.push_str(&field("Bump", self.bump));
        output.push_str(&field("Is Paused", self.is_paused));

        output.push_str(&section_header("Token Information"));
        output.push_str(&field("VRT Mint", self.vrt_mint));
        output.push_str(&field("Supported Mint", self.supported_mint));
        output.push_str(&field("VRT Supply", self.vrt_supply));
        output.push_str(&field("Tokens Deposited", self.tokens_deposited));
        output.push_str(&field("Deposit Capacity", self.deposit_capacity));

        output.push_str(&section_header("Accounting"));
        output.push_str(&field("Staked Amount", self.delegation_state.staked_amount));
        output.push_str(&field(
            "Cooling Down Amount",
            self.delegation_state.cooling_down_amount,
        ));
        output.push_str(&field(
            "Enqueued for Cooldown Amount",
            self.delegation_state.enqueued_for_cooldown_amount,
        ));
        output.push_str(&field(
            "Additional Assets Need Unstaking",
            self.additional_assets_need_unstaking,
        ));
        output.push_str(&field(
            "VRT Enqueued for Cooldown Amount",
            self.vrt_enqueued_for_cooldown_amount,
        ));
        output.push_str(&field(
            "VRT Cooling Down Amount",
            self.vrt_cooling_down_amount,
        ));
        output.push_str(&field(
            "VRT Ready To Claim Amount",
            self.vrt_ready_to_claim_amount,
        ));

        output.push_str(&section_header("Admin Authorities"));
        output.push_str(&field("Admin", self.admin));
        output.push_str(&field("Delegate Admin", self.delegation_admin));
        output.push_str(&field("Operator Admin", self.operator_admin));
        output.push_str(&field("NCN Admin", self.ncn_admin));
        output.push_str(&field("Slasher Admin", self.slasher_admin));
        output.push_str(&field("Capacity Admin", self.capacity_admin));
        output.push_str(&field("Fee Admin", self.fee_admin));
        output.push_str(&field("Delegate Asset Admin", self.delegate_asset_admin));
        output.push_str(&field("Fee Wallet", self.fee_wallet));
        output.push_str(&field("Mint Burn Admin", self.mint_burn_admin));
        output.push_str(&field("Metadata Admin", self.metadata_admin));

        output.push_str(&section_header("Statistics"));
        output.push_str(&field("NCN Count", self.ncn_count));
        output.push_str(&field("Operator Count", self.operator_count));
        output.push_str(&field("Slasher Count", self.slasher_count));
        output.push_str(&field("Last Fee Change Slot", self.last_fee_change_slot));
        output.push_str(&field(
            "Last Start State Update Slot",
            self.last_start_state_update_slot,
        ));
        output.push_str(&field(
            "Last Full State Update Slot",
            self.last_full_state_update_slot,
        ));
        output.push_str(&field("Withdrawal Fee BPS", self.withdrawal_fee_bps));
        output.push_str(&field(
            "Next Withdrawal Fee BPS",
            self.next_withdrawal_fee_bps,
        ));
        output.push_str(&field("Reward Fee BPS", self.reward_fee_bps));
        output.push_str(&field("Program Fee BPS", self.program_fee_bps));

        output
    }
}
