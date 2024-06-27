use borsh::BorshDeserialize;
use jito_restaking_core::{
    avs::Avs,
    avs_operator_list::AvsOperatorList,
    avs_vault_list::AvsVaultList,
    config::Config,
    node_operator::{NodeOperator, NodeOperatorAvsList, OperatorVaultList},
};
use jito_restaking_sdk::{
    avs_add_vault, avs_remove_vault, initialize_avs, initialize_config, initialize_operator,
    operator_add_vault, operator_remove_vault,
};
use solana_program::pubkey::Pubkey;
use solana_program_test::{BanksClient, BanksClientError};
use solana_sdk::{commitment_config::CommitmentLevel, signature::Signer, transaction::Transaction};

use crate::fixtures::{
    restaking_test_config::RestakingTestConfig, vault_test_config::VaultTestConfig,
};

pub struct RestakingProgramClient {
    banks_client: BanksClient,
}

impl RestakingProgramClient {
    pub fn new(banks_client: BanksClient) -> Self {
        Self { banks_client }
    }

    pub async fn get_avs(&mut self, avs: &Pubkey) -> Result<Avs, BanksClientError> {
        let account = self.banks_client.get_account(*avs).await?.unwrap();

        Ok(Avs::deserialize(&mut account.data.as_slice())?)
    }

    pub async fn get_config(&mut self, account: &Pubkey) -> Result<Config, BanksClientError> {
        let account = self.banks_client.get_account(*account).await?.unwrap();
        Ok(Config::deserialize(&mut account.data.as_slice())?)
    }

    pub async fn get_avs_vault_list(
        &mut self,
        account: &Pubkey,
    ) -> Result<AvsVaultList, BanksClientError> {
        let account = self.banks_client.get_account(*account).await?.unwrap();
        Ok(AvsVaultList::deserialize(&mut account.data.as_slice())?)
    }

    pub async fn get_avs_operator_list(
        &mut self,
        account: &Pubkey,
    ) -> Result<AvsOperatorList, BanksClientError> {
        let account = self.banks_client.get_account(*account).await?.unwrap();
        Ok(AvsOperatorList::deserialize(&mut account.data.as_slice())?)
    }

    pub async fn get_operator(
        &mut self,
        account: &Pubkey,
    ) -> Result<NodeOperator, BanksClientError> {
        let account = self.banks_client.get_account(*account).await?.unwrap();
        Ok(NodeOperator::deserialize(&mut account.data.as_slice())?)
    }

    pub async fn get_operator_vault_list(
        &mut self,
        account: &Pubkey,
    ) -> Result<OperatorVaultList, BanksClientError> {
        let account = self.banks_client.get_account(*account).await?.unwrap();
        Ok(OperatorVaultList::deserialize(
            &mut account.data.as_slice(),
        )?)
    }

    pub async fn get_operator_avs_list(
        &mut self,
        account: &Pubkey,
    ) -> Result<NodeOperatorAvsList, BanksClientError> {
        let account = self.banks_client.get_account(*account).await?.unwrap();
        Ok(NodeOperatorAvsList::deserialize(
            &mut account.data.as_slice(),
        )?)
    }

    pub async fn initialize_config(
        &mut self,
        restaking_test_config: &RestakingTestConfig,
    ) -> Result<(), BanksClientError> {
        let blockhash = self.banks_client.get_latest_blockhash().await?;
        self.process_transaction(&Transaction::new_signed_with_payer(
            &[initialize_config(
                &jito_restaking_program::id(),
                &restaking_test_config.config,
                &restaking_test_config.config_admin.pubkey(),
                &jito_vault_program::id(),
            )],
            Some(&restaking_test_config.config_admin.pubkey()),
            &[&restaking_test_config.config_admin],
            blockhash,
        ))
        .await
    }

    pub async fn initialize_avs(
        &mut self,
        restaking_test_config: &RestakingTestConfig,
    ) -> Result<(), BanksClientError> {
        let blockhash = self.banks_client.get_latest_blockhash().await?;

        self.process_transaction(&Transaction::new_signed_with_payer(
            &[initialize_avs(
                &jito_restaking_program::id(),
                &restaking_test_config.config,
                &restaking_test_config.avs,
                &restaking_test_config.avs_operator_list,
                &restaking_test_config.avs_vault_list,
                &restaking_test_config.avs_slasher_list,
                &restaking_test_config.avs_admin.pubkey(),
                &restaking_test_config.avs_base.pubkey(),
            )],
            Some(&restaking_test_config.avs_admin.pubkey()),
            &[
                &restaking_test_config.avs_admin,
                &restaking_test_config.avs_base,
            ],
            blockhash,
        ))
        .await
    }

    pub async fn avs_add_vault(
        &mut self,
        restaking_test_config: &RestakingTestConfig,
        vault_test_config: &VaultTestConfig,
    ) -> Result<(), BanksClientError> {
        let blockhash = self.banks_client.get_latest_blockhash().await?;

        self.process_transaction(&Transaction::new_signed_with_payer(
            &[avs_add_vault(
                &jito_restaking_program::id(),
                &restaking_test_config.config,
                &restaking_test_config.avs,
                &restaking_test_config.avs_vault_list,
                &restaking_test_config.avs_admin.pubkey(),
                &jito_vault_program::id(),
                &vault_test_config.vault,
                &vault_test_config.config,
                &vault_test_config.vault_avs_list,
                &restaking_test_config.avs_admin.pubkey(),
            )],
            Some(&restaking_test_config.avs_admin.pubkey()),
            &[&restaking_test_config.avs_admin],
            blockhash,
        ))
        .await
    }

    pub async fn avs_remove_vault(
        &mut self,
        restaking_test_config: &RestakingTestConfig,
        vault_test_config: &VaultTestConfig,
    ) -> Result<(), BanksClientError> {
        let blockhash = self.banks_client.get_latest_blockhash().await?;

        self.process_transaction(&Transaction::new_signed_with_payer(
            &[avs_remove_vault(
                &jito_restaking_program::id(),
                &restaking_test_config.config,
                &restaking_test_config.avs,
                &restaking_test_config.avs_vault_list,
                &restaking_test_config.avs_admin.pubkey(),
                &jito_vault_program::id(),
                &vault_test_config.vault,
                &vault_test_config.config,
                &vault_test_config.vault_avs_list,
                &restaking_test_config.avs_admin.pubkey(),
            )],
            Some(&restaking_test_config.avs_admin.pubkey()),
            &[&restaking_test_config.avs_admin],
            blockhash,
        ))
        .await
    }

    pub async fn initialize_operator(
        &mut self,
        restaking_test_config: &RestakingTestConfig,
    ) -> Result<(), BanksClientError> {
        let blockhash = self.banks_client.get_latest_blockhash().await?;

        self.process_transaction(&Transaction::new_signed_with_payer(
            &[initialize_operator(
                &jito_restaking_program::id(),
                &restaking_test_config.config,
                &restaking_test_config.operator,
                &restaking_test_config.operator_avs_list,
                &restaking_test_config.operator_vault_list,
                &restaking_test_config.operator_admin.pubkey(),
                &restaking_test_config.operator_base.pubkey(),
            )],
            Some(&restaking_test_config.operator_admin.pubkey()),
            &[
                &restaking_test_config.operator_admin,
                &restaking_test_config.operator_base,
            ],
            blockhash,
        ))
        .await
    }

    pub async fn operator_add_vault(
        &mut self,
        restaking_test_config: &RestakingTestConfig,
        vault_test_config: &VaultTestConfig,
    ) -> Result<(), BanksClientError> {
        let blockhash = self.banks_client.get_latest_blockhash().await?;

        self.process_transaction(&Transaction::new_signed_with_payer(
            &[operator_add_vault(
                &jito_restaking_program::id(),
                &restaking_test_config.config,
                &restaking_test_config.operator,
                &restaking_test_config.operator_vault_list,
                &restaking_test_config.operator_admin.pubkey(),
                &jito_vault_program::id(),
                &vault_test_config.vault,
                &vault_test_config.config,
                &vault_test_config.vault_operator_list,
                &restaking_test_config.operator_admin.pubkey(),
            )],
            Some(&restaking_test_config.operator_admin.pubkey()),
            &[&restaking_test_config.operator_admin],
            blockhash,
        ))
        .await
    }

    pub async fn operator_remove_vault(
        &mut self,
        restaking_test_config: &RestakingTestConfig,
        vault_test_config: &VaultTestConfig,
    ) -> Result<(), BanksClientError> {
        let blockhash = self.banks_client.get_latest_blockhash().await?;

        self.process_transaction(&Transaction::new_signed_with_payer(
            &[operator_remove_vault(
                &jito_restaking_program::id(),
                &restaking_test_config.config,
                &restaking_test_config.operator,
                &restaking_test_config.operator_vault_list,
                &restaking_test_config.avs_admin.pubkey(),
                &jito_vault_program::id(),
                &vault_test_config.vault,
                &vault_test_config.config,
                &vault_test_config.vault_operator_list,
                &restaking_test_config.operator_admin.pubkey(),
            )],
            Some(&restaking_test_config.operator_admin.pubkey()),
            &[&restaking_test_config.operator_admin],
            blockhash,
        ))
        .await
    }

    pub async fn process_transaction(&mut self, tx: &Transaction) -> Result<(), BanksClientError> {
        Ok(self
            .banks_client
            .process_transaction_with_preflight_and_commitment(
                tx.clone(),
                CommitmentLevel::Processed,
            )
            .await?)
    }
}
