use jito_vault_core::{
    config::Config, vault::Vault, vault_avs_list::VaultAvsList,
    vault_operator_list::VaultOperatorList, vault_slasher_list::VaultSlasherList,
};
use solana_program::pubkey::Pubkey;
use solana_sdk::signature::{Keypair, Signer};

pub struct VaultTestConfig {
    pub config: Pubkey,
    pub config_admin: Keypair,
    pub vault_base: Keypair,
    pub vault: Pubkey,
    pub vault_avs_list: Pubkey,
    pub vault_operator_list: Pubkey,
    pub vault_slasher_list: Pubkey,
    pub lrt_mint: Keypair,
    pub token_mint: Keypair,
    pub vault_admin: Keypair,
    pub deposit_fee_bps: u16,
    pub withdrawal_fee_bps: u16,
}

impl Clone for VaultTestConfig {
    fn clone(&self) -> Self {
        Self {
            config: self.config,
            config_admin: self.config_admin.insecure_clone(),
            vault_base: self.vault_base.insecure_clone(),
            vault: self.vault,
            vault_avs_list: self.vault_avs_list,
            vault_operator_list: self.vault_operator_list,
            vault_slasher_list: self.vault_slasher_list,
            lrt_mint: self.lrt_mint.insecure_clone(),
            token_mint: self.token_mint.insecure_clone(),
            vault_admin: self.vault_admin.insecure_clone(),
            deposit_fee_bps: self.deposit_fee_bps,
            withdrawal_fee_bps: self.withdrawal_fee_bps,
        }
    }
}

impl VaultTestConfig {
    pub fn new_random() -> Self {
        let vault_base = Keypair::new();
        let vault = Vault::find_program_address(&jito_vault_program::id(), &vault_base.pubkey()).0;
        let vault_avs_list =
            VaultAvsList::find_program_address(&jito_vault_program::id(), &vault).0;
        let vault_operator_list =
            VaultOperatorList::find_program_address(&jito_vault_program::id(), &vault).0;
        let vault_slasher_list =
            VaultSlasherList::find_program_address(&jito_vault_program::id(), &vault).0;

        Self {
            config: Config::find_program_address(&jito_vault_program::id()).0,
            config_admin: Keypair::new(),
            vault_base,
            vault,
            vault_avs_list,
            vault_operator_list,
            vault_slasher_list,
            lrt_mint: Keypair::new(),
            token_mint: Keypair::new(),
            vault_admin: Keypair::new(),
            deposit_fee_bps: 100,
            withdrawal_fee_bps: 100,
        }
    }
}
