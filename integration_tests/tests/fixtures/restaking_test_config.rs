use jito_restaking_core::{
    avs::Avs,
    avs_operator_list::AvsOperatorList,
    avs_slasher_list::AvsSlasherList,
    avs_vault_list::AvsVaultList,
    config::Config,
    operator::{Operator, OperatorAvsList, OperatorVaultList},
};
use solana_program::pubkey::Pubkey;
use solana_sdk::signature::{Keypair, Signer};

pub struct RestakingTestConfig {
    pub config: Pubkey,
    pub config_admin: Keypair,

    pub avs: Pubkey,
    pub avs_base: Keypair,
    pub avs_operator_list: Pubkey,
    pub avs_vault_list: Pubkey,
    pub avs_slasher_list: Pubkey,
    pub avs_admin: Keypair,

    pub operator: Pubkey,
    pub operator_base: Keypair,
    pub operator_avs_list: Pubkey,
    pub operator_vault_list: Pubkey,
    pub operator_admin: Keypair,
}

impl RestakingTestConfig {
    pub fn new_random() -> Self {
        let avs_base = Keypair::new();
        let avs = Avs::find_program_address(&jito_restaking_program::id(), &avs_base.pubkey()).0;
        let avs_operator_list =
            AvsOperatorList::find_program_address(&jito_restaking_program::id(), &avs).0;
        let avs_vault_list =
            AvsVaultList::find_program_address(&jito_restaking_program::id(), &avs).0;
        let avs_slasher_list =
            AvsSlasherList::find_program_address(&jito_restaking_program::id(), &avs).0;

        let node_operator_base = Keypair::new();
        let node_operator = Operator::find_program_address(
            &jito_restaking_program::id(),
            &node_operator_base.pubkey(),
        )
        .0;
        let node_operator_avs_list =
            OperatorAvsList::find_program_address(&jito_restaking_program::id(), &node_operator).0;
        let node_operator_vault_list =
            OperatorVaultList::find_program_address(&jito_restaking_program::id(), &node_operator)
                .0;
        Self {
            config: Config::find_program_address(&jito_restaking_program::id()).0,
            avs_base,
            avs,
            config_admin: Keypair::new(),
            avs_operator_list,
            avs_vault_list,
            avs_slasher_list,
            avs_admin: Keypair::new(),
            operator_base: node_operator_base,
            operator: node_operator,
            operator_avs_list: node_operator_avs_list,
            operator_vault_list: node_operator_vault_list,
            operator_admin: Keypair::new(),
        }
    }
}

impl Clone for RestakingTestConfig {
    fn clone(&self) -> Self {
        Self {
            config: self.config,
            avs_base: self.avs_base.insecure_clone(),
            avs: self.avs,
            config_admin: self.config_admin.insecure_clone(),
            avs_operator_list: self.avs_operator_list,
            avs_vault_list: self.avs_vault_list,
            avs_slasher_list: self.avs_slasher_list,
            avs_admin: self.avs_admin.insecure_clone(),
            operator: self.operator,
            operator_base: self.operator_base.insecure_clone(),
            operator_avs_list: self.operator_avs_list,
            operator_vault_list: self.operator_vault_list,
            operator_admin: self.operator_admin.insecure_clone(),
        }
    }
}
