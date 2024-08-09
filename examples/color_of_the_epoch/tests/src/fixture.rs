use std::{cell::RefCell, rc::Rc};

use anchor_lang::{InstructionData, ToAccountMetas};
use anchor_spl::token::ID as SPL_TOKEN_ID;
use color_of_the_epoch::{
    accounts as color_accounts, derive_color_coin_mint_address, derive_color_of_epoch_address,
    instruction as color_instruction, InitializeColorOfTheEpochParams,
};
use solana_program_test::{ProgramTest, ProgramTestContext};
use solana_sdk::{
    account::Account, instruction::Instruction, pubkey::Pubkey, signature::Keypair, signer::Signer,
    transaction::Transaction,
};

pub struct TestFixture {
    pub ctx: Rc<RefCell<ProgramTestContext>>,
    pub color_of_the_epoch: Pubkey,
    pub color_program_id: Pubkey,
    pub color_coin_mint: Pubkey,
    pub keypair: Keypair,
}

impl TestFixture {
    pub async fn new() -> Self {
        let color_program_id = color_of_the_epoch::ID;

        let mut program = {
            let program = ProgramTest::new("color_of_the_epoch", color_program_id, None);
            // program.add_program("spl_stake_pool", spl_stake_pool::id(), None);
            program
        };

        let color_of_the_epoch = derive_color_of_epoch_address(&color_program_id);
        let color_coin_mint = derive_color_coin_mint_address(&color_program_id);
        let keypair = Keypair::new();

        program.add_account(keypair.pubkey(), system_account(100_000_000_000));

        let ctx = Rc::new(RefCell::new(program.start_with_context().await));
        Self {
            ctx,
            color_of_the_epoch,
            color_program_id,
            color_coin_mint,
            keypair,
        }
    }

    pub async fn initialize_color_of_the_epoch(&self, params: InitializeColorOfTheEpochParams) {
        let ix = Instruction {
            program_id: self.color_program_id,
            accounts: color_accounts::InitializeColorOfTheEpoch {
                color_of_the_epoch: self.color_of_the_epoch,
                color_coin_mint: self.color_coin_mint,
                token_program: SPL_TOKEN_ID,
                system_program: anchor_lang::system_program::ID,
                authority: self.keypair.pubkey(),
            }
            .to_account_metas(None),
            data: color_instruction::InitializeColorOfTheEpoch { params }.data(),
        };

        let transaction = Transaction::new_signed_with_payer(
            &[ix],
            Some(&self.keypair.pubkey()),
            &[&self.keypair],
            self.ctx.borrow().last_blockhash,
        );

        self.submit_transaction_assert_success(transaction).await;
    }

    pub async fn load_and_deserialize<T: anchor_lang::AccountDeserialize>(
        &self,
        address: &Pubkey,
    ) -> T {
        let ai = {
            let mut banks_client = self.ctx.borrow_mut().banks_client.clone();
            banks_client.get_account(*address).await.unwrap().unwrap()
        };

        T::try_deserialize(&mut ai.data.as_slice()).unwrap()
    }

    pub async fn get_account(&self, address: &Pubkey) -> Account {
        let account = {
            let mut banks_client = self.ctx.borrow_mut().banks_client.clone();
            banks_client.get_account(*address).await.unwrap().unwrap()
        };

        account
    }

    pub async fn submit_transaction_assert_success(&self, transaction: Transaction) {
        let process_transaction_result = {
            let mut banks_client = self.ctx.borrow_mut().banks_client.clone();
            banks_client
                .process_transaction_with_preflight(transaction)
                .await
        };

        if let Err(e) = process_transaction_result {
            panic!("Error: {}", e);
        }
    }

    pub async fn submit_transaction_assert_error(
        &self,
        transaction: Transaction,
        error_message: &str,
    ) {
        let process_transaction_result = {
            let mut banks_client = self.ctx.borrow_mut().banks_client.clone();
            banks_client
                .process_transaction_with_preflight(transaction)
                .await
        };

        if let Err(e) = process_transaction_result {
            if !e.to_string().contains(error_message) {
                panic!("Error: {}\n\nDoes not match {}", e, error_message);
            }

            assert!(e.to_string().contains(error_message));
        } else {
            panic!("Error: Transaction succeeded. Expected {}", error_message);
        }
    }
}

pub fn system_account(lamports: u64) -> Account {
    Account {
        lamports,
        owner: anchor_lang::system_program::ID,
        executable: false,
        rent_epoch: 0,
        data: vec![],
    }
}
