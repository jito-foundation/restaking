use borsh::BorshDeserialize;
use jito_restaking_core::{
    avs::Avs, avs_operator_ticket::AvsOperatorTicket,
    avs_vault_slasher_ticket::AvsVaultSlasherTicket, avs_vault_ticket::AvsVaultTicket,
    config::Config, operator::Operator, operator_avs_ticket::OperatorAvsTicket,
    operator_vault_ticket::OperatorVaultTicket,
};
use jito_restaking_sdk::{
    avs_add_operator, avs_add_vault, avs_add_vault_slasher, initialize_avs, initialize_config,
    initialize_operator, operator_add_avs, operator_add_vault,
};
use log::info;
use solana_program::{native_token::sol_to_lamports, pubkey::Pubkey, system_instruction::transfer};
use solana_program_test::{BanksClient, BanksClientError};
use solana_sdk::{
    commitment_config::CommitmentLevel,
    signature::{Keypair, Signer},
    transaction::Transaction,
};

pub struct AvsRoot {
    pub avs_pubkey: Pubkey,
    pub avs_admin: Keypair,
}

pub struct OperatorRoot {
    pub operator_pubkey: Pubkey,
    pub operator_admin: Keypair,
}

pub struct RestakingProgramClient {
    banks_client: BanksClient,
    payer: Keypair,
}

impl RestakingProgramClient {
    pub const fn new(banks_client: BanksClient, payer: Keypair) -> Self {
        Self {
            banks_client,
            payer,
        }
    }

    pub async fn get_avs(&mut self, avs: &Pubkey) -> Result<Avs, BanksClientError> {
        let account = self
            .banks_client
            .get_account_with_commitment(*avs, CommitmentLevel::Processed)
            .await?
            .unwrap();

        Ok(Avs::deserialize(&mut account.data.as_slice())?)
    }

    pub async fn get_config(&mut self, account: &Pubkey) -> Result<Config, BanksClientError> {
        let account = self.banks_client.get_account(*account).await?.unwrap();
        Ok(Config::deserialize(&mut account.data.as_slice())?)
    }

    pub async fn get_avs_vault_ticket(
        &mut self,
        avs: &Pubkey,
        vault: &Pubkey,
    ) -> Result<AvsVaultTicket, BanksClientError> {
        let account =
            AvsVaultTicket::find_program_address(&jito_restaking_program::id(), &avs, &vault).0;
        let account = self.banks_client.get_account(account).await?.unwrap();
        Ok(AvsVaultTicket::deserialize(&mut account.data.as_slice())?)
    }

    pub async fn get_avs_operator_ticket(
        &mut self,
        avs: &Pubkey,
        operator: &Pubkey,
    ) -> Result<AvsOperatorTicket, BanksClientError> {
        let account =
            AvsOperatorTicket::find_program_address(&jito_restaking_program::id(), &avs, &operator)
                .0;
        let account = self.banks_client.get_account(account).await?.unwrap();
        Ok(AvsOperatorTicket::deserialize(
            &mut account.data.as_slice(),
        )?)
    }

    pub async fn get_avs_vault_slasher_ticket(
        &mut self,
        avs: &Pubkey,
        vault: &Pubkey,
        slasher: &Pubkey,
    ) -> Result<AvsVaultSlasherTicket, BanksClientError> {
        let account = AvsVaultSlasherTicket::find_program_address(
            &jito_restaking_program::id(),
            &avs,
            &vault,
            &slasher,
        )
        .0;
        let account = self.banks_client.get_account(account).await?.unwrap();
        Ok(AvsVaultSlasherTicket::deserialize(
            &mut account.data.as_slice(),
        )?)
    }

    pub async fn get_operator(&mut self, account: &Pubkey) -> Result<Operator, BanksClientError> {
        let account = self.banks_client.get_account(*account).await?.unwrap();
        Ok(Operator::deserialize(&mut account.data.as_slice())?)
    }

    pub async fn get_operator_vault_ticket(
        &mut self,
        operator: &Pubkey,
        vault: &Pubkey,
    ) -> Result<OperatorVaultTicket, BanksClientError> {
        let account = OperatorVaultTicket::find_program_address(
            &jito_restaking_program::id(),
            &operator,
            &vault,
        )
        .0;
        let account = self.banks_client.get_account(account).await?.unwrap();
        Ok(OperatorVaultTicket::deserialize(
            &mut account.data.as_slice(),
        )?)
    }

    pub async fn get_operator_avs_ticket(
        &mut self,
        operator: &Pubkey,
        avs: &Pubkey,
    ) -> Result<OperatorAvsTicket, BanksClientError> {
        let account =
            OperatorAvsTicket::find_program_address(&jito_restaking_program::id(), &operator, &avs)
                .0;
        let account = self.banks_client.get_account(account).await?.unwrap();
        Ok(OperatorAvsTicket::deserialize(
            &mut account.data.as_slice(),
        )?)
    }

    pub async fn setup_config(&mut self) -> Result<Keypair, BanksClientError> {
        let restaking_config_pubkey = Config::find_program_address(&jito_restaking_program::id()).0;
        let restaking_config_admin = Keypair::new();

        self._airdrop(&restaking_config_admin.pubkey(), 1.0).await?;
        self.initialize_config(&restaking_config_pubkey, &restaking_config_admin)
            .await?;

        Ok(restaking_config_admin)
    }

    pub async fn setup_operator(&mut self) -> Result<OperatorRoot, BanksClientError> {
        // create operator + add operator vault
        let operator_base = Keypair::new();
        let operator_pubkey =
            Operator::find_program_address(&jito_restaking_program::id(), &operator_base.pubkey())
                .0;
        let operator_admin = Keypair::new();
        self._airdrop(&operator_admin.pubkey(), 1.0).await?;
        self.initialize_operator(
            &Config::find_program_address(&jito_restaking_program::id()).0,
            &operator_pubkey,
            &operator_admin,
            &operator_base,
        )
        .await
        .unwrap();
        Ok(OperatorRoot {
            operator_pubkey,
            operator_admin,
        })
    }

    pub async fn operator_vault_opt_in(
        &mut self,
        operator_root: &OperatorRoot,
        vault_pubkey: &Pubkey,
    ) -> Result<(), BanksClientError> {
        let operator_vault_ticket = OperatorVaultTicket::find_program_address(
            &jito_restaking_program::id(),
            &operator_root.operator_pubkey,
            &vault_pubkey,
        )
        .0;
        self.operator_add_vault(
            &Config::find_program_address(&jito_restaking_program::id()).0,
            &operator_root.operator_pubkey,
            &vault_pubkey,
            &operator_vault_ticket,
            &operator_root.operator_admin,
            &operator_root.operator_admin,
        )
        .await?;

        Ok(())
    }

    pub async fn initialize_config(
        &mut self,
        config: &Pubkey,
        config_admin: &Keypair,
    ) -> Result<(), BanksClientError> {
        let blockhash = self.banks_client.get_latest_blockhash().await?;
        self.process_transaction(&Transaction::new_signed_with_payer(
            &[initialize_config(
                &jito_restaking_program::id(),
                config,
                &config_admin.pubkey(),
                &jito_vault_program::id(),
            )],
            Some(&config_admin.pubkey()),
            &[config_admin],
            blockhash,
        ))
        .await
    }

    pub async fn setup_avs(&mut self) -> Result<AvsRoot, BanksClientError> {
        let avs_admin = Keypair::new();
        let avs_base = Keypair::new();

        self._airdrop(&avs_admin.pubkey(), 1.0).await?;

        let avs_pubkey =
            Avs::find_program_address(&jito_restaking_program::id(), &avs_base.pubkey()).0;
        self.initialize_avs(
            &Config::find_program_address(&jito_restaking_program::id()).0,
            &avs_pubkey,
            &avs_admin,
            &avs_base,
        )
        .await
        .unwrap();

        Ok(AvsRoot {
            avs_pubkey,
            avs_admin,
        })
    }

    pub async fn avs_vault_opt_in(
        &mut self,
        avs_root: &AvsRoot,
        vault: &Pubkey,
    ) -> Result<(), BanksClientError> {
        let avs_vault_ticket = AvsVaultTicket::find_program_address(
            &jito_restaking_program::id(),
            &avs_root.avs_pubkey,
            vault,
        )
        .0;

        self.avs_add_vault(
            &Config::find_program_address(&jito_restaking_program::id()).0,
            &avs_root.avs_pubkey,
            vault,
            &avs_vault_ticket,
            &avs_root.avs_admin,
            &self.payer.insecure_clone(),
        )
        .await
    }

    pub async fn avs_operator_opt_in(
        &mut self,
        avs_root: &AvsRoot,
        operator: &Pubkey,
    ) -> Result<(), BanksClientError> {
        let avs_operator_ticket = AvsOperatorTicket::find_program_address(
            &jito_restaking_program::id(),
            &avs_root.avs_pubkey,
            operator,
        )
        .0;
        let operator_avs_ticket = OperatorAvsTicket::find_program_address(
            &jito_restaking_program::id(),
            operator,
            &avs_root.avs_pubkey,
        )
        .0;

        self.avs_add_operator(
            &Config::find_program_address(&jito_restaking_program::id()).0,
            &avs_root.avs_pubkey,
            operator,
            &avs_operator_ticket,
            &operator_avs_ticket,
            &avs_root.avs_admin,
            &self.payer.insecure_clone(),
        )
        .await
    }

    pub async fn operator_avs_opt_in(
        &mut self,
        operator_root: &OperatorRoot,
        avs: &Pubkey,
    ) -> Result<(), BanksClientError> {
        let operator_avs_ticket = OperatorAvsTicket::find_program_address(
            &jito_restaking_program::id(),
            &operator_root.operator_pubkey,
            avs,
        )
        .0;

        self.operator_add_avs(
            &Config::find_program_address(&jito_restaking_program::id()).0,
            &operator_root.operator_pubkey,
            avs,
            &operator_avs_ticket,
            &operator_root.operator_admin,
            &operator_root.operator_admin,
        )
        .await
    }

    pub async fn avs_vault_slasher_opt_in(
        &mut self,
        avs_root: &AvsRoot,
        vault: &Pubkey,
        slasher: &Pubkey,
        max_slash_amount: u64,
    ) -> Result<(), BanksClientError> {
        let avs_vault_ticket = AvsVaultTicket::find_program_address(
            &jito_restaking_program::id(),
            &avs_root.avs_pubkey,
            vault,
        )
        .0;
        let avs_slasher_ticket = AvsVaultSlasherTicket::find_program_address(
            &jito_restaking_program::id(),
            &avs_root.avs_pubkey,
            vault,
            slasher,
        )
        .0;

        self.avs_add_vault_slasher(
            &Config::find_program_address(&jito_restaking_program::id()).0,
            &avs_root.avs_pubkey,
            vault,
            slasher,
            &avs_vault_ticket,
            &avs_slasher_ticket,
            &avs_root.avs_admin,
            &self.payer.insecure_clone(),
            max_slash_amount,
        )
        .await
    }

    pub async fn initialize_avs(
        &mut self,
        config: &Pubkey,
        avs: &Pubkey,
        avs_admin: &Keypair,
        avs_base: &Keypair,
    ) -> Result<(), BanksClientError> {
        let blockhash = self.banks_client.get_latest_blockhash().await?;

        let admin_account = self
            .banks_client
            .get_account_with_commitment(avs_admin.pubkey(), CommitmentLevel::Processed)
            .await
            .unwrap();

        self.process_transaction(&Transaction::new_signed_with_payer(
            &[initialize_avs(
                &jito_restaking_program::id(),
                &config,
                &avs,
                &avs_admin.pubkey(),
                &avs_base.pubkey(),
            )],
            Some(&avs_admin.pubkey()),
            &[&avs_admin, &avs_base],
            blockhash,
        ))
        .await
    }

    pub async fn avs_add_vault(
        &mut self,
        config: &Pubkey,
        avs: &Pubkey,
        vault: &Pubkey,
        avs_vault_ticket: &Pubkey,
        avs_admin: &Keypair,
        payer: &Keypair,
    ) -> Result<(), BanksClientError> {
        let blockhash = self.banks_client.get_latest_blockhash().await?;

        self.process_transaction(&Transaction::new_signed_with_payer(
            &[avs_add_vault(
                &jito_restaking_program::id(),
                config,
                avs,
                vault,
                avs_vault_ticket,
                &avs_admin.pubkey(),
                &payer.pubkey(),
            )],
            Some(&payer.pubkey()),
            &[avs_admin, payer],
            blockhash,
        ))
        .await
    }

    // pub async fn avs_remove_vault(
    //     &mut self,
    //     config: &Pubkey,
    //     avs: &Pubkey,
    //     vault: &Pubkey,
    //     avs_vault_ticket: &Pubkey,
    //     avs_admin: &Keypair,
    // ) -> Result<(), BanksClientError> {
    //     let blockhash = self.banks_client.get_latest_blockhash().await?;
    //
    //     self.process_transaction(&Transaction::new_signed_with_payer(
    //         &[avs_remove_vault(
    //             &jito_restaking_program::id(),
    //             config,
    //             avs,
    //             vault,
    //             avs_vault_ticket,
    //             &avs_admin.pubkey(),
    //         )],
    //         Some(&avs_admin.pubkey()),
    //         &[avs_admin],
    //         blockhash,
    //     ))
    //     .await
    // }

    pub async fn avs_add_operator(
        &mut self,
        config: &Pubkey,
        avs: &Pubkey,
        operator: &Pubkey,
        avs_operator_ticket: &Pubkey,
        operator_avs_ticket: &Pubkey,
        avs_admin: &Keypair,
        payer: &Keypair,
    ) -> Result<(), BanksClientError> {
        let blockhash = self.banks_client.get_latest_blockhash().await?;

        self.process_transaction(&Transaction::new_signed_with_payer(
            &[avs_add_operator(
                &jito_restaking_program::id(),
                config,
                avs,
                operator,
                avs_operator_ticket,
                operator_avs_ticket,
                &avs_admin.pubkey(),
                &payer.pubkey(),
            )],
            Some(&payer.pubkey()),
            &[avs_admin, payer],
            blockhash,
        ))
        .await
    }

    // pub async fn avs_remove_operator(
    //     &mut self,
    //     config: &Pubkey,
    //     avs: &Pubkey,
    //     operator: &Pubkey,
    //     avs_operator_ticket: &Pubkey,
    //     avs_admin: &Keypair,
    // ) -> Result<(), BanksClientError> {
    //     let blockhash = self.banks_client.get_latest_blockhash().await?;
    //
    //     self.process_transaction(&Transaction::new_signed_with_payer(
    //         &[avs_remove_operator(
    //             &jito_restaking_program::id(),
    //             config,
    //             avs,
    //             operator,
    //             avs_operator_ticket,
    //             &avs_admin.pubkey(),
    //         )],
    //         Some(&avs_admin.pubkey()),
    //         &[avs_admin],
    //         blockhash,
    //     ))
    //     .await
    // }

    pub async fn avs_add_vault_slasher(
        &mut self,
        config: &Pubkey,
        avs: &Pubkey,
        vault: &Pubkey,
        slasher: &Pubkey,
        avs_vault_ticket: &Pubkey,
        avs_slasher_ticket: &Pubkey,
        avs_admin: &Keypair,
        payer: &Keypair,
        max_slash_amount: u64,
    ) -> Result<(), BanksClientError> {
        let blockhash = self.banks_client.get_latest_blockhash().await?;

        self.process_transaction(&Transaction::new_signed_with_payer(
            &[avs_add_vault_slasher(
                &jito_restaking_program::id(),
                config,
                avs,
                vault,
                slasher,
                avs_vault_ticket,
                avs_slasher_ticket,
                &avs_admin.pubkey(),
                &payer.pubkey(),
                max_slash_amount,
            )],
            Some(&payer.pubkey()),
            &[avs_admin, payer],
            blockhash,
        ))
        .await
    }

    // pub async fn avs_remove_vault_slasher(
    //     &mut self,
    //     config: &Pubkey,
    //     avs: &Pubkey,
    //     vault: &Pubkey,
    //     slasher: &Pubkey,
    //     avs_slasher_ticket: &Pubkey,
    //     avs_admin: &Keypair,
    // ) -> Result<(), BanksClientError> {
    //     let blockhash = self.banks_client.get_latest_blockhash().await?;
    //
    //     self.process_transaction(&Transaction::new_signed_with_payer(
    //         &[avs_remove_vault_slasher(
    //             &jito_restaking_program::id(),
    //             config,
    //             avs,
    //             vault,
    //             slasher,
    //             avs_slasher_ticket,
    //             &avs_admin.pubkey(),
    //         )],
    //         Some(&avs_admin.pubkey()),
    //         &[avs_admin],
    //         blockhash,
    //     ))
    //     .await
    // }
    //
    // pub async fn avs_set_admin(
    //     &mut self,
    //     avs: &Pubkey,
    //     old_admin: &Keypair,
    //     new_admin: &Keypair,
    // ) -> Result<(), BanksClientError> {
    //     let blockhash = self.banks_client.get_latest_blockhash().await?;
    //
    //     self.process_transaction(&Transaction::new_signed_with_payer(
    //         &[avs_set_admin(
    //             &jito_restaking_program::id(),
    //             avs,
    //             &old_admin.pubkey(),
    //             &new_admin.pubkey(),
    //         )],
    //         Some(&old_admin.pubkey()),
    //         &[old_admin, new_admin],
    //         blockhash,
    //     ))
    //     .await
    // }
    //
    // pub async fn avs_set_secondary_admin(
    //     &mut self,
    //     avs: &Pubkey,
    //     admin: &Keypair,
    //     new_admin: &Pubkey,
    //     role: AvsAdminRole,
    // ) -> Result<(), BanksClientError> {
    //     let blockhash = self.banks_client.get_latest_blockhash().await?;
    //
    //     self.process_transaction(&Transaction::new_signed_with_payer(
    //         &[avs_set_secondary_admin(
    //             &jito_restaking_program::id(),
    //             avs,
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

    pub async fn initialize_operator(
        &mut self,
        config: &Pubkey,
        operator: &Pubkey,
        admin: &Keypair,
        base: &Keypair,
    ) -> Result<(), BanksClientError> {
        let blockhash = self.banks_client.get_latest_blockhash().await?;

        self.process_transaction(&Transaction::new_signed_with_payer(
            &[initialize_operator(
                &jito_restaking_program::id(),
                config,
                operator,
                &admin.pubkey(),
                &base.pubkey(),
            )],
            Some(&admin.pubkey()),
            &[admin, base],
            blockhash,
        ))
        .await
    }

    // pub async fn operator_set_admin(
    //     &mut self,
    //     node_operator: &Pubkey,
    //     old_admin: &Keypair,
    //     new_admin: &Keypair,
    // ) -> Result<(), BanksClientError> {
    //     let blockhash = self.banks_client.get_latest_blockhash().await?;
    //
    //     self.process_transaction(&Transaction::new_signed_with_payer(
    //         &[operator_set_admin(
    //             &jito_restaking_program::id(),
    //             node_operator,
    //             &old_admin.pubkey(),
    //             &new_admin.pubkey(),
    //         )],
    //         Some(&old_admin.pubkey()),
    //         &[old_admin, new_admin],
    //         blockhash,
    //     ))
    //     .await
    // }
    //
    // pub async fn operator_set_voter(
    //     &mut self,
    //     node_operator: &Pubkey,
    //     admin: &Keypair,
    //     voter: &Pubkey,
    // ) -> Result<(), BanksClientError> {
    //     let blockhash = self.banks_client.get_latest_blockhash().await?;
    //
    //     self.process_transaction(&Transaction::new_signed_with_payer(
    //         &[operator_set_voter(
    //             &jito_restaking_program::id(),
    //             node_operator,
    //             &admin.pubkey(),
    //             voter,
    //         )],
    //         Some(&admin.pubkey()),
    //         &[admin],
    //         blockhash,
    //     ))
    //     .await
    // }

    pub async fn operator_add_vault(
        &mut self,
        config: &Pubkey,
        operator: &Pubkey,
        vault: &Pubkey,
        operator_vault_ticket: &Pubkey,
        admin: &Keypair,
        payer: &Keypair,
    ) -> Result<(), BanksClientError> {
        let blockhash = self.banks_client.get_latest_blockhash().await?;

        self.process_transaction(&Transaction::new_signed_with_payer(
            &[operator_add_vault(
                &jito_restaking_program::id(),
                config,
                operator,
                vault,
                operator_vault_ticket,
                &admin.pubkey(),
                &payer.pubkey(),
            )],
            Some(&payer.pubkey()),
            &[admin, payer],
            blockhash,
        ))
        .await
    }

    // pub async fn operator_remove_vault(
    //     &mut self,
    //     config: &Pubkey,
    //     operator: &Pubkey,
    //     vault: &Pubkey,
    //     operator_vault_ticket: &Pubkey,
    //     admin: &Keypair,
    // ) -> Result<(), BanksClientError> {
    //     let blockhash = self.banks_client.get_latest_blockhash().await?;
    //
    //     self.process_transaction(&Transaction::new_signed_with_payer(
    //         &[operator_remove_vault(
    //             &jito_restaking_program::id(),
    //             config,
    //             operator,
    //             vault,
    //             operator_vault_ticket,
    //             &admin.pubkey(),
    //         )],
    //         Some(&admin.pubkey()),
    //         &[admin],
    //         blockhash,
    //     ))
    //     .await
    // }

    pub async fn operator_add_avs(
        &mut self,
        config: &Pubkey,
        operator: &Pubkey,
        avs: &Pubkey,
        operator_avs_ticket: &Pubkey,
        admin: &Keypair,
        payer: &Keypair,
    ) -> Result<(), BanksClientError> {
        let blockhash = self.banks_client.get_latest_blockhash().await?;

        self.process_transaction(&Transaction::new_signed_with_payer(
            &[operator_add_avs(
                &jito_restaking_program::id(),
                config,
                operator,
                avs,
                operator_avs_ticket,
                &admin.pubkey(),
                &payer.pubkey(),
            )],
            Some(&payer.pubkey()),
            &[admin, payer],
            blockhash,
        ))
        .await
    }

    // pub async fn operator_remove_avs(
    //     &mut self,
    //     config: &Pubkey,
    //     operator: &Pubkey,
    //     avs: &Pubkey,
    //     operator_avs_ticket: &Pubkey,
    //     admin: &Keypair,
    // ) -> Result<(), BanksClientError> {
    //     let blockhash = self.banks_client.get_latest_blockhash().await?;
    //
    //     self.process_transaction(&Transaction::new_signed_with_payer(
    //         &[operator_remove_avs(
    //             &jito_restaking_program::id(),
    //             config,
    //             operator,
    //             avs,
    //             operator_avs_ticket,
    //             &admin.pubkey(),
    //         )],
    //         Some(&admin.pubkey()),
    //         &[admin],
    //         blockhash,
    //     ))
    //     .await
    // }
    //
    // pub async fn avs_withdrawal_asset(
    //     &mut self,
    //     avs: &Pubkey,
    //     avs_token_account: &Pubkey,
    //     receiver_token_account: &Pubkey,
    //     admin: &Keypair,
    //     token_program: &Pubkey,
    //     token_mint: Pubkey,
    //     amount: u64,
    // ) -> Result<(), BanksClientError> {
    //     let blockhash = self.banks_client.get_latest_blockhash().await?;
    //
    //     self.process_transaction(&Transaction::new_signed_with_payer(
    //         &[avs_withdrawal_asset(
    //             &jito_restaking_program::id(),
    //             avs,
    //             avs_token_account,
    //             receiver_token_account,
    //             &admin.pubkey(),
    //             token_program,
    //             token_mint,
    //             amount,
    //         )],
    //         Some(&admin.pubkey()),
    //         &[admin],
    //         blockhash,
    //     ))
    //     .await
    // }
    //
    // pub async fn operator_withdrawal_asset(
    //     &mut self,
    //     operator: &Pubkey,
    //     admin: &Keypair,
    //     operator_token_account: &Pubkey,
    //     receiver_token_account: &Pubkey,
    //     token_program: &Pubkey,
    //     token_mint: Pubkey,
    //     amount: u64,
    // ) -> Result<(), BanksClientError> {
    //     let blockhash = self.banks_client.get_latest_blockhash().await?;
    //
    //     self.process_transaction(&Transaction::new_signed_with_payer(
    //         &[operator_withdrawal_asset(
    //             &jito_restaking_program::id(),
    //             operator,
    //             &admin.pubkey(),
    //             operator_token_account,
    //             receiver_token_account,
    //             token_program,
    //             token_mint,
    //             amount,
    //         )],
    //         Some(&admin.pubkey()),
    //         &[admin],
    //         blockhash,
    //     ))
    //     .await
    // }

    pub async fn process_transaction(&mut self, tx: &Transaction) -> Result<(), BanksClientError> {
        self.banks_client
            .process_transaction_with_preflight_and_commitment(
                tx.clone(),
                CommitmentLevel::Processed,
            )
            .await
    }

    pub async fn _airdrop(&mut self, to: &Pubkey, sol: f64) -> Result<(), BanksClientError> {
        let blockhash = self.banks_client.get_latest_blockhash().await?;
        self.banks_client
            .process_transaction_with_preflight_and_commitment(
                Transaction::new_signed_with_payer(
                    &[transfer(&self.payer.pubkey(), to, sol_to_lamports(sol))],
                    Some(&self.payer.pubkey()),
                    &[&self.payer],
                    blockhash,
                ),
                CommitmentLevel::Processed,
            )
            .await
    }
}
