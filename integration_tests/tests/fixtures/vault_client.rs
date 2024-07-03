use borsh::BorshDeserialize;
use jito_vault_core::{
    config::Config, vault::Vault, vault_avs_list::VaultAvsList,
    vault_operator_list::VaultOperatorList,
};
use jito_vault_sdk::{add_delegation, initialize_config, initialize_vault, remove_delegation};
use solana_program::pubkey::Pubkey;
use solana_program_test::{BanksClient, BanksClientError};
use solana_sdk::{commitment_config::CommitmentLevel, signature::Signer, transaction::Transaction};
use spl_associated_token_account::instruction::create_associated_token_account_idempotent;

use crate::fixtures::{
    restaking_test_config::RestakingTestConfig, vault_test_config::VaultTestConfig,
};

pub struct VaultProgramClient {
    banks_client: BanksClient,
}

impl VaultProgramClient {
    pub const fn new(banks_client: BanksClient) -> Self {
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
        vault_config: &VaultTestConfig,
    ) -> Result<(), BanksClientError> {
        let blockhash = self.banks_client.get_latest_blockhash().await?;
        self.process_transaction(&Transaction::new_signed_with_payer(
            &[initialize_config(
                &jito_vault_program::id(),
                &vault_config.config,
                &vault_config.config_admin.pubkey(),
                &jito_restaking_program::id(),
            )],
            Some(&vault_config.config_admin.pubkey()),
            &[&vault_config.config_admin],
            blockhash,
        ))
        .await
    }

    pub async fn initialize_vault(
        &mut self,
        config: &VaultTestConfig,
    ) -> Result<(), BanksClientError> {
        let blockhash = self.banks_client.get_latest_blockhash().await?;

        self.process_transaction(&Transaction::new_signed_with_payer(
            &[
                initialize_vault(
                    &jito_vault_program::id(),
                    &config.config,
                    &config.vault,
                    &config.vault_avs_list,
                    &config.vault_operator_list,
                    &config.vault_slasher_list,
                    &config.lrt_mint.pubkey(),
                    &config.token_mint.pubkey(),
                    &config.vault_admin.pubkey(),
                    &config.vault_base.pubkey(),
                    config.deposit_fee_bps,
                    config.withdrawal_fee_bps,
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

    // pub async fn mint(
    //     &mut self,
    //     vault_config: VaultTestConfig
    // ) -> Result<(), BanksClientError> {
    //     let blockhash = self.banks_client.get_latest_blockhash().await?;
    //
    //     self.process_transaction(&Transaction::new_signed_with_payer(
    //         &[mint_to(
    //             program_id: &Pubkey,
    //             vault: &Pubkey,
    //             lrt_mint: &Pubkey,
    //             depositor: &Pubkey,
    //             depositor_token_account: &Pubkey,
    //             vault_token_account: &Pubkey,
    //             depositor_lrt_token_account: &Pubkey,
    //             vault_fee_token_account: &Pubkey,
    //             amount: u64,
    //             &jito_vault_program::id(),
    //             lrt,
    //             lrt_mint,
    //             source_owner,
    //             source_token_account,
    //             dest_token_account,
    //             lrt_receiver,
    //             amount,
    //         )],
    //         Some(&signer.pubkey()),
    //         &[&signer],
    //         blockhash,
    //     ))
    //     .await
    // }

    pub async fn add_delegation(
        &mut self,
        vault_test_config: &VaultTestConfig,
        restaking_test_config: &RestakingTestConfig,
        amount: u64,
    ) -> Result<(), BanksClientError> {
        let blockhash = self.banks_client.get_latest_blockhash().await?;

        self.process_transaction(&Transaction::new_signed_with_payer(
            &[add_delegation(
                &jito_vault_program::id(),
                &vault_test_config.config,
                &vault_test_config.vault,
                &vault_test_config.vault_operator_list,
                &restaking_test_config.operator,
                &vault_test_config.vault_admin.pubkey(),
                &vault_test_config.vault_admin.pubkey(),
                amount,
            )],
            Some(&vault_test_config.vault_admin.pubkey()),
            &[&vault_test_config.vault_admin],
            blockhash,
        ))
        .await
    }

    pub async fn remove_delegation(
        &mut self,
        vault_test_config: &VaultTestConfig,
        restaking_test_config: &RestakingTestConfig,
        amount: u64,
    ) -> Result<(), BanksClientError> {
        let blockhash = self.banks_client.get_latest_blockhash().await?;

        self.process_transaction(&Transaction::new_signed_with_payer(
            &[remove_delegation(
                &jito_vault_program::id(),
                &vault_test_config.config,
                &vault_test_config.vault,
                &vault_test_config.vault_operator_list,
                &restaking_test_config.operator,
                &vault_test_config.vault_admin.pubkey(),
                &vault_test_config.vault_admin.pubkey(),
                amount,
            )],
            Some(&vault_test_config.vault_admin.pubkey()),
            &[&vault_test_config.vault_admin],
            blockhash,
        ))
        .await
    }

    pub async fn process_transaction(&mut self, tx: &Transaction) -> Result<(), BanksClientError> {
        self.banks_client
            .process_transaction_with_preflight_and_commitment(
                tx.clone(),
                CommitmentLevel::Processed,
            )
            .await
    }
}
