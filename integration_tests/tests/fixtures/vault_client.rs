use borsh::BorshDeserialize;
use jito_vault_core::{
    config::Config, vault::Vault, vault_avs_slasher_operator_ticket::VaultAvsSlasherOperatorTicket,
    vault_avs_slasher_ticket::VaultAvsSlasherTicket, vault_avs_ticket::VaultAvsTicket,
    vault_delegation_list::VaultDelegationList, vault_operator_ticket::VaultOperatorTicket,
};
use jito_vault_sdk::{add_delegation, initialize_config, initialize_vault};
use mpl_token_metadata::accounts::Metadata;
use solana_program::pubkey::Pubkey;
use solana_program_test::{BanksClient, BanksClientError};
use solana_sdk::{
    commitment_config::CommitmentLevel,
    signature::{Keypair, Signer},
    transaction::Transaction,
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

    pub async fn get_vault_avs_ticket(
        &mut self,
        vault: &Pubkey,
        avs: &Pubkey,
    ) -> Result<VaultAvsTicket, BanksClientError> {
        let account = VaultAvsTicket::find_program_address(&jito_vault_program::id(), vault, avs).0;
        let account = self.banks_client.get_account(account).await?.unwrap();
        Ok(VaultAvsTicket::deserialize(&mut account.data.as_slice())?)
    }

    pub async fn get_vault_operator_ticket(
        &mut self,
        vault: &Pubkey,
        operator: &Pubkey,
    ) -> Result<VaultOperatorTicket, BanksClientError> {
        let account =
            VaultOperatorTicket::find_program_address(&jito_vault_program::id(), vault, operator).0;
        let account = self.banks_client.get_account(account).await?.unwrap();
        Ok(VaultOperatorTicket::deserialize(
            &mut account.data.as_slice(),
        )?)
    }

    pub async fn get_vault_delegation_list(
        &mut self,
        account: &Pubkey,
    ) -> Result<VaultDelegationList, BanksClientError> {
        let account = self.banks_client.get_account(*account).await?.unwrap();
        Ok(VaultDelegationList::deserialize(
            &mut account.data.as_slice(),
        )?)
    }

    pub async fn get_vault_avs_slasher_ticket(
        &mut self,
        vault: &Pubkey,
        avs: &Pubkey,
        slasher: &Pubkey,
    ) -> Result<VaultAvsSlasherTicket, BanksClientError> {
        let account = VaultAvsSlasherTicket::find_program_address(
            &jito_vault_program::id(),
            vault,
            avs,
            slasher,
        )
        .0;
        let account = self.banks_client.get_account(account).await?.unwrap();
        Ok(VaultAvsSlasherTicket::deserialize(
            &mut account.data.as_slice(),
        )?)
    }

    pub async fn get_vault_avs_slasher_operator_ticket(
        &mut self,
        vault: &Pubkey,
        avs: &Pubkey,
        slasher: &Pubkey,
        operator: &Pubkey,
        epoch: u64,
    ) -> Result<VaultAvsSlasherOperatorTicket, BanksClientError> {
        let account = VaultAvsSlasherOperatorTicket::find_program_address(
            &jito_vault_program::id(),
            vault,
            avs,
            slasher,
            operator,
            epoch,
        )
        .0;
        let account = self.banks_client.get_account(account).await?.unwrap();
        Ok(VaultAvsSlasherOperatorTicket::deserialize(
            &mut account.data.as_slice(),
        )?)
    }

    pub async fn get_token_metadata(
        &mut self,
        account: &Pubkey,
    ) -> Result<Metadata, BanksClientError> {
        let account = self.banks_client.get_account(*account).await?.unwrap();
        Ok(Metadata::deserialize(&mut account.data.as_slice())?)
    }

    pub async fn initialize_config(
        &mut self,
        config: &Pubkey,
        config_admin: &Keypair,
    ) -> Result<(), BanksClientError> {
        let blockhash = self.banks_client.get_latest_blockhash().await?;
        self.process_transaction(&Transaction::new_signed_with_payer(
            &[initialize_config(
                &jito_vault_program::id(),
                &config,
                &config_admin.pubkey(),
                &jito_restaking_program::id(),
            )],
            Some(&config_admin.pubkey()),
            &[config_admin],
            blockhash,
        ))
        .await
    }

    pub async fn initialize_vault(
        &mut self,
        config: &Pubkey,
        vault: &Pubkey,
        vault_delegation_list: &Pubkey,
        lrt_mint: &Keypair,
        token_mint: &Keypair,
        vault_admin: &Keypair,
        vault_base: &Keypair,
        deposit_fee_bps: u16,
        withdrawal_fee_bps: u16,
    ) -> Result<(), BanksClientError> {
        let blockhash = self.banks_client.get_latest_blockhash().await?;

        self.process_transaction(&Transaction::new_signed_with_payer(
            &[initialize_vault(
                &jito_vault_program::id(),
                &config,
                &vault,
                &vault_delegation_list,
                &lrt_mint.pubkey(),
                &token_mint.pubkey(),
                &vault_admin.pubkey(),
                &vault_base.pubkey(),
                deposit_fee_bps,
                withdrawal_fee_bps,
            )],
            Some(&vault_admin.pubkey()),
            &[&vault_admin, &lrt_mint, &vault_base],
            blockhash,
        ))
        .await
    }

    pub async fn add_avs(
        &mut self,
        config: &Pubkey,
        vault: &Pubkey,
        avs: &Pubkey,
        avs_vault_ticket: &Pubkey,
        vault_avs_ticket: &Pubkey,
        admin: &Keypair,
        payer: &Keypair,
    ) -> Result<(), BanksClientError> {
        let blockhash = self.banks_client.get_latest_blockhash().await?;
        self.process_transaction(&Transaction::new_signed_with_payer(
            &[jito_vault_sdk::add_avs(
                &jito_vault_program::id(),
                config,
                vault,
                avs,
                avs_vault_ticket,
                vault_avs_ticket,
                &admin.pubkey(),
                &payer.pubkey(),
            )],
            Some(&payer.pubkey()),
            &[admin, payer],
            blockhash,
        ))
        .await
    }

    // pub async fn remove_avs(
    //     &mut self,
    //     config: &Pubkey,
    //     vault: &Pubkey,
    //     avs: &Pubkey,
    //     vault_avs_ticket: &Pubkey,
    //     admin: &Keypair,
    // ) -> Result<(), BanksClientError> {
    //     let blockhash = self.banks_client.get_latest_blockhash().await?;
    //     self.process_transaction(&Transaction::new_signed_with_payer(
    //         &[jito_vault_sdk::remove_avs(
    //             &jito_vault_program::id(),
    //             config,
    //             vault,
    //             avs,
    //             vault_avs_ticket,
    //             &admin.pubkey(),
    //         )],
    //         Some(&admin.pubkey()),
    //         &[admin],
    //         blockhash,
    //     ))
    //     .await
    // }

    pub async fn add_operator(
        &mut self,
        config: &Pubkey,
        vault: &Pubkey,
        operator: &Pubkey,
        operator_vault_ticket: &Pubkey,
        vault_operator_ticket: &Pubkey,
        admin: &Keypair,
        payer: &Keypair,
    ) -> Result<(), BanksClientError> {
        let blockhash = self.banks_client.get_latest_blockhash().await?;
        self.process_transaction(&Transaction::new_signed_with_payer(
            &[jito_vault_sdk::add_operator(
                &jito_vault_program::id(),
                config,
                vault,
                operator,
                operator_vault_ticket,
                vault_operator_ticket,
                &admin.pubkey(),
                &payer.pubkey(),
            )],
            Some(&payer.pubkey()),
            &[admin, payer],
            blockhash,
        ))
        .await
    }

    // pub async fn remove_operator(
    //     &mut self,
    //     config: &Pubkey,
    //     vault: &Pubkey,
    //     operator: &Pubkey,
    //     vault_operator_ticket: &Pubkey,
    //     admin: &Keypair,
    // ) -> Result<(), BanksClientError> {
    //     let blockhash = self.banks_client.get_latest_blockhash().await?;
    //     self.process_transaction(&Transaction::new_signed_with_payer(
    //         &[jito_vault_sdk::remove_operator(
    //             &jito_vault_program::id(),
    //             config,
    //             vault,
    //             operator,
    //             vault_operator_ticket,
    //             &admin.pubkey(),
    //         )],
    //         Some(&admin.pubkey()),
    //         &[admin],
    //         blockhash,
    //     ))
    //     .await
    // }

    pub async fn add_delegation(
        &mut self,
        config: &Pubkey,
        vault: &Pubkey,
        operator: &Pubkey,
        vault_operator_ticket: &Pubkey,
        vault_delegation_list: &Pubkey,
        admin: &Keypair,
        payer: &Keypair,
        amount: u64,
    ) -> Result<(), BanksClientError> {
        let blockhash = self.banks_client.get_latest_blockhash().await?;
        self.process_transaction(&Transaction::new_signed_with_payer(
            &[add_delegation(
                &jito_vault_program::id(),
                config,
                vault,
                operator,
                vault_operator_ticket,
                vault_delegation_list,
                &admin.pubkey(),
                &payer.pubkey(),
                amount,
            )],
            Some(&payer.pubkey()),
            &[admin, payer],
            blockhash,
        ))
        .await
    }

    // pub async fn remove_delegation(
    //     &mut self,
    //     config: &Pubkey,
    //     vault: &Pubkey,
    //     operator: &Pubkey,
    //     vault_delegation_list: &Pubkey,
    //     admin: &Keypair,
    //     amount: u64,
    // ) -> Result<(), BanksClientError> {
    //     let blockhash = self.banks_client.get_latest_blockhash().await?;
    //     self.process_transaction(&Transaction::new_signed_with_payer(
    //         &[remove_delegation(
    //             &jito_vault_program::id(),
    //             config,
    //             vault,
    //             operator,
    //             vault_delegation_list,
    //             &admin.pubkey(),
    //             amount,
    //         )],
    //         Some(&admin.pubkey()),
    //         &[admin],
    //         blockhash,
    //     ))
    //     .await
    // }

    pub async fn mint_to(
        &mut self,
        vault: &Pubkey,
        lrt_mint: &Pubkey,
        depositor: &Keypair,
        depositor_token_account: &Pubkey,
        vault_token_account: &Pubkey,
        depositor_lrt_token_account: &Pubkey,
        vault_fee_token_account: &Pubkey,
        mint_signer: Option<&Keypair>,
        amount: u64,
    ) -> Result<(), BanksClientError> {
        let blockhash = self.banks_client.get_latest_blockhash().await?;
        let mut signers = vec![depositor];
        if let Some(signer) = mint_signer {
            signers.push(signer);
        }
        self.process_transaction(&Transaction::new_signed_with_payer(
            &[jito_vault_sdk::mint_to(
                &jito_vault_program::id(),
                vault,
                lrt_mint,
                &depositor.pubkey(),
                depositor_token_account,
                vault_token_account,
                depositor_lrt_token_account,
                vault_fee_token_account,
                mint_signer.map(|s| s.pubkey()).as_ref(),
                amount,
            )],
            Some(&depositor.pubkey()),
            &signers,
            blockhash,
        ))
        .await
    }

    // pub async fn set_deposit_capacity(
    //     &mut self,
    //     vault: &Pubkey,
    //     admin: &Keypair,
    //     amount: u64,
    // ) -> Result<(), BanksClientError> {
    //     let blockhash = self.banks_client.get_latest_blockhash().await?;
    //     self.process_transaction(&Transaction::new_signed_with_payer(
    //         &[jito_vault_sdk::set_deposit_capacity(
    //             &jito_vault_program::id(),
    //             vault,
    //             &admin.pubkey(),
    //             amount,
    //         )],
    //         Some(&admin.pubkey()),
    //         &[admin],
    //         blockhash,
    //     ))
    //     .await
    // }

    // pub async fn set_admin(
    //     &mut self,
    //     vault: &Pubkey,
    //     old_admin: &Keypair,
    //     new_admin: &Pubkey,
    // ) -> Result<(), BanksClientError> {
    //     let blockhash = self.banks_client.get_latest_blockhash().await?;
    //     self.process_transaction(&Transaction::new_signed_with_payer(
    //         &[jito_vault_sdk::set_admin(
    //             &jito_vault_program::id(),
    //             vault,
    //             &old_admin.pubkey(),
    //             new_admin,
    //         )],
    //         Some(&old_admin.pubkey()),
    //         &[old_admin],
    //         blockhash,
    //     ))
    //     .await
    // }

    // pub async fn set_secondary_admin(
    //     &mut self,
    //     vault: &Pubkey,
    //     admin: &Keypair,
    //     new_admin: &Pubkey,
    //     role: VaultAdminRole,
    // ) -> Result<(), BanksClientError> {
    //     let blockhash = self.banks_client.get_latest_blockhash().await?;
    //     self.process_transaction(&Transaction::new_signed_with_payer(
    //         &[jito_vault_sdk::set_secondary_admin(
    //             &jito_vault_program::id(),
    //             vault,
    //             &admin.pubkey(),
    //             new_admin,
    //             role,
    //         )],
    //         Some(&admin.pubkey()),
    //         &[admin],
    //         blockhash,
    //     ))
    //     .await
    // }

    // pub async fn update_delegations(
    //     &mut self,
    //     config: &Pubkey,
    //     vault: &Pubkey,
    //     vault_delegation_list: &Pubkey,
    //     payer: &Keypair,
    // ) -> Result<(), BanksClientError> {
    //     let blockhash = self.banks_client.get_latest_blockhash().await?;
    //     self.process_transaction(&Transaction::new_signed_with_payer(
    //         &[jito_vault_sdk::update_delegations(
    //             &jito_vault_program::id(),
    //             config,
    //             vault,
    //             vault_delegation_list,
    //             &payer.pubkey(),
    //         )],
    //         Some(&payer.pubkey()),
    //         &[payer],
    //         blockhash,
    //     ))
    //     .await
    // }

    pub async fn add_slasher(
        &mut self,
        config: &Pubkey,
        vault: &Pubkey,
        avs: &Pubkey,
        slasher: &Pubkey,
        avs_slasher_ticket: &Pubkey,
        vault_slasher_ticket: &Pubkey,
        admin: &Keypair,
        payer: &Keypair,
    ) -> Result<(), BanksClientError> {
        let blockhash = self.banks_client.get_latest_blockhash().await?;
        self.process_transaction(&Transaction::new_signed_with_payer(
            &[jito_vault_sdk::add_slasher(
                &jito_vault_program::id(),
                config,
                vault,
                avs,
                slasher,
                avs_slasher_ticket,
                vault_slasher_ticket,
                &admin.pubkey(),
                &payer.pubkey(),
            )],
            Some(&payer.pubkey()),
            &[admin, payer],
            blockhash,
        ))
        .await
    }

    pub async fn initialize_vault_avs_slasher_operator_ticket(
        &mut self,
        config: &Pubkey,
        vault: &Pubkey,
        avs: &Pubkey,
        slasher: &Pubkey,
        operator: &Pubkey,
        vault_avs_slasher_ticket: &Pubkey,
        vault_avs_slasher_operator_ticket: &Pubkey,
        payer: &Keypair,
    ) -> Result<(), BanksClientError> {
        let blockhash = self.banks_client.get_latest_blockhash().await?;
        self.process_transaction(&Transaction::new_signed_with_payer(
            &[
                jito_vault_sdk::initialize_vault_avs_slasher_operator_ticket(
                    &jito_vault_program::id(),
                    config,
                    vault,
                    avs,
                    slasher,
                    operator,
                    vault_avs_slasher_ticket,
                    vault_avs_slasher_operator_ticket,
                    &payer.pubkey(),
                ),
            ],
            Some(&payer.pubkey()),
            &[payer],
            blockhash,
        ))
        .await
    }

    pub async fn slash(
        &mut self,
        config: &Pubkey,
        vault: &Pubkey,
        avs: &Pubkey,
        operator: &Pubkey,
        slasher: &Keypair,
        avs_operator_ticket: &Pubkey,
        operator_avs_ticket: &Pubkey,
        avs_vault_ticket: &Pubkey,
        operator_vault_ticket: &Pubkey,
        vault_avs_ticket: &Pubkey,
        vault_operator_ticket: &Pubkey,
        avs_vault_slasher_ticket: &Pubkey,
        vault_avs_slasher_ticket: &Pubkey,
        vault_delegation_list: &Pubkey,
        vault_avs_slasher_operator_ticket: &Pubkey,
        vault_token_account: &Pubkey,
        slasher_token_account: &Pubkey,
        amount: u64,
    ) -> Result<(), BanksClientError> {
        let blockhash = self.banks_client.get_latest_blockhash().await?;
        self.process_transaction(&Transaction::new_signed_with_payer(
            &[jito_vault_sdk::slash(
                &jito_vault_program::id(),
                config,
                vault,
                avs,
                operator,
                &slasher.pubkey(),
                avs_operator_ticket,
                operator_avs_ticket,
                avs_vault_ticket,
                operator_vault_ticket,
                vault_avs_ticket,
                vault_operator_ticket,
                avs_vault_slasher_ticket,
                vault_avs_slasher_ticket,
                vault_delegation_list,
                vault_avs_slasher_operator_ticket,
                vault_token_account,
                slasher_token_account,
                amount,
            )],
            Some(&slasher.pubkey()),
            &[slasher],
            blockhash,
        ))
        .await
    }

    pub async fn create_token_metadata(
        &mut self,
        vault: &Pubkey,
        lrt_mint: &Pubkey,
        metadata: &Pubkey,
        admin: &Keypair,
        name: String,
        symbol: String,
        uri: String,
    ) -> Result<(), BanksClientError> {
        let blockhash = self.banks_client.get_latest_blockhash().await?;
        self.process_transaction(&Transaction::new_signed_with_payer(
            &[jito_vault_sdk::create_token_metadata(
                &jito_vault_program::id(),
                vault,
                lrt_mint,
                metadata,
                &admin.pubkey(),
                name,
                symbol,
                uri,
            )],
            Some(&admin.pubkey()),
            &[admin],
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
