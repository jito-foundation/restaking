use borsh::BorshDeserialize;
use jito_lrt_core::{
    config::Config, vault::Vault, vault_avs_list::VaultAvsList,
    vault_operator_list::VaultOperatorList,
};
use jito_lrt_sdk::{
    add_delegation, initialize_config, initialize_vault, mint_to, remove_delegation,
};
use solana_program::pubkey::Pubkey;
use solana_program_test::{BanksClient, BanksClientError};
use solana_sdk::{
    commitment_config::CommitmentLevel,
    signature::{Keypair, Signer},
    transaction::Transaction,
};
use spl_associated_token_account::instruction::create_associated_token_account_idempotent;

use crate::fixtures::{lrt_test_config::LrtTestConfig, restaking_test_config::RestakingTestConfig};

pub struct LrtProgramClient {
    banks_client: BanksClient,
}

impl LrtProgramClient {
    pub fn new(banks_client: BanksClient) -> Self {
        Self { banks_client }
    }

    pub async fn get_config(&mut self, account: &Pubkey) -> Result<Config, BanksClientError> {
        let account = self.banks_client.get_account(*account).await?.unwrap();
        Ok(Config::deserialize(&mut account.data.as_slice())?)
    }

    pub async fn get_vault(&mut self, account: &Pubkey) -> Result<Vault, BanksClientError> {
        let account = self.banks_client.get_account(*account).await?.unwrap();
        Ok(Vault::deserialize(&mut account.data.as_slice())?)
    }

    pub async fn get_vault_avs_list(
        &mut self,
        account: &Pubkey,
    ) -> Result<VaultAvsList, BanksClientError> {
        let account = self.banks_client.get_account(*account).await?.unwrap();
        Ok(VaultAvsList::deserialize(&mut account.data.as_slice())?)
    }

    pub async fn get_vault_operator_list(
        &mut self,
        account: &Pubkey,
    ) -> Result<VaultOperatorList, BanksClientError> {
        let account = self.banks_client.get_account(*account).await?.unwrap();
        Ok(VaultOperatorList::deserialize(
            &mut account.data.as_slice(),
        )?)
    }

    pub async fn initialize_config(
        &mut self,
        config: &LrtTestConfig,
    ) -> Result<(), BanksClientError> {
        let blockhash = self.banks_client.get_latest_blockhash().await?;
        self.process_transaction(&Transaction::new_signed_with_payer(
            &[initialize_config(
                &jito_lrt_program::id(),
                &config.config,
                &config.config_admin.pubkey(),
                &config.restaking_program_signer,
            )],
            Some(&config.config_admin.pubkey()),
            &[&config.config_admin],
            blockhash,
        ))
        .await
    }

    pub async fn initialize_vault(
        &mut self,
        config: &LrtTestConfig,
    ) -> Result<(), BanksClientError> {
        let blockhash = self.banks_client.get_latest_blockhash().await?;

        self.process_transaction(&Transaction::new_signed_with_payer(
            &[
                initialize_vault(
                    &jito_lrt_program::id(),
                    &config.config,
                    &config.vault,
                    &config.vault_avs_list,
                    &config.vault_operator_list,
                    &config.lrt_mint.pubkey(),
                    &config.token_mint.pubkey(),
                    &config.vault_admin.pubkey(),
                    &config.vault_base.pubkey(),
                ),
                // crate an ATA for the collateral backing tokens
                create_associated_token_account_idempotent(
                    &config.vault_admin.pubkey(),
                    &config.vault,
                    &config.token_mint.pubkey(),
                    &spl_token::id(),
                ),
            ],
            Some(&config.vault_admin.pubkey()),
            &[&config.vault_admin, &config.lrt_mint, &config.vault_base],
            blockhash,
        ))
        .await
    }

    pub async fn mint(
        &mut self,
        lrt: &Pubkey,
        lrt_mint: &Pubkey,
        source_owner: &Pubkey,
        source_token_account: &Pubkey,
        dest_token_account: &Pubkey,
        lrt_receiver: &Pubkey,
        amount: u64,
        signer: &Keypair,
    ) -> Result<(), BanksClientError> {
        let blockhash = self.banks_client.get_latest_blockhash().await?;

        self.process_transaction(&Transaction::new_signed_with_payer(
            &[mint_to(
                &jito_lrt_program::id(),
                lrt,
                lrt_mint,
                source_owner,
                source_token_account,
                dest_token_account,
                lrt_receiver,
                amount,
            )],
            Some(&signer.pubkey()),
            &[&signer],
            blockhash,
        ))
        .await
    }

    pub async fn add_delegation(
        &mut self,
        lrt_test_config: &LrtTestConfig,
        restaking_test_config: &RestakingTestConfig,
        amount: u64,
    ) -> Result<(), BanksClientError> {
        let blockhash = self.banks_client.get_latest_blockhash().await?;

        self.process_transaction(&Transaction::new_signed_with_payer(
            &[add_delegation(
                &jito_lrt_program::id(),
                &lrt_test_config.config,
                &lrt_test_config.vault,
                &lrt_test_config.vault_operator_list,
                &restaking_test_config.operator,
                &lrt_test_config.vault_admin.pubkey(),
                &lrt_test_config.vault_admin.pubkey(),
                amount,
            )],
            Some(&lrt_test_config.vault_admin.pubkey()),
            &[&lrt_test_config.vault_admin],
            blockhash,
        ))
        .await
    }

    pub async fn remove_delegation(
        &mut self,
        lrt_test_config: &LrtTestConfig,
        restaking_test_config: &RestakingTestConfig,
        amount: u64,
    ) -> Result<(), BanksClientError> {
        let blockhash = self.banks_client.get_latest_blockhash().await?;

        self.process_transaction(&Transaction::new_signed_with_payer(
            &[remove_delegation(
                &jito_lrt_program::id(),
                &lrt_test_config.config,
                &lrt_test_config.vault,
                &lrt_test_config.vault_operator_list,
                &restaking_test_config.operator,
                &lrt_test_config.vault_admin.pubkey(),
                &lrt_test_config.vault_admin.pubkey(),
                amount,
            )],
            Some(&lrt_test_config.vault_admin.pubkey()),
            &[&lrt_test_config.vault_admin],
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
