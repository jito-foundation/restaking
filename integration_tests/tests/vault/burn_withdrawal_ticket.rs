#[cfg(test)]
mod tests {
    use jito_vault_core::{
        config::Config, delegation_state::DelegationState,
        vault_staker_withdrawal_ticket::VaultStakerWithdrawalTicket,
    };
    use jito_vault_sdk::error::VaultError;
    use solana_program::pubkey::Pubkey;
    use solana_sdk::{instruction::InstructionError, signature::Keypair, signer::Signer};
    use spl_associated_token_account::get_associated_token_address;

    use crate::fixtures::{
        assert_ix_error,
        fixture::{ConfiguredVault, TestBuilder},
        vault_client::{assert_vault_error, VaultStakerWithdrawalTicketRoot},
    };

    /// One can't burn the withdrawal ticket until a full epoch has passed
    #[tokio::test]
    async fn test_burn_withdrawal_ticket_same_epoch_fails() {
        const MINT_AMOUNT: u64 = 100_000;

        let deposit_fee_bps = 0;
        let withdrawal_fee_bps = 0;
        let reward_fee_bps = 0;
        let num_operators = 1;
        let slasher_amounts = vec![];

        let mut fixture = TestBuilder::new().await;
        let ConfiguredVault {
            mut vault_program_client,
            restaking_program_client: _,
            vault_config_admin: _,
            vault_root,
            restaking_config_admin: _,
            ncn_root: _,
            operator_roots,
            slashers_amounts: _,
        } = fixture
            .setup_vault_with_ncn_and_operators(
                deposit_fee_bps,
                withdrawal_fee_bps,
                reward_fee_bps,
                num_operators,
                &slasher_amounts,
            )
            .await
            .unwrap();

        // Initial deposit + mint
        let depositor = Keypair::new();
        vault_program_client
            .configure_depositor(&vault_root, &depositor.pubkey(), MINT_AMOUNT)
            .await
            .unwrap();
        vault_program_client
            .do_mint_to(&vault_root, &depositor, MINT_AMOUNT, MINT_AMOUNT)
            .await
            .unwrap();

        let config = vault_program_client
            .get_config(&Config::find_program_address(&jito_vault_program::id()).0)
            .await
            .unwrap();
        fixture
            .warp_slot_incremental(2 * config.epoch_length())
            .await
            .unwrap();

        vault_program_client
            .do_full_vault_update(
                &vault_root.vault_pubkey,
                &[operator_roots[0].operator_pubkey],
            )
            .await
            .unwrap();

        // Delegate all funds to the operator
        vault_program_client
            .do_add_delegation(&vault_root, &operator_roots[0].operator_pubkey, MINT_AMOUNT)
            .await
            .unwrap();

        let VaultStakerWithdrawalTicketRoot { base } = vault_program_client
            .do_enqueue_withdrawal(&vault_root, &depositor, MINT_AMOUNT)
            .await
            .unwrap();

        let transaction_error = vault_program_client
            .do_burn_withdrawal_ticket(&vault_root, &depositor, &base, &config.program_fee_wallet)
            .await;
        assert_vault_error(
            transaction_error,
            VaultError::VaultStakerWithdrawalTicketNotWithdrawable,
        );
    }

    /// One can't burn the withdrawal ticket until a full epoch has passed
    #[tokio::test]
    async fn test_burn_withdrawal_ticket_next_epoch_fails() {
        const MINT_AMOUNT: u64 = 100_000;

        let deposit_fee_bps = 0;
        let withdrawal_fee_bps = 0;
        let reward_fee_bps = 0;
        let num_operators = 1;
        let slasher_amounts = vec![];

        let mut fixture = TestBuilder::new().await;
        let ConfiguredVault {
            mut vault_program_client,
            restaking_program_client: _,
            vault_config_admin: _,
            vault_root,
            restaking_config_admin: _,
            ncn_root: _,
            operator_roots,
            slashers_amounts: _,
        } = fixture
            .setup_vault_with_ncn_and_operators(
                deposit_fee_bps,
                withdrawal_fee_bps,
                reward_fee_bps,
                num_operators,
                &slasher_amounts,
            )
            .await
            .unwrap();

        // Initial deposit + mint
        let depositor = Keypair::new();
        vault_program_client
            .configure_depositor(&vault_root, &depositor.pubkey(), MINT_AMOUNT)
            .await
            .unwrap();
        vault_program_client
            .do_mint_to(&vault_root, &depositor, MINT_AMOUNT, MINT_AMOUNT)
            .await
            .unwrap();

        let config = vault_program_client
            .get_config(&Config::find_program_address(&jito_vault_program::id()).0)
            .await
            .unwrap();
        fixture
            .warp_slot_incremental(2 * config.epoch_length())
            .await
            .unwrap();

        vault_program_client
            .do_full_vault_update(
                &vault_root.vault_pubkey,
                &[operator_roots[0].operator_pubkey],
            )
            .await
            .unwrap();

        // Delegate all funds to the operator
        vault_program_client
            .do_add_delegation(&vault_root, &operator_roots[0].operator_pubkey, MINT_AMOUNT)
            .await
            .unwrap();

        let VaultStakerWithdrawalTicketRoot { base } = vault_program_client
            .do_enqueue_withdrawal(&vault_root, &depositor, MINT_AMOUNT)
            .await
            .unwrap();

        let config = vault_program_client
            .get_config(&Config::find_program_address(&jito_vault_program::id()).0)
            .await
            .unwrap();
        fixture
            .warp_slot_incremental(config.epoch_length())
            .await
            .unwrap();
        vault_program_client
            .do_full_vault_update(
                &vault_root.vault_pubkey,
                &[operator_roots[0].operator_pubkey],
            )
            .await
            .unwrap();

        let transaction_error = vault_program_client
            .do_burn_withdrawal_ticket(&vault_root, &depositor, &base, &config.program_fee_wallet)
            .await;
        assert_vault_error(
            transaction_error,
            VaultError::VaultStakerWithdrawalTicketNotWithdrawable,
        );
    }

    /// Tests basic withdrawal ticket with no rewards or slashing incidents
    #[tokio::test]
    async fn test_burn_withdrawal_ticket_basic_success() {
        const MINT_AMOUNT: u64 = 100_000;

        let deposit_fee_bps = 0;
        let withdraw_fee_bps = 0;
        let reward_fee_bps = 0;
        let num_operators = 1;
        let slasher_amounts = vec![];

        let mut fixture = TestBuilder::new().await;
        let ConfiguredVault {
            mut vault_program_client,
            restaking_program_client: _,
            vault_config_admin: _,
            vault_root,
            restaking_config_admin: _,
            ncn_root: _,
            operator_roots,
            slashers_amounts: _,
        } = fixture
            .setup_vault_with_ncn_and_operators(
                deposit_fee_bps,
                withdraw_fee_bps,
                reward_fee_bps,
                num_operators,
                &slasher_amounts,
            )
            .await
            .unwrap();

        // Initial deposit + mint
        let depositor = Keypair::new();
        vault_program_client
            .configure_depositor(&vault_root, &depositor.pubkey(), MINT_AMOUNT)
            .await
            .unwrap();
        vault_program_client
            .do_mint_to(&vault_root, &depositor, MINT_AMOUNT, MINT_AMOUNT)
            .await
            .unwrap();

        let config = vault_program_client
            .get_config(&Config::find_program_address(&jito_vault_program::id()).0)
            .await
            .unwrap();
        // fixture
        //     .warp_slot_incremental(2 * config.epoch_length())
        //     .await
        //     .unwrap();
        //
        // vault_program_client
        //     .do_full_vault_update(
        //         &vault_root.vault_pubkey,
        //         &[operator_roots[0].operator_pubkey],
        //     )
        //     .await
        //     .unwrap();

        // Delegate all funds to the operator
        vault_program_client
            .do_add_delegation(&vault_root, &operator_roots[0].operator_pubkey, MINT_AMOUNT)
            .await
            .unwrap();

        let VaultStakerWithdrawalTicketRoot { base } = vault_program_client
            .do_enqueue_withdrawal(&vault_root, &depositor, MINT_AMOUNT)
            .await
            .unwrap();

        vault_program_client
            .do_cooldown_delegation(&vault_root, &operator_roots[0].operator_pubkey, MINT_AMOUNT)
            .await
            .unwrap();

        fixture
            .warp_slot_incremental(config.epoch_length())
            .await
            .unwrap();
        vault_program_client
            .do_full_vault_update(
                &vault_root.vault_pubkey,
                &[operator_roots[0].operator_pubkey],
            )
            .await
            .unwrap();
        fixture
            .warp_slot_incremental(config.epoch_length())
            .await
            .unwrap();
        vault_program_client
            .do_full_vault_update(
                &vault_root.vault_pubkey,
                &[operator_roots[0].operator_pubkey],
            )
            .await
            .unwrap();

        vault_program_client
            .do_burn_withdrawal_ticket(&vault_root, &depositor, &base, &config.program_fee_wallet)
            .await
            .unwrap();

        let vault = vault_program_client
            .get_vault(&vault_root.vault_pubkey)
            .await
            .unwrap();
        assert_eq!(vault.tokens_deposited(), 0);
        assert_eq!(vault.vrt_supply(), 0);
        assert_eq!(vault.delegation_state, DelegationState::default());
        assert_eq!(vault.vrt_enqueued_for_cooldown_amount(), 0);
        assert_eq!(vault.vrt_ready_to_claim_amount(), 0);
        assert_eq!(vault.vrt_cooling_down_amount(), 0);

        let depositor_token_account = fixture
            .get_token_account(&get_associated_token_address(
                &depositor.pubkey(),
                &vault.supported_mint,
            ))
            .await
            .unwrap();
        assert_eq!(depositor_token_account.amount, MINT_AMOUNT);
        // assert!(false);
    }

    /// Tests basic withdrawal ticket with no rewards or slashing incidents
    #[tokio::test]
    async fn test_burn_withdrawal_ticket_slippage_fails() {
        const MINT_AMOUNT: u64 = 100_000;

        let deposit_fee_bps = 0;
        let withdraw_fee_bps = 0;
        let reward_fee_bps = 0;
        let num_operators = 1;
        let slasher_amounts = vec![];

        let mut fixture = TestBuilder::new().await;
        let ConfiguredVault {
            mut vault_program_client,
            restaking_program_client: _,
            vault_config_admin: _,
            vault_root,
            restaking_config_admin: _,
            ncn_root: _,
            operator_roots,
            slashers_amounts: _,
        } = fixture
            .setup_vault_with_ncn_and_operators(
                deposit_fee_bps,
                withdraw_fee_bps,
                reward_fee_bps,
                num_operators,
                &slasher_amounts,
            )
            .await
            .unwrap();

        // Initial deposit + mint
        let depositor = Keypair::new();
        vault_program_client
            .configure_depositor(&vault_root, &depositor.pubkey(), MINT_AMOUNT)
            .await
            .unwrap();
        vault_program_client
            .do_mint_to(&vault_root, &depositor, MINT_AMOUNT, MINT_AMOUNT)
            .await
            .unwrap();

        let config = vault_program_client
            .get_config(&Config::find_program_address(&jito_vault_program::id()).0)
            .await
            .unwrap();
        fixture
            .warp_slot_incremental(2 * config.epoch_length())
            .await
            .unwrap();

        vault_program_client
            .do_full_vault_update(
                &vault_root.vault_pubkey,
                &[operator_roots[0].operator_pubkey],
            )
            .await
            .unwrap();

        // Delegate all funds to the operator
        vault_program_client
            .do_add_delegation(&vault_root, &operator_roots[0].operator_pubkey, MINT_AMOUNT)
            .await
            .unwrap();

        let vault = vault_program_client
            .get_vault(&vault_root.vault_pubkey)
            .await
            .unwrap();

        let VaultStakerWithdrawalTicketRoot { base } = vault_program_client
            .do_enqueue_withdrawal(&vault_root, &depositor, MINT_AMOUNT)
            .await
            .unwrap();

        let config = vault_program_client
            .get_config(&Config::find_program_address(&jito_vault_program::id()).0)
            .await
            .unwrap();

        let result = vault_program_client
            .do_burn_withdrawal_ticket(&vault_root, &depositor, &base, &config.program_fee_wallet)
            .await;
        assert_vault_error(
            result,
            VaultError::VaultStakerWithdrawalTicketNotWithdrawable,
        );

        fixture
            .warp_slot_incremental(config.epoch_length())
            .await
            .unwrap();
        vault_program_client
            .do_full_vault_update(
                &vault_root.vault_pubkey,
                &[operator_roots[0].operator_pubkey],
            )
            .await
            .unwrap();
        let result = vault_program_client
            .do_burn_withdrawal_ticket(&vault_root, &depositor, &base, &config.program_fee_wallet)
            .await;
        assert_vault_error(
            result,
            VaultError::VaultStakerWithdrawalTicketNotWithdrawable,
        );

        fixture
            .warp_slot_incremental(config.epoch_length())
            .await
            .unwrap();
        vault_program_client
            .do_full_vault_update(
                &vault_root.vault_pubkey,
                &[operator_roots[0].operator_pubkey],
            )
            .await
            .unwrap();

        vault_program_client
            .do_burn_withdrawal_ticket(&vault_root, &depositor, &base, &config.program_fee_wallet)
            .await
            .unwrap();
        let staker_token_account = fixture
            .get_token_account(&get_associated_token_address(
                &depositor.pubkey(),
                &vault.supported_mint,
            ))
            .await
            .unwrap();
        assert_eq!(staker_token_account.amount, MINT_AMOUNT);
        let vault = vault_program_client
            .get_vault(&vault_root.vault_pubkey)
            .await
            .unwrap();
        assert_eq!(vault.tokens_deposited(), 0);
        assert_eq!(vault.vrt_supply(), 0);
        assert_eq!(vault.delegation_state, DelegationState::default());
        assert_eq!(vault.vrt_enqueued_for_cooldown_amount(), 0);
        assert_eq!(vault.vrt_ready_to_claim_amount(), 0);
        assert_eq!(vault.vrt_cooling_down_amount(), 0);
    }

    /// Tests basic withdraw ticket with no rewards or slashing incidents
    #[tokio::test]
    async fn test_burn_withdrawal_ticket_min_amount_out_fails() {
        const MINT_AMOUNT: u64 = 100_000;

        let deposit_fee_bps = 0;
        let withdrawal_fee_bps = 0;
        let reward_fee_bps = 0;
        let num_operators = 1;
        let slasher_amounts = vec![];

        let mut fixture = TestBuilder::new().await;
        let ConfiguredVault {
            mut vault_program_client,
            restaking_program_client: _,
            vault_config_admin: _,
            vault_root,
            restaking_config_admin: _,
            ncn_root: _,
            operator_roots,
            slashers_amounts: _,
        } = fixture
            .setup_vault_with_ncn_and_operators(
                deposit_fee_bps,
                withdrawal_fee_bps,
                reward_fee_bps,
                num_operators,
                &slasher_amounts,
            )
            .await
            .unwrap();

        // Initial deposit + mint
        let depositor = Keypair::new();
        vault_program_client
            .configure_depositor(&vault_root, &depositor.pubkey(), MINT_AMOUNT)
            .await
            .unwrap();
        vault_program_client
            .do_mint_to(&vault_root, &depositor, MINT_AMOUNT, MINT_AMOUNT)
            .await
            .unwrap();
        let config = vault_program_client
            .get_config(&Config::find_program_address(&jito_vault_program::id()).0)
            .await
            .unwrap();

        let VaultStakerWithdrawalTicketRoot { base } = vault_program_client
            .do_enqueue_withdrawal(&vault_root, &depositor, MINT_AMOUNT)
            .await
            .unwrap();

        fixture
            .warp_slot_incremental(2 * config.epoch_length())
            .await
            .unwrap();

        vault_program_client
            .do_full_vault_update(
                &vault_root.vault_pubkey,
                &[operator_roots[0].operator_pubkey],
            )
            .await
            .unwrap();

        let vault = vault_program_client
            .get_vault(&vault_root.vault_pubkey)
            .await
            .unwrap();

        let random_pubkey = Pubkey::new_unique();
        fixture
            .create_ata(&vault.supported_mint, &random_pubkey)
            .await
            .unwrap();

        let vault_staker_withdrawal_ticket = VaultStakerWithdrawalTicket::find_program_address(
            &jito_vault_program::id(),
            &vault_root.vault_pubkey,
            &base,
        )
        .0;

        let result = vault_program_client
            .burn_withdrawal_ticket(
                &Config::find_program_address(&jito_vault_program::id()).0,
                &vault_root.vault_pubkey,
                &get_associated_token_address(&vault_root.vault_pubkey, &vault.supported_mint),
                &vault.vrt_mint,
                &random_pubkey,
                &get_associated_token_address(&random_pubkey, &vault.supported_mint),
                &vault_staker_withdrawal_ticket,
                &get_associated_token_address(&vault_staker_withdrawal_ticket, &vault.vrt_mint),
                &get_associated_token_address(&vault.fee_wallet, &vault.vrt_mint),
                &get_associated_token_address(&config.program_fee_wallet, &vault.vrt_mint),
            )
            .await;
        assert_vault_error(result, VaultError::VaultStakerWithdrawalTicketInvalidStaker);
    }

    /// Tests that the program fee is correctly deducted and transferred during burn_withdrawal_ticket
    #[tokio::test]
    async fn test_burn_withdrawal_ticket_program_fee() {
        const MINT_AMOUNT: u64 = 100_000;
        const PROGRAM_FEE_BPS: u16 = 10; // 0.1%

        let deposit_fee_bps = 0;
        let withdraw_fee_bps = 0;
        let reward_fee_bps = 0;
        let num_operators = 1;
        let slasher_amounts = vec![];

        let mut fixture = TestBuilder::new().await;
        let ConfiguredVault {
            mut vault_program_client,
            restaking_program_client: _,
            vault_config_admin,
            vault_root,
            restaking_config_admin: _,
            ncn_root: _,
            operator_roots,
            slashers_amounts: _,
        } = fixture
            .setup_vault_with_ncn_and_operators(
                deposit_fee_bps,
                withdraw_fee_bps,
                reward_fee_bps,
                num_operators,
                &slasher_amounts,
            )
            .await
            .unwrap();

        // Set program fee
        vault_program_client
            .set_program_fee(&vault_config_admin, PROGRAM_FEE_BPS)
            .await
            .unwrap();

        // Initial deposit + mint
        let depositor = Keypair::new();
        vault_program_client
            .configure_depositor(&vault_root, &depositor.pubkey(), MINT_AMOUNT)
            .await
            .unwrap();
        vault_program_client
            .do_mint_to(&vault_root, &depositor, MINT_AMOUNT, MINT_AMOUNT)
            .await
            .unwrap();

        let config = vault_program_client
            .get_config(&Config::find_program_address(&jito_vault_program::id()).0)
            .await
            .unwrap();

        // Delegate all funds to the operator
        vault_program_client
            .do_add_delegation(&vault_root, &operator_roots[0].operator_pubkey, MINT_AMOUNT)
            .await
            .unwrap();

        let VaultStakerWithdrawalTicketRoot { base } = vault_program_client
            .do_enqueue_withdrawal(&vault_root, &depositor, MINT_AMOUNT)
            .await
            .unwrap();

        // If this breaks: MINT_AMOUNT * 9990 / 10000 - 1,
        vault_program_client
            .do_cooldown_delegation(&vault_root, &operator_roots[0].operator_pubkey, MINT_AMOUNT)
            .await
            .unwrap();

        // Warp forward two epochs
        fixture
            .warp_slot_incremental(config.epoch_length())
            .await
            .unwrap();
        vault_program_client
            .do_full_vault_update(
                &vault_root.vault_pubkey,
                &[operator_roots[0].operator_pubkey],
            )
            .await
            .unwrap();
        fixture
            .warp_slot_incremental(config.epoch_length())
            .await
            .unwrap();
        vault_program_client
            .do_full_vault_update(
                &vault_root.vault_pubkey,
                &[operator_roots[0].operator_pubkey],
            )
            .await
            .unwrap();

        let vault = vault_program_client
            .get_vault(&vault_root.vault_pubkey)
            .await
            .unwrap();

        let initial_depositor_balance = fixture
            .get_token_account(&get_associated_token_address(
                &depositor.pubkey(),
                &vault.supported_mint,
            ))
            .await
            .unwrap()
            .amount;

        vault_program_client
            .do_burn_withdrawal_ticket(&vault_root, &depositor, &base, &config.program_fee_wallet)
            .await
            .unwrap();

        // Calculate expected fee
        let expected_fee = (MINT_AMOUNT as u128 * PROGRAM_FEE_BPS as u128 / 10000) as u64;
        let expected_withdrawal = MINT_AMOUNT - expected_fee;

        // Check final balances
        let final_depositor_balance = fixture
            .get_token_account(&get_associated_token_address(
                &depositor.pubkey(),
                &vault.supported_mint,
            ))
            .await
            .unwrap()
            .amount;
        let program_fee_balance = fixture
            .get_token_account(&get_associated_token_address(
                &config.program_fee_wallet,
                &vault.vrt_mint,
            ))
            .await
            .unwrap()
            .amount;

        // Assert correct amounts were transferred
        assert_eq!(
            final_depositor_balance - initial_depositor_balance,
            expected_withdrawal
        );
        assert_eq!(program_fee_balance, expected_fee);

        // Check that the vault state is correct
        let vault = vault_program_client
            .get_vault(&vault_root.vault_pubkey)
            .await
            .unwrap();
        assert_eq!(vault.tokens_deposited(), expected_fee);
        assert_eq!(vault.vrt_supply(), expected_fee);
        assert_eq!(vault.delegation_state, DelegationState::default());
        assert_eq!(vault.vrt_enqueued_for_cooldown_amount(), 0);
        assert_eq!(vault.vrt_ready_to_claim_amount(), 0);
        assert_eq!(vault.vrt_cooling_down_amount(), 0);
    }

    /// Tests that burn_withdrawal_ticket fails if the program fee account is not the ATA of the program fee wallet
    #[tokio::test]
    async fn test_burn_withdrawal_ticket_invalid_program_fee_account() {
        const MINT_AMOUNT: u64 = 100_000;
        const PROGRAM_FEE_BPS: u16 = 10; // 0.1%

        let deposit_fee_bps = 0;
        let withdraw_fee_bps = 0;
        let reward_fee_bps = 0;
        let num_operators = 1;
        let slasher_amounts = vec![];

        let mut fixture = TestBuilder::new().await;
        let ConfiguredVault {
            mut vault_program_client,
            restaking_program_client: _,
            vault_config_admin,
            vault_root,
            restaking_config_admin: _,
            ncn_root: _,
            operator_roots,
            slashers_amounts: _,
        } = fixture
            .setup_vault_with_ncn_and_operators(
                deposit_fee_bps,
                withdraw_fee_bps,
                reward_fee_bps,
                num_operators,
                &slasher_amounts,
            )
            .await
            .unwrap();

        // Set program fee
        vault_program_client
            .set_program_fee(&vault_config_admin, PROGRAM_FEE_BPS)
            .await
            .unwrap();

        // Initial deposit + mint
        let depositor = Keypair::new();
        vault_program_client
            .configure_depositor(&vault_root, &depositor.pubkey(), MINT_AMOUNT)
            .await
            .unwrap();
        vault_program_client
            .do_mint_to(&vault_root, &depositor, MINT_AMOUNT, MINT_AMOUNT)
            .await
            .unwrap();

        let config = vault_program_client
            .get_config(&Config::find_program_address(&jito_vault_program::id()).0)
            .await
            .unwrap();

        // Delegate all funds to the operator
        vault_program_client
            .do_add_delegation(&vault_root, &operator_roots[0].operator_pubkey, MINT_AMOUNT)
            .await
            .unwrap();

        let VaultStakerWithdrawalTicketRoot { base } = vault_program_client
            .do_enqueue_withdrawal(&vault_root, &depositor, MINT_AMOUNT)
            .await
            .unwrap();

        vault_program_client
            .do_cooldown_delegation(&vault_root, &operator_roots[0].operator_pubkey, MINT_AMOUNT)
            .await
            .unwrap();

        // If this breaks: do individual warps

        fixture
            .warp_slot_incremental(2 * config.epoch_length())
            .await
            .unwrap();

        vault_program_client
            .do_full_vault_update(
                &vault_root.vault_pubkey,
                &[operator_roots[0].operator_pubkey],
            )
            .await
            .unwrap();

        let vault = vault_program_client
            .get_vault(&vault_root.vault_pubkey)
            .await
            .unwrap();

        // Create an invalid program fee account
        let invalid_program_fee_keypair = Keypair::new();
        fixture
            .create_ata(&vault.supported_mint, &invalid_program_fee_keypair.pubkey())
            .await
            .unwrap();
        let invalid_program_fee_account = get_associated_token_address(
            &invalid_program_fee_keypair.pubkey(),
            &vault.supported_mint,
        );

        // Attempt to burn withdrawal ticket with invalid program fee account
        let result = vault_program_client
            .burn_withdrawal_ticket(
                &Config::find_program_address(&jito_vault_program::id()).0,
                &vault_root.vault_pubkey,
                &get_associated_token_address(&vault_root.vault_pubkey, &vault.supported_mint),
                &vault.vrt_mint,
                &depositor.pubkey(),
                &get_associated_token_address(&depositor.pubkey(), &vault.supported_mint),
                &VaultStakerWithdrawalTicket::find_program_address(
                    &jito_vault_program::id(),
                    &vault_root.vault_pubkey,
                    &base,
                )
                .0,
                &get_associated_token_address(
                    &VaultStakerWithdrawalTicket::find_program_address(
                        &jito_vault_program::id(),
                        &vault_root.vault_pubkey,
                        &base,
                    )
                    .0,
                    &vault.vrt_mint,
                ),
                &get_associated_token_address(&vault.fee_wallet, &vault.vrt_mint),
                &invalid_program_fee_account,
            )
            .await;

        // Assert that the instruction failed with the expected error
        assert_ix_error(result, InstructionError::InvalidAccountData);
    }

    #[tokio::test]
    async fn test_burn_withdrawal_ticket_vault_is_paused_fails() {
        const MINT_AMOUNT: u64 = 100_000;

        let deposit_fee_bps = 0;
        let withdraw_fee_bps = 0;
        let reward_fee_bps = 0;
        let num_operators = 1;
        let slasher_amounts = vec![];

        let mut fixture = TestBuilder::new().await;
        let ConfiguredVault {
            mut vault_program_client,
            restaking_program_client: _,
            vault_config_admin: _,
            vault_root,
            restaking_config_admin: _,
            ncn_root: _,
            operator_roots,
            slashers_amounts: _,
        } = fixture
            .setup_vault_with_ncn_and_operators(
                deposit_fee_bps,
                withdraw_fee_bps,
                reward_fee_bps,
                num_operators,
                &slasher_amounts,
            )
            .await
            .unwrap();

        // Initial deposit + mint
        let depositor = Keypair::new();
        vault_program_client
            .configure_depositor(&vault_root, &depositor.pubkey(), MINT_AMOUNT)
            .await
            .unwrap();
        vault_program_client
            .do_mint_to(&vault_root, &depositor, MINT_AMOUNT, MINT_AMOUNT)
            .await
            .unwrap();

        let config = vault_program_client
            .get_config(&Config::find_program_address(&jito_vault_program::id()).0)
            .await
            .unwrap();

        let VaultStakerWithdrawalTicketRoot { base } = vault_program_client
            .do_enqueue_withdrawal(&vault_root, &depositor, MINT_AMOUNT)
            .await
            .unwrap();

        fixture
            .warp_slot_incremental(2 * config.epoch_length())
            .await
            .unwrap();

        vault_program_client
            .do_full_vault_update(
                &vault_root.vault_pubkey,
                &[operator_roots[0].operator_pubkey],
            )
            .await
            .unwrap();

        let vault = vault_program_client
            .get_vault(&vault_root.vault_pubkey)
            .await
            .unwrap();

        let random_pubkey = Pubkey::new_unique();
        fixture
            .create_ata(&vault.supported_mint, &random_pubkey)
            .await
            .unwrap();

        let vault_staker_withdrawal_ticket = VaultStakerWithdrawalTicket::find_program_address(
            &jito_vault_program::id(),
            &vault_root.vault_pubkey,
            &base,
        )
        .0;

        vault_program_client
            .set_is_paused(&vault_root.vault_pubkey, &vault_root.vault_admin, true)
            .await
            .unwrap();

        let result = vault_program_client
            .burn_withdrawal_ticket(
                &Config::find_program_address(&jito_vault_program::id()).0,
                &vault_root.vault_pubkey,
                &get_associated_token_address(&vault_root.vault_pubkey, &vault.supported_mint),
                &vault.vrt_mint,
                &random_pubkey,
                &get_associated_token_address(&random_pubkey, &vault.supported_mint),
                &vault_staker_withdrawal_ticket,
                &get_associated_token_address(&vault_staker_withdrawal_ticket, &vault.vrt_mint),
                &get_associated_token_address(&vault.fee_wallet, &vault.vrt_mint),
                &get_associated_token_address(&config.program_fee_wallet, &vault.vrt_mint),
            )
            .await;

        assert_vault_error(result, VaultError::VaultIsPaused);
    }
}
