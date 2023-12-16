use jito_lrt_core::{
    config::Config, vault::Vault, vault_avs_list::VaultAvsList,
    vault_operator_list::VaultOperatorList,
};
use solana_program::pubkey::Pubkey;
use solana_sdk::signature::{Keypair, Signer};

pub struct LrtTestConfig {
    pub config: Pubkey,
    pub config_admin: Keypair,
    pub restaking_program_signer: Pubkey,
    pub vault_base: Keypair,
    pub vault: Pubkey,
    pub vault_avs_list: Pubkey,
    pub vault_operator_list: Pubkey,
    pub lrt_mint: Keypair,
    pub token_mint: Keypair,
    pub vault_admin: Keypair,
}

impl Clone for LrtTestConfig {
    fn clone(&self) -> Self {
        Self {
            config: self.config,
            config_admin: self.config_admin.insecure_clone(),
            restaking_program_signer: self.restaking_program_signer,
            vault_base: self.vault_base.insecure_clone(),
            vault: self.vault,
            vault_avs_list: self.vault_avs_list,
            vault_operator_list: self.vault_operator_list,
            lrt_mint: self.lrt_mint.insecure_clone(),
            token_mint: self.token_mint.insecure_clone(),
            vault_admin: self.vault_admin.insecure_clone(),
        }
    }
}

impl LrtTestConfig {
    pub fn new_random(restaking_program_signer: Pubkey) -> Self {
        let vault_base = Keypair::new();
        let vault = Vault::find_program_address(&jito_lrt_program::id(), &vault_base.pubkey()).0;
        let vault_avs_list = VaultAvsList::find_program_address(&jito_lrt_program::id(), &vault).0;
        let vault_operator_list =
            VaultOperatorList::find_program_address(&jito_lrt_program::id(), &vault).0;

        Self {
            config: Config::find_program_address(&jito_lrt_program::id()).0,
            config_admin: Keypair::new(),
            restaking_program_signer,
            vault_base,
            vault,
            vault_avs_list,
            vault_operator_list,
            lrt_mint: Keypair::new(),
            token_mint: Keypair::new(),
            vault_admin: Keypair::new(),
        }
    }
}
