use std::fmt::{Debug, Formatter};

use solana_program::{
    native_token::sol_to_lamports,
    program_pack::Pack,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction::{create_account, transfer},
};
use solana_program_test::{processor, BanksClientError, ProgramTest, ProgramTestContext};
use solana_sdk::{
    clock::Clock,
    commitment_config::CommitmentLevel,
    signature::{Keypair, Signer},
    transaction::Transaction,
};
use spl_associated_token_account::{
    get_associated_token_address, instruction::create_associated_token_account_idempotent,
};
use spl_token::{
    instruction::initialize_mint2,
    state::{Account, Mint},
};

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

    // pub async fn store_account<T: BorshSerialize>(
    //     &mut self,
    //     pubkey: &Pubkey,
    //     owner: &Pubkey,
    //     data: &T,
    // ) -> Result<(), BanksClientError> {
    //     let rent: Rent = self.context.banks_client.get_sysvar().await?;
    //
    //     let serialized = data.try_to_vec().unwrap();
    //     let mut data = AccountSharedData::new(
    //         rent.minimum_balance(serialized.len()),
    //         serialized.len(),
    //         owner,
    //     );
    //     data.set_data_from_slice(serialized.as_slice());
    //     self.context.set_account(pubkey, &data);
    //     Ok(())
    // }

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

    // pub async fn get_mint(&mut self, mint: &Pubkey) -> Result<Mint, BanksClientError> {
    //     let account = self.context.banks_client.get_account(*mint).await?.unwrap();
    //     Ok(Mint::unpack(&account.data).unwrap())
    // }

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

    pub async fn create_token_mint(&mut self, mint: &Keypair) -> Result<(), BanksClientError> {
        let blockhash = self.context.banks_client.get_latest_blockhash().await?;
        let rent: Rent = self.context.banks_client.get_sysvar().await?;
        self.context
            .banks_client
            .process_transaction_with_preflight_and_commitment(
                Transaction::new_signed_with_payer(
                    &[
                        create_account(
                            &self.context.payer.pubkey(),
                            &mint.pubkey(),
                            rent.minimum_balance(Mint::LEN),
                            Mint::LEN as u64,
                            &spl_token::id(),
                        ),
                        initialize_mint2(
                            &spl_token::id(),
                            &mint.pubkey(),
                            &self.context.payer.pubkey(),
                            None,
                            9,
                        )
                        .unwrap(),
                    ],
                    Some(&self.context.payer.pubkey()),
                    &[&self.context.payer, mint],
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

    pub async fn get_current_slot(&mut self) -> Result<u64, BanksClientError> {
        let clock: Clock = self.context.banks_client.get_sysvar().await?;
        Ok(clock.slot)
    }

    pub async fn get_current_epoch(&mut self, epoch_length: u64) -> Result<u64, BanksClientError> {
        let current_slot = self.get_current_slot().await?;
        Ok(current_slot / epoch_length)
    }

    // pub async fn warp_to_next_slot(&mut self) -> Result<(), BanksClientError> {
    //     let clock: Clock = self.context.banks_client.get_sysvar().await?;
    //     self.context
    //         .warp_to_slot(clock.slot.checked_add(1).unwrap())
    //         .map_err(|_| BanksClientError::ClientError("failed to warp slot"))?;
    //     Ok(())
    // }

    pub fn vault_program_client(&self) -> VaultProgramClient {
        VaultProgramClient::new(self.context.banks_client.clone())
    }

    pub fn restaking_program_client(&self) -> RestakingProgramClient {
        RestakingProgramClient::new(self.context.banks_client.clone())
    }
}
