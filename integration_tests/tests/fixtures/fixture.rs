use std::fmt::{Debug, Formatter};

use jito_vault_sdk::inline_mpl_token_metadata;
use solana_program::{
    clock::Clock, native_token::sol_to_lamports, pubkey::Pubkey, system_instruction::transfer,
};
use solana_program_test::{processor, BanksClientError, ProgramTest, ProgramTestContext};
use solana_sdk::{
    commitment_config::CommitmentLevel,
    signature::{Keypair, Signer},
    transaction::Transaction,
};
use spl_associated_token_account::{
    get_associated_token_address, instruction::create_associated_token_account_idempotent,
};
use spl_token_2022::extension::{ExtensionType, StateWithExtensionsOwned};

use crate::fixtures::{
    restaking_client::{NcnRoot, OperatorRoot, RestakingProgramClient},
    vault_client::{VaultProgramClient, VaultRoot},
    TestResult,
};

pub struct TestBuilder {
    context: ProgramTestContext,
}

impl Debug for TestBuilder {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "TestBuilder",)
    }
}

pub struct ConfiguredVault {
    pub vault_program_client: VaultProgramClient,
    #[allow(dead_code)]
    pub restaking_program_client: RestakingProgramClient,
    pub vault_config_admin: Keypair,
    pub vault_root: VaultRoot,
    #[allow(dead_code)]
    pub restaking_config_admin: Keypair,
    pub ncn_root: NcnRoot,
    pub operator_roots: Vec<OperatorRoot>,
    pub slashers_amounts: Vec<(Keypair, u64)>,
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
        program_test.prefer_bpf(true);
        program_test.add_program("mpl_token_metadata", inline_mpl_token_metadata::id(), None);

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

    pub async fn create_token_account(
        &mut self,
        token_program_id: &Pubkey,
        account: &Keypair,
        pool_mint: &Pubkey,
        owner: &Pubkey,
        extensions: &[ExtensionType],
    ) -> Result<(), BanksClientError> {
        let blockhash = self.context.banks_client.get_latest_blockhash().await?;
        let rent = self.context.banks_client.get_rent().await?;
        let space =
            ExtensionType::try_calculate_account_len::<spl_token_2022::state::Account>(extensions)
                .map_err(|_e| {
                    BanksClientError::ClientError("failed to try calculate account length")
                })?;
        let account_rent = rent.minimum_balance(space);

        let mut instructions = vec![solana_program::system_instruction::create_account(
            &self.context.payer.pubkey(),
            &account.pubkey(),
            account_rent,
            space as u64,
            token_program_id,
        )];

        for extension in extensions {
            match extension {
                ExtensionType::ImmutableOwner => instructions.push(
                    spl_token_2022::instruction::initialize_immutable_owner(
                        token_program_id,
                        &account.pubkey(),
                    )
                    .unwrap(),
                ),
                ExtensionType::TransferFeeAmount
                | ExtensionType::MemoTransfer
                | ExtensionType::CpiGuard
                | ExtensionType::NonTransferableAccount => (),
                _ => unimplemented!(),
            };
        }

        instructions.push(
            spl_token_2022::instruction::initialize_account(
                token_program_id,
                &account.pubkey(),
                pool_mint,
                &owner,
            )
            .map_err(|_e| BanksClientError::ClientError("failed to initialize account"))?,
        );

        let mut signers = vec![&self.context.payer, account];
        for extension in extensions {
            match extension {
                ExtensionType::MemoTransfer => {
                    signers.push(&self.context.payer);
                    instructions.push(
                spl_token_2022::extension::memo_transfer::instruction::enable_required_transfer_memos(
                    token_program_id,
                    &account.pubkey(),
                    &self.context.payer.pubkey(),
                    &[],
                ).map_err(|_e| BanksClientError::ClientError("failed to enable required transfer memos"))?
                )
                }
                ExtensionType::CpiGuard => {
                    signers.push(&self.context.payer);
                    instructions.push(
                        spl_token_2022::extension::cpi_guard::instruction::enable_cpi_guard(
                            token_program_id,
                            &account.pubkey(),
                            &self.context.payer.pubkey(),
                            &[],
                        )
                        .map_err(|_e| {
                            BanksClientError::ClientError("failed to enable cpi guard")
                        })?,
                    )
                }
                ExtensionType::ImmutableOwner
                | ExtensionType::TransferFeeAmount
                | ExtensionType::NonTransferableAccount => (),
                _ => unimplemented!(),
            }
        }

        let transaction = Transaction::new_signed_with_payer(
            &instructions,
            Some(&self.context.payer.pubkey()),
            &signers,
            blockhash,
        );

        self.context
            .banks_client
            .process_transaction(transaction)
            .await
            .map_err(|e| e.into())
    }

    pub async fn get_token_account(
        &mut self,
        token_account: &Pubkey,
    ) -> Result<spl_token_2022::state::Account, BanksClientError> {
        let account = self
            .context
            .banks_client
            .get_account(*token_account)
            .await?
            .ok_or(BanksClientError::ClientError("failed to get token account"))?;

        let account_info =
            StateWithExtensionsOwned::<spl_token_2022::state::Account>::unpack(account.data)
                .map_err(|_e| BanksClientError::ClientError("failed to unpack"))?;

        Ok(account_info.base)
    }

    pub async fn get_token_mint(
        &mut self,
        token_mint: &Pubkey,
    ) -> Result<spl_token_2022::state::Mint, BanksClientError> {
        let account = self
            .context
            .banks_client
            .get_account(*token_mint)
            .await?
            .ok_or(BanksClientError::ClientError("failed to get token account"))?;

        let account_info =
            StateWithExtensionsOwned::<spl_token_2022::state::Mint>::unpack(account.data)
                .map_err(|_e| BanksClientError::ClientError("failed to unpack"))?;

        Ok(account_info.base)
    }

    /// Mints tokens to an ATA owned by the `to` address
    pub async fn mint_spl_to(
        &mut self,
        mint: &Pubkey,
        to: &Pubkey,
        amount: u64,
        token_program: &Pubkey,
    ) -> Result<(), BanksClientError> {
        let blockhash = self.context.banks_client.get_latest_blockhash().await?;

        let mint_to_ix = if token_program.eq(&spl_token::id()) {
            vec![
                create_associated_token_account_idempotent(
                    &self.context.payer.pubkey(),
                    to,
                    mint,
                    token_program,
                ),
                spl_token::instruction::mint_to(
                    token_program,
                    mint,
                    &get_associated_token_address(to, mint),
                    &self.context.payer.pubkey(),
                    &[],
                    amount,
                )
                .map_err(|_e| BanksClientError::ClientError("failed to mint to"))?,
            ]
        } else {
            vec![spl_token_2022::instruction::mint_to(
                token_program,
                mint,
                &to,
                &self.context.payer.pubkey(),
                &[],
                amount,
            )
            .map_err(|_e| BanksClientError::ClientError("failed to mint to"))?]
        };
        self.context
            .banks_client
            .process_transaction_with_preflight_and_commitment(
                Transaction::new_signed_with_payer(
                    &mint_to_ix,
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

    pub async fn warp_to_slot(&mut self, warp_slot: u64) -> Result<(), BanksClientError> {
        self.context
            .warp_to_slot(warp_slot)
            .map_err(|_| BanksClientError::ClientError("failed to warp slot"))?;
        Ok(())
    }

    pub async fn get_current_slot(&mut self) -> Result<u64, BanksClientError> {
        let clock: Clock = self.context.banks_client.get_sysvar().await?;
        Ok(clock.slot)
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

    /// Configures a vault with an NCN and operators fully configured
    pub async fn setup_vault_with_ncn_and_operators(
        &mut self,
        deposit_fee_bps: u16,
        withdrawal_fee_bps: u16,
        reward_fee_bps: u16,
        num_operators: u16,
        slasher_amounts: &[u64],
    ) -> TestResult<ConfiguredVault> {
        let mut vault_program_client = self.vault_program_client();
        let mut restaking_program_client = self.restaking_program_client();

        let (vault_config_admin, vault_root) = vault_program_client
            .setup_config_and_vault(deposit_fee_bps, withdrawal_fee_bps, reward_fee_bps)
            .await?;
        let restaking_config_admin = restaking_program_client.do_initialize_config().await?;

        let ncn_root = restaking_program_client.do_initialize_ncn().await?;

        // vault <> ncn
        restaking_program_client
            .do_initialize_ncn_vault_ticket(&ncn_root, &vault_root.vault_pubkey)
            .await?;
        self.warp_slot_incremental(1).await.unwrap();
        restaking_program_client
            .do_warmup_ncn_vault_ticket(&ncn_root, &vault_root.vault_pubkey)
            .await?;
        vault_program_client
            .do_initialize_vault_ncn_ticket(&vault_root, &ncn_root.ncn_pubkey)
            .await?;
        self.warp_slot_incremental(1).await.unwrap();
        vault_program_client
            .do_warmup_vault_ncn_ticket(&vault_root, &ncn_root.ncn_pubkey)
            .await?;

        let mut operator_roots = Vec::with_capacity(num_operators as usize);
        for _ in 0..num_operators {
            let operator_root = restaking_program_client.do_initialize_operator().await?;

            // ncn <> operator
            restaking_program_client
                .do_initialize_ncn_operator_state(&ncn_root, &operator_root.operator_pubkey)
                .await?;
            self.warp_slot_incremental(1).await.unwrap();
            restaking_program_client
                .do_ncn_warmup_operator(&ncn_root, &operator_root.operator_pubkey)
                .await?;
            restaking_program_client
                .do_operator_warmup_ncn(&operator_root, &ncn_root.ncn_pubkey)
                .await?;

            // vault <> operator
            restaking_program_client
                .do_initialize_operator_vault_ticket(&operator_root, &vault_root.vault_pubkey)
                .await?;
            self.warp_slot_incremental(1).await.unwrap();
            restaking_program_client
                .do_warmup_operator_vault_ticket(&operator_root, &vault_root.vault_pubkey)
                .await?;
            vault_program_client
                .do_initialize_vault_operator_delegation(
                    &vault_root,
                    &operator_root.operator_pubkey,
                )
                .await?;

            operator_roots.push(operator_root);
        }

        let mut slashers_amounts: Vec<(Keypair, u64)> = Vec::with_capacity(slasher_amounts.len());
        for amount in slasher_amounts {
            let slasher = Keypair::new();
            self.transfer(&slasher.pubkey(), 10.0).await?;

            restaking_program_client
                .do_initialize_ncn_vault_slasher_ticket(
                    &ncn_root,
                    &vault_root.vault_pubkey,
                    &slasher.pubkey(),
                    *amount,
                )
                .await?;
            self.warp_slot_incremental(1).await.unwrap();
            restaking_program_client
                .do_warmup_ncn_vault_slasher_ticket(
                    &ncn_root,
                    &vault_root.vault_pubkey,
                    &slasher.pubkey(),
                )
                .await?;

            vault_program_client
                .do_initialize_vault_ncn_slasher_ticket(
                    &vault_root,
                    &ncn_root.ncn_pubkey,
                    &slasher.pubkey(),
                )
                .await?;
            self.warp_slot_incremental(1).await.unwrap();
            vault_program_client
                .do_warmup_vault_ncn_slasher_ticket(
                    &vault_root,
                    &ncn_root.ncn_pubkey,
                    &slasher.pubkey(),
                )
                .await?;

            slashers_amounts.push((slasher, *amount));
        }

        Ok(ConfiguredVault {
            vault_program_client,
            restaking_program_client,
            vault_root,
            vault_config_admin,
            restaking_config_admin,
            ncn_root,
            operator_roots,
            slashers_amounts,
        })
    }
}
