use std::fmt::{Debug, Formatter};

use solana_program::{
    clock::Clock, native_token::sol_to_lamports, program_pack::Pack, pubkey::Pubkey,
    system_instruction::transfer,
};
use solana_program_test::{processor, BanksClientError, ProgramTest, ProgramTestContext};
use solana_sdk::{commitment_config::CommitmentLevel, signature::Signer, transaction::Transaction};
use spl_associated_token_account::{
    get_associated_token_address, instruction::create_associated_token_account_idempotent,
};
use spl_token::state::Account;

use crate::fixtures::{restaking_client::RestakingProgramClient, vault_client::VaultProgramClient};

pub struct TestBuilder {
    context: ProgramTestContext,
}

impl Debug for TestBuilder {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "TestBuilder",)
    }
}

impl TestBuilder {
    pub async fn new() -> Self {
        // $ cargo-build-sbf && SBF_OUT_DIR=$(pwd)/target/sbf-solana-solana/release cargo nextest run
        let mut program_test = ProgramTest::new(
            "jito_vault_program",
            jito_vault_program::id(),
            processor!(jito_vault_program::process_instruction),
        );
        program_test.add_program(
            "jito_restaking_program",
            jito_restaking_program::id(),
            processor!(jito_restaking_program::process_instruction),
        );

        let context = program_test.start_with_context().await;
        Self { context }
    }

    pub async fn transfer(&mut self, to: &Pubkey, sol: f64) -> Result<(), BanksClientError> {
        let blockhash = self.context.banks_client.get_latest_blockhash().await?;
        self.context
            .banks_client
            .process_transaction_with_preflight_and_commitment(
                Transaction::new_signed_with_payer(
                    &[transfer(
                        &self.context.payer.pubkey(),
                        to,
                        sol_to_lamports(sol),
                    )],
                    Some(&self.context.payer.pubkey()),
                    &[&self.context.payer],
                    blockhash,
                ),
                CommitmentLevel::Processed,
            )
            .await
    }

    pub async fn get_token_account(
        &mut self,
        token_account: &Pubkey,
    ) -> Result<Account, BanksClientError> {
        let account = self
            .context
            .banks_client
            .get_account(*token_account)
            .await?
            .unwrap();
        Ok(Account::unpack(&account.data).unwrap())
    }

    /// Mints tokens to an ATA owned by the `to` address
    pub async fn mint_to(
        &mut self,
        mint: &Pubkey,
        to: &Pubkey,
        amount: u64,
    ) -> Result<(), BanksClientError> {
        let blockhash = self.context.banks_client.get_latest_blockhash().await?;
        self.context
            .banks_client
            .process_transaction_with_preflight_and_commitment(
                Transaction::new_signed_with_payer(
                    &[
                        create_associated_token_account_idempotent(
                            &self.context.payer.pubkey(),
                            to,
                            mint,
                            &spl_token::id(),
                        ),
                        spl_token::instruction::mint_to(
                            &spl_token::id(),
                            mint,
                            &get_associated_token_address(to, mint),
                            &self.context.payer.pubkey(),
                            &[],
                            amount,
                        )
                        .unwrap(),
                    ],
                    Some(&self.context.payer.pubkey()),
                    &[&self.context.payer],
                    blockhash,
                ),
                CommitmentLevel::Processed,
            )
            .await
    }

    pub async fn create_ata(
        &mut self,
        mint: &Pubkey,
        owner: &Pubkey,
    ) -> Result<(), BanksClientError> {
        let blockhash = self.context.banks_client.get_latest_blockhash().await?;
        self.context
            .banks_client
            .process_transaction_with_preflight_and_commitment(
                Transaction::new_signed_with_payer(
                    &[create_associated_token_account_idempotent(
                        &self.context.payer.pubkey(),
                        owner,
                        mint,
                        &spl_token::id(),
                    )],
                    Some(&self.context.payer.pubkey()),
                    &[&self.context.payer],
                    blockhash,
                ),
                CommitmentLevel::Processed,
            )
            .await
    }

    pub async fn warp_slot_incremental(
        &mut self,
        incremental_slots: u64,
    ) -> Result<(), BanksClientError> {
        let clock: Clock = self.context.banks_client.get_sysvar().await?;
        self.context
            .warp_to_slot(clock.slot.checked_add(incremental_slots).unwrap())
            .map_err(|_| BanksClientError::ClientError("failed to warp slot"))?;
        Ok(())
    }

    pub fn vault_program_client(&self) -> VaultProgramClient {
        VaultProgramClient::new(
            self.context.banks_client.clone(),
            self.context.payer.insecure_clone(),
        )
    }

    pub fn restaking_program_client(&self) -> RestakingProgramClient {
        RestakingProgramClient::new(
            self.context.banks_client.clone(),
            self.context.payer.insecure_clone(),
        )
    }
}
