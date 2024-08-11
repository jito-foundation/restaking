use jito_account_traits::AccountDeserialize;
use jito_restaking_core::{
    config::Config, ncn::Ncn, ncn_operator_ticket::NcnOperatorTicket,
    ncn_vault_slasher_ticket::NcnVaultSlasherTicket, ncn_vault_ticket::NcnVaultTicket,
    operator::Operator, operator_ncn_ticket::OperatorNcnTicket,
    operator_vault_ticket::OperatorVaultTicket,
};
use jito_restaking_sdk::{
    initialize_config, initialize_ncn, initialize_ncn_operator_ticket,
    initialize_ncn_vault_slasher_ticket, initialize_ncn_vault_ticket, initialize_operator,
    initialize_operator_ncn_ticket, initialize_operator_vault_ticket,
};
use solana_program::{native_token::sol_to_lamports, pubkey::Pubkey, system_instruction::transfer};
use solana_program_test::BanksClient;
use solana_sdk::{
    commitment_config::CommitmentLevel,
    signature::{Keypair, Signer},
    transaction::Transaction,
};

use crate::fixtures::TestError;

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

    pub async fn get_ncn(&mut self, ncn: &Pubkey) -> Result<Ncn, TestError> {
        let account = self
            .banks_client
            .get_account_with_commitment(*ncn, CommitmentLevel::Processed)
            .await?
            .unwrap();

        Ok(Ncn::try_from_slice(&mut account.data.as_slice())?.clone())
    }

    pub async fn get_config(&mut self, account: &Pubkey) -> Result<Config, TestError> {
        let account = self.banks_client.get_account(*account).await?.unwrap();
        Ok(Config::try_from_slice(&mut account.data.as_slice())?.clone())
    }

    pub async fn get_ncn_vault_ticket(
        &mut self,
        ncn: &Pubkey,
        vault: &Pubkey,
    ) -> Result<NcnVaultTicket, TestError> {
        let account =
            NcnVaultTicket::find_program_address(&jito_restaking_program::id(), ncn, vault).0;
        let account = self.banks_client.get_account(account).await?.unwrap();
        Ok(NcnVaultTicket::try_from_slice(&mut account.data.as_slice())?.clone())
    }

    pub async fn get_ncn_operator_ticket(
        &mut self,
        ncn: &Pubkey,
        operator: &Pubkey,
    ) -> Result<NcnOperatorTicket, TestError> {
        let account =
            NcnOperatorTicket::find_program_address(&jito_restaking_program::id(), ncn, operator).0;
        let account = self.banks_client.get_account(account).await?.unwrap();
        Ok(NcnOperatorTicket::try_from_slice(&mut account.data.as_slice())?.clone())
    }

    pub async fn get_ncn_vault_slasher_ticket(
        &mut self,
        ncn: &Pubkey,
        vault: &Pubkey,
        slasher: &Pubkey,
    ) -> Result<NcnVaultSlasherTicket, TestError> {
        let account = NcnVaultSlasherTicket::find_program_address(
            &jito_restaking_program::id(),
            ncn,
            vault,
            slasher,
        )
        .0;
        let account = self.banks_client.get_account(account).await?.unwrap();
        Ok(NcnVaultSlasherTicket::try_from_slice(&mut account.data.as_slice())?.clone())
    }

    pub async fn get_operator(&mut self, account: &Pubkey) -> Result<Operator, TestError> {
        let account = self.banks_client.get_account(*account).await?.unwrap();
        Ok(Operator::try_from_slice(&mut account.data.as_slice())?.clone())
    }

    pub async fn get_operator_vault_ticket(
        &mut self,
        operator: &Pubkey,
        vault: &Pubkey,
    ) -> Result<OperatorVaultTicket, TestError> {
        let account = OperatorVaultTicket::find_program_address(
            &jito_restaking_program::id(),
            operator,
            vault,
        )
        .0;
        let account = self.banks_client.get_account(account).await?.unwrap();
        Ok(OperatorVaultTicket::try_from_slice(&mut account.data.as_slice())?.clone())
    }

    pub async fn get_operator_ncn_ticket(
        &mut self,
        operator: &Pubkey,
        ncn: &Pubkey,
    ) -> Result<OperatorNcnTicket, TestError> {
        let account =
            OperatorNcnTicket::find_program_address(&jito_restaking_program::id(), operator, ncn).0;
        let account = self.banks_client.get_account(account).await?.unwrap();
        Ok(OperatorNcnTicket::try_from_slice(&mut account.data.as_slice())?.clone())
    }

    pub async fn setup_config(&mut self) -> Result<Keypair, TestError> {
        let restaking_config_pubkey = Config::find_program_address(&jito_restaking_program::id()).0;
        let restaking_config_admin = Keypair::new();

        self._airdrop(&restaking_config_admin.pubkey(), 1.0).await?;
        self.initialize_config(&restaking_config_pubkey, &restaking_config_admin)
            .await?;

        Ok(restaking_config_admin)
    }

    pub async fn setup_operator(&mut self) -> Result<OperatorRoot, TestError> {
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
    ) -> Result<(), TestError> {
        let operator_vault_ticket = OperatorVaultTicket::find_program_address(
            &jito_restaking_program::id(),
            &operator_root.operator_pubkey,
            &vault_pubkey,
        )
        .0;
        self.initialize_operator_vault_ticket(
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
    ) -> Result<(), TestError> {
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

    pub async fn setup_ncn(&mut self) -> Result<NcnRoot, TestError> {
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
    ) -> Result<(), TestError> {
        let ncn_vault_ticket = NcnVaultTicket::find_program_address(
            &jito_restaking_program::id(),
            &ncn_root.ncn_pubkey,
            vault,
        )
        .0;

        self.initialize_ncn_vault_ticket(
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
    ) -> Result<(), TestError> {
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

        self.initialize_ncn_operator_ticket(
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
    ) -> Result<(), TestError> {
        let operator_ncn_ticket = OperatorNcnTicket::find_program_address(
            &jito_restaking_program::id(),
            &operator_root.operator_pubkey,
            ncn,
        )
        .0;

        self.initialize_operator_ncn_ticket(
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
    ) -> Result<(), TestError> {
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

        self.initialize_ncn_vault_slasher_ticket(
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
    ) -> Result<(), TestError> {
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

    pub async fn initialize_ncn_vault_ticket(
        &mut self,
        config: &Pubkey,
        ncn: &Pubkey,
        vault: &Pubkey,
        ncn_vault_ticket: &Pubkey,
        ncn_admin: &Keypair,
        payer: &Keypair,
    ) -> Result<(), TestError> {
        let blockhash = self.banks_client.get_latest_blockhash().await?;

        self.process_transaction(&Transaction::new_signed_with_payer(
            &[initialize_ncn_vault_ticket(
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

    pub async fn initialize_ncn_operator_ticket(
        &mut self,
        config: &Pubkey,
        ncn: &Pubkey,
        operator: &Pubkey,
        ncn_operator_ticket: &Pubkey,
        operator_ncn_ticket: &Pubkey,
        ncn_admin: &Keypair,
        payer: &Keypair,
    ) -> Result<(), TestError> {
        let blockhash = self.banks_client.get_latest_blockhash().await?;

        self.process_transaction(&Transaction::new_signed_with_payer(
            &[initialize_ncn_operator_ticket(
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

    pub async fn initialize_ncn_vault_slasher_ticket(
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
    ) -> Result<(), TestError> {
        let blockhash = self.banks_client.get_latest_blockhash().await?;

        self.process_transaction(&Transaction::new_signed_with_payer(
            &[initialize_ncn_vault_slasher_ticket(
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

    pub async fn initialize_operator(
        &mut self,
        config: &Pubkey,
        operator: &Pubkey,
        admin: &Keypair,
        base: &Keypair,
    ) -> Result<(), TestError> {
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

    pub async fn initialize_operator_vault_ticket(
        &mut self,
        config: &Pubkey,
        operator: &Pubkey,
        vault: &Pubkey,
        operator_vault_ticket: &Pubkey,
        admin: &Keypair,
        payer: &Keypair,
    ) -> Result<(), TestError> {
        let blockhash = self.banks_client.get_latest_blockhash().await?;

        self.process_transaction(&Transaction::new_signed_with_payer(
            &[initialize_operator_vault_ticket(
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

    pub async fn initialize_operator_ncn_ticket(
        &mut self,
        config: &Pubkey,
        operator: &Pubkey,
        ncn: &Pubkey,
        operator_ncn_ticket: &Pubkey,
        admin: &Keypair,
        payer: &Keypair,
    ) -> Result<(), TestError> {
        let blockhash = self.banks_client.get_latest_blockhash().await?;

        self.process_transaction(&Transaction::new_signed_with_payer(
            &[initialize_operator_ncn_ticket(
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

    pub async fn process_transaction(&mut self, tx: &Transaction) -> Result<(), TestError> {
        self.banks_client
            .process_transaction_with_preflight_and_commitment(
                tx.clone(),
                CommitmentLevel::Processed,
            )
            .await?;
        Ok(())
    }

    pub async fn _airdrop(&mut self, to: &Pubkey, sol: f64) -> Result<(), TestError> {
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
            .await?;
        Ok(())
    }
}
