use jito_restaking_client_common::log::{account_header, field, section_header, PrettyDisplay};

use crate::accounts::Vault;

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
        output.push_str(&field("Deposit Fee BPS", self.deposit_fee_bps));
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

#[cfg(test)]
mod tests {
    use anchor_lang::prelude::Pubkey;
    use jito_restaking_client_common::log::PrettyDisplay;

    use crate::{accounts::Vault, types::DelegationState};

    #[test]
    fn test_vault_pretty_display_structure() {
        let vault = Vault {
            discriminator: 12345,
            base: Pubkey::new_unique(),
            vrt_mint: Pubkey::new_unique(),
            supported_mint: Pubkey::new_unique(),
            vrt_supply: 1,
            tokens_deposited: 2,
            deposit_capacity: 3,
            delegation_state: DelegationState {
                staked_amount: 4,
                enqueued_for_cooldown_amount: 5,
                cooling_down_amount: 6,
                reserved: [0; 256],
            },
            additional_assets_need_unstaking: 7,
            vrt_enqueued_for_cooldown_amount: 8,
            vrt_cooling_down_amount: 9,
            vrt_ready_to_claim_amount: 10,
            admin: Pubkey::new_unique(),
            delegation_admin: Pubkey::new_unique(),
            operator_admin: Pubkey::new_unique(),
            ncn_admin: Pubkey::new_unique(),
            slasher_admin: Pubkey::new_unique(),
            capacity_admin: Pubkey::new_unique(),
            fee_admin: Pubkey::new_unique(),
            delegate_asset_admin: Pubkey::new_unique(),
            fee_wallet: Pubkey::new_unique(),
            mint_burn_admin: Pubkey::new_unique(),
            metadata_admin: Pubkey::new_unique(),
            vault_index: 11,
            ncn_count: 12,
            operator_count: 13,
            slasher_count: 14,
            last_fee_change_slot: 15,
            last_full_state_update_slot: 16,
            deposit_fee_bps: 17,
            withdrawal_fee_bps: 18,
            next_withdrawal_fee_bps: 19,
            reward_fee_bps: 20,
            program_fee_bps: 21,
            bump: 22,
            is_paused: false,
            last_start_state_update_slot: 23,
            reserved: [0; 251],
        };

        let output = vault.pretty_display();

        assert!(output.contains(&vault.base.to_string()));
        assert!(output.contains(&vault.vrt_mint.to_string()));
        assert!(output.contains(&vault.supported_mint.to_string()));
        assert!(output.contains(&vault.vrt_supply.to_string()));
        assert!(output.contains(&vault.tokens_deposited.to_string()));
        assert!(output.contains(&vault.deposit_capacity.to_string()));
        assert!(output.contains(&vault.delegation_state.staked_amount.to_string()));
        assert!(output.contains(
            &vault
                .delegation_state
                .enqueued_for_cooldown_amount
                .to_string()
        ));
        assert!(output.contains(&vault.delegation_state.cooling_down_amount.to_string()));
        assert!(output.contains(&vault.additional_assets_need_unstaking.to_string()));
        assert!(output.contains(&vault.vrt_enqueued_for_cooldown_amount.to_string()));
        assert!(output.contains(&vault.vrt_cooling_down_amount.to_string()));
        assert!(output.contains(&vault.vrt_ready_to_claim_amount.to_string()));
        assert!(output.contains(&vault.admin.to_string()));
        assert!(output.contains(&vault.delegation_admin.to_string()));
        assert!(output.contains(&vault.operator_admin.to_string()));
        assert!(output.contains(&vault.ncn_admin.to_string()));
        assert!(output.contains(&vault.slasher_admin.to_string()));
        assert!(output.contains(&vault.capacity_admin.to_string()));
        assert!(output.contains(&vault.fee_admin.to_string()));
        assert!(output.contains(&vault.delegate_asset_admin.to_string()));
        assert!(output.contains(&vault.fee_wallet.to_string()));
        assert!(output.contains(&vault.mint_burn_admin.to_string()));
        assert!(output.contains(&vault.metadata_admin.to_string()));

        assert!(output.contains(&vault.vault_index.to_string()));
        assert!(output.contains(&vault.ncn_count.to_string()));
        assert!(output.contains(&vault.operator_count.to_string()));
        assert!(output.contains(&vault.slasher_count.to_string()));

        assert!(output.contains(&vault.deposit_fee_bps.to_string()));
        assert!(output.contains(&vault.withdrawal_fee_bps.to_string()));
        assert!(output.contains(&vault.next_withdrawal_fee_bps.to_string()));
        assert!(output.contains(&vault.reward_fee_bps.to_string()));
        assert!(output.contains(&vault.program_fee_bps.to_string()));
        assert!(output.contains(&vault.last_fee_change_slot.to_string()));
        assert!(output.contains(&vault.last_full_state_update_slot.to_string()));
        assert!(output.contains(&vault.last_start_state_update_slot.to_string()));
        assert!(output.contains(&vault.bump.to_string()));
        assert!(output.contains("false")); // is_paused value
    }
}
