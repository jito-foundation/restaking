use borsh::BorshDeserialize;
use jito_restaking_core::{
    config::Config, ncn::Ncn, ncn_operator_ticket::NcnOperatorTicket,
    ncn_vault_slasher_ticket::NcnVaultSlasherTicket, ncn_vault_ticket::NcnVaultTicket,
    operator::Operator, operator_ncn_ticket::OperatorNcnTicket,
    operator_vault_ticket::OperatorVaultTicket,
};
use jito_restaking_sdk::{
    initialize_config, initialize_ncn, initialize_operator, ncn_add_operator, ncn_add_vault,
    ncn_add_vault_slasher, operator_add_ncn, operator_add_vault,
};
use solana_program::{native_token::sol_to_lamports, pubkey::Pubkey, system_instruction::transfer};
use solana_program_test::{BanksClient, BanksClientError};
use solana_sdk::{
    commitment_config::CommitmentLevel,
    signature::{Keypair, Signer},
    transaction::Transaction,
};

pub struct NcnRoot {
    pub ncn_pubkey: Pubkey,
    pub ncn_admin: Keypair,
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

    pub async fn get_ncn(&mut self, ncn: &Pubkey) -> Result<Ncn, BanksClientError> {
        let account = self
            .banks_client
            .get_account_with_commitment(*ncn, CommitmentLevel::Processed)
            .await?
            .unwrap();

        Ok(Ncn::deserialize(&mut account.data.as_slice())?)
    }

    pub async fn get_config(&mut self, account: &Pubkey) -> Result<Config, BanksClientError> {
        let account = self.banks_client.get_account(*account).await?.unwrap();
        Ok(Config::deserialize(&mut account.data.as_slice())?)
    }

    pub async fn get_ncn_vault_ticket(
        &mut self,
        ncn: &Pubkey,
        vault: &Pubkey,
    ) -> Result<NcnVaultTicket, BanksClientError> {
        let account =
            NcnVaultTicket::find_program_address(&jito_restaking_program::id(), &ncn, &vault).0;
        let account = self.banks_client.get_account(account).await?.unwrap();
        Ok(NcnVaultTicket::deserialize(&mut account.data.as_slice())?)
    }

    pub async fn get_ncn_operator_ticket(
        &mut self,
        ncn: &Pubkey,
        operator: &Pubkey,
    ) -> Result<NcnOperatorTicket, BanksClientError> {
        let account =
            NcnOperatorTicket::find_program_address(&jito_restaking_program::id(), &ncn, &operator)
                .0;
        let account = self.banks_client.get_account(account).await?.unwrap();
        Ok(NcnOperatorTicket::deserialize(
            &mut account.data.as_slice(),
        )?)
    }

    pub async fn get_ncn_vault_slasher_ticket(
        &mut self,
        ncn: &Pubkey,
        vault: &Pubkey,
        slasher: &Pubkey,
    ) -> Result<NcnVaultSlasherTicket, BanksClientError> {
        let account = NcnVaultSlasherTicket::find_program_address(
            &jito_restaking_program::id(),
            &ncn,
            &vault,
            &slasher,
        )
        .0;
        let account = self.banks_client.get_account(account).await?.unwrap();
        Ok(NcnVaultSlasherTicket::deserialize(
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

    pub async fn get_operator_ncn_ticket(
        &mut self,
        operator: &Pubkey,
        ncn: &Pubkey,
    ) -> Result<OperatorNcnTicket, BanksClientError> {
        let account =
            OperatorNcnTicket::find_program_address(&jito_restaking_program::id(), &operator, &ncn)
                .0;
        let account = self.banks_client.get_account(account).await?.unwrap();
        Ok(OperatorNcnTicket::deserialize(
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

    pub async fn setup_ncn(&mut self) -> Result<NcnRoot, BanksClientError> {
        let ncn_admin = Keypair::new();
        let ncn_base = Keypair::new();

        self._airdrop(&ncn_admin.pubkey(), 1.0).await?;

        let ncn_pubkey =
            Ncn::find_program_address(&jito_restaking_program::id(), &ncn_base.pubkey()).0;
        self.initialize_ncn(
            &Config::find_program_address(&jito_restaking_program::id()).0,
            &ncn_pubkey,
            &ncn_admin,
            &ncn_base,
        )
        .await
        .unwrap();

        Ok(NcnRoot {
            ncn_pubkey,
            ncn_admin,
        })
    }

    pub async fn ncn_vault_opt_in(
        &mut self,
        ncn_root: &NcnRoot,
        vault: &Pubkey,
    ) -> Result<(), BanksClientError> {
        let ncn_vault_ticket = NcnVaultTicket::find_program_address(
            &jito_restaking_program::id(),
            &ncn_root.ncn_pubkey,
            vault,
        )
        .0;

        self.ncn_add_vault(
            &Config::find_program_address(&jito_restaking_program::id()).0,
            &ncn_root.ncn_pubkey,
            vault,
            &ncn_vault_ticket,
            &ncn_root.ncn_admin,
            &self.payer.insecure_clone(),
        )
        .await
    }

    pub async fn ncn_operator_opt_in(
        &mut self,
        ncn_root: &NcnRoot,
        operator: &Pubkey,
    ) -> Result<(), BanksClientError> {
        let ncn_operator_ticket = NcnOperatorTicket::find_program_address(
            &jito_restaking_program::id(),
            &ncn_root.ncn_pubkey,
            operator,
        )
        .0;
        let operator_ncn_ticket = OperatorNcnTicket::find_program_address(
            &jito_restaking_program::id(),
            operator,
            &ncn_root.ncn_pubkey,
        )
        .0;

        self.ncn_add_operator(
            &Config::find_program_address(&jito_restaking_program::id()).0,
            &ncn_root.ncn_pubkey,
            operator,
            &ncn_operator_ticket,
            &operator_ncn_ticket,
            &ncn_root.ncn_admin,
            &self.payer.insecure_clone(),
        )
        .await
    }

    pub async fn operator_ncn_opt_in(
        &mut self,
        operator_root: &OperatorRoot,
        ncn: &Pubkey,
    ) -> Result<(), BanksClientError> {
        let operator_ncn_ticket = OperatorNcnTicket::find_program_address(
            &jito_restaking_program::id(),
            &operator_root.operator_pubkey,
            ncn,
        )
        .0;

        self.operator_add_ncn(
            &Config::find_program_address(&jito_restaking_program::id()).0,
            &operator_root.operator_pubkey,
            ncn,
            &operator_ncn_ticket,
            &operator_root.operator_admin,
            &operator_root.operator_admin,
        )
        .await
    }

    pub async fn ncn_vault_slasher_opt_in(
        &mut self,
        ncn_root: &NcnRoot,
        vault: &Pubkey,
        slasher: &Pubkey,
        max_slash_amount: u64,
    ) -> Result<(), BanksClientError> {
        let ncn_vault_ticket = NcnVaultTicket::find_program_address(
            &jito_restaking_program::id(),
            &ncn_root.ncn_pubkey,
            vault,
        )
        .0;
        let ncn_slasher_ticket = NcnVaultSlasherTicket::find_program_address(
            &jito_restaking_program::id(),
            &ncn_root.ncn_pubkey,
            vault,
            slasher,
        )
        .0;

        self.ncn_add_vault_slasher(
            &Config::find_program_address(&jito_restaking_program::id()).0,
            &ncn_root.ncn_pubkey,
            vault,
            slasher,
            &ncn_vault_ticket,
            &ncn_slasher_ticket,
            &ncn_root.ncn_admin,
            &self.payer.insecure_clone(),
            max_slash_amount,
        )
        .await
    }

    pub async fn initialize_ncn(
        &mut self,
        config: &Pubkey,
        ncn: &Pubkey,
        ncn_admin: &Keypair,
        ncn_base: &Keypair,
    ) -> Result<(), BanksClientError> {
        let blockhash = self.banks_client.get_latest_blockhash().await?;

        self.process_transaction(&Transaction::new_signed_with_payer(
            &[initialize_ncn(
                &jito_restaking_program::id(),
                &config,
                &ncn,
                &ncn_admin.pubkey(),
                &ncn_base.pubkey(),
            )],
            Some(&ncn_admin.pubkey()),
            &[&ncn_admin, &ncn_base],
            blockhash,
        ))
        .await
    }

    pub async fn ncn_add_vault(
        &mut self,
        config: &Pubkey,
        ncn: &Pubkey,
        vault: &Pubkey,
        ncn_vault_ticket: &Pubkey,
        ncn_admin: &Keypair,
        payer: &Keypair,
    ) -> Result<(), BanksClientError> {
        let blockhash = self.banks_client.get_latest_blockhash().await?;

        self.process_transaction(&Transaction::new_signed_with_payer(
            &[ncn_add_vault(
                &jito_restaking_program::id(),
                config,
                ncn,
                vault,
                ncn_vault_ticket,
                &ncn_admin.pubkey(),
                &payer.pubkey(),
            )],
            Some(&payer.pubkey()),
            &[ncn_admin, payer],
            blockhash,
        ))
        .await
    }

    // pub async fn ncn_remove_vault(
    //     &mut self,
    //     config: &Pubkey,
    //     ncn: &Pubkey,
    //     vault: &Pubkey,
    //     ncn_vault_ticket: &Pubkey,
    //     ncn_admin: &Keypair,
    // ) -> Result<(), BanksClientError> {
    //     let blockhash = self.banks_client.get_latest_blockhash().await?;
    //
    //     self.process_transaction(&Transaction::new_signed_with_payer(
    //         &[ncn_remove_vault(
    //             &jito_restaking_program::id(),
    //             config,
    //             ncn,
    //             vault,
    //             ncn_vault_ticket,
    //             &ncn_admin.pubkey(),
    //         )],
    //         Some(&ncn_admin.pubkey()),
    //         &[ncn_admin],
    //         blockhash,
    //     ))
    //     .await
    // }

    pub async fn ncn_add_operator(
        &mut self,
        config: &Pubkey,
        ncn: &Pubkey,
        operator: &Pubkey,
        ncn_operator_ticket: &Pubkey,
        operator_ncn_ticket: &Pubkey,
        ncn_admin: &Keypair,
        payer: &Keypair,
    ) -> Result<(), BanksClientError> {
        let blockhash = self.banks_client.get_latest_blockhash().await?;

        self.process_transaction(&Transaction::new_signed_with_payer(
            &[ncn_add_operator(
                &jito_restaking_program::id(),
                config,
                ncn,
                operator,
                ncn_operator_ticket,
                operator_ncn_ticket,
                &ncn_admin.pubkey(),
                &payer.pubkey(),
            )],
            Some(&payer.pubkey()),
            &[ncn_admin, payer],
            blockhash,
        ))
        .await
    }

    // pub async fn ncn_remove_operator(
    //     &mut self,
    //     config: &Pubkey,
    //     ncn: &Pubkey,
    //     operator: &Pubkey,
    //     ncn_operator_ticket: &Pubkey,
    //     ncn_admin: &Keypair,
    // ) -> Result<(), BanksClientError> {
    //     let blockhash = self.banks_client.get_latest_blockhash().await?;
    //
    //     self.process_transaction(&Transaction::new_signed_with_payer(
    //         &[ncn_remove_operator(
    //             &jito_restaking_program::id(),
    //             config,
    //             ncn,
    //             operator,
    //             ncn_operator_ticket,
    //             &ncn_admin.pubkey(),
    //         )],
    //         Some(&ncn_admin.pubkey()),
    //         &[ncn_admin],
    //         blockhash,
    //     ))
    //     .await
    // }

    pub async fn ncn_add_vault_slasher(
        &mut self,
        config: &Pubkey,
        ncn: &Pubkey,
        vault: &Pubkey,
        slasher: &Pubkey,
        ncn_vault_ticket: &Pubkey,
        ncn_slasher_ticket: &Pubkey,
        ncn_admin: &Keypair,
        payer: &Keypair,
        max_slash_amount: u64,
    ) -> Result<(), BanksClientError> {
        let blockhash = self.banks_client.get_latest_blockhash().await?;

        self.process_transaction(&Transaction::new_signed_with_payer(
            &[ncn_add_vault_slasher(
                &jito_restaking_program::id(),
                config,
                ncn,
                vault,
                slasher,
                ncn_vault_ticket,
                ncn_slasher_ticket,
                &ncn_admin.pubkey(),
                &payer.pubkey(),
                max_slash_amount,
            )],
            Some(&payer.pubkey()),
            &[ncn_admin, payer],
            blockhash,
        ))
        .await
    }

    // pub async fn ncn_remove_vault_slasher(
    //     &mut self,
    //     config: &Pubkey,
    //     ncn: &Pubkey,
    //     vault: &Pubkey,
    //     slasher: &Pubkey,
    //     ncn_slasher_ticket: &Pubkey,
    //     ncn_admin: &Keypair,
    // ) -> Result<(), BanksClientError> {
    //     let blockhash = self.banks_client.get_latest_blockhash().await?;
    //
    //     self.process_transaction(&Transaction::new_signed_with_payer(
    //         &[ncn_remove_vault_slasher(
    //             &jito_restaking_program::id(),
    //             config,
    //             ncn,
    //             vault,
    //             slasher,
    //             ncn_slasher_ticket,
    //             &ncn_admin.pubkey(),
    //         )],
    //         Some(&ncn_admin.pubkey()),
    //         &[ncn_admin],
    //         blockhash,
    //     ))
    //     .await
    // }
    //
    // pub async fn ncn_set_admin(
    //     &mut self,
    //     ncn: &Pubkey,
    //     old_admin: &Keypair,
    //     new_admin: &Keypair,
    // ) -> Result<(), BanksClientError> {
    //     let blockhash = self.banks_client.get_latest_blockhash().await?;
    //
    //     self.process_transaction(&Transaction::new_signed_with_payer(
    //         &[ncn_set_admin(
    //             &jito_restaking_program::id(),
    //             ncn,
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
    // pub async fn ncn_set_secondary_admin(
    //     &mut self,
    //     ncn: &Pubkey,
    //     admin: &Keypair,
    //     new_admin: &Pubkey,
    //     role: NcnAdminRole,
    // ) -> Result<(), BanksClientError> {
    //     let blockhash = self.banks_client.get_latest_blockhash().await?;
    //
    //     self.process_transaction(&Transaction::new_signed_with_payer(
    //         &[ncn_set_secondary_admin(
    //             &jito_restaking_program::id(),
    //             ncn,
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

    pub async fn operator_add_ncn(
        &mut self,
        config: &Pubkey,
        operator: &Pubkey,
        ncn: &Pubkey,
        operator_ncn_ticket: &Pubkey,
        admin: &Keypair,
        payer: &Keypair,
    ) -> Result<(), BanksClientError> {
        let blockhash = self.banks_client.get_latest_blockhash().await?;

        self.process_transaction(&Transaction::new_signed_with_payer(
            &[operator_add_ncn(
                &jito_restaking_program::id(),
                config,
                operator,
                ncn,
                operator_ncn_ticket,
                &admin.pubkey(),
                &payer.pubkey(),
            )],
            Some(&payer.pubkey()),
            &[admin, payer],
            blockhash,
        ))
        .await
    }

    // pub async fn operator_remove_ncn(
    //     &mut self,
    //     config: &Pubkey,
    //     operator: &Pubkey,
    //     ncn: &Pubkey,
    //     operator_ncn_ticket: &Pubkey,
    //     admin: &Keypair,
    // ) -> Result<(), BanksClientError> {
    //     let blockhash = self.banks_client.get_latest_blockhash().await?;
    //
    //     self.process_transaction(&Transaction::new_signed_with_payer(
    //         &[operator_remove_ncn(
    //             &jito_restaking_program::id(),
    //             config,
    //             operator,
    //             ncn,
    //             operator_ncn_ticket,
    //             &admin.pubkey(),
    //         )],
    //         Some(&admin.pubkey()),
    //         &[admin],
    //         blockhash,
    //     ))
    //     .await
    // }
    //
    // pub async fn ncn_withdrawal_asset(
    //     &mut self,
    //     ncn: &Pubkey,
    //     ncn_token_account: &Pubkey,
    //     receiver_token_account: &Pubkey,
    //     admin: &Keypair,
    //     token_program: &Pubkey,
    //     token_mint: Pubkey,
    //     amount: u64,
    // ) -> Result<(), BanksClientError> {
    //     let blockhash = self.banks_client.get_latest_blockhash().await?;
    //
    //     self.process_transaction(&Transaction::new_signed_with_payer(
    //         &[ncn_withdrawal_asset(
    //             &jito_restaking_program::id(),
    //             ncn,
    //             ncn_token_account,
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
