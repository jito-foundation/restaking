#[cfg(test)]
mod tests {
    use jito_restaking_core::config::Config;
    use jito_vault_sdk::error::VaultError;
    use solana_program::instruction::InstructionError;
    use solana_sdk::{
        signature::{Keypair, Signer},
        transaction::TransactionError,
    };
    use spl_associated_token_account::get_associated_token_address;

    use crate::fixtures::{fixture::TestBuilder, vault_client::VaultStakerWithdrawalTicketRoot};

    #[tokio::test]
    async fn test_enqueue_withdraw_more_than_staked_fails() {
        let mut fixture = TestBuilder::new().await;

        let mut vault_program_client = fixture.vault_program_client();

        let (_vault_config_admin, vault_root) = vault_program_client
            .setup_config_and_vault(100, 100)
            .await
            .unwrap();

        let vault = vault_program_client
            .get_vault(&vault_root.vault_pubkey)
            .await
            .unwrap();

        let depositor = Keypair::new();
        fixture.transfer(&depositor.pubkey(), 100.0).await.unwrap();
        fixture
            .mint_to(&vault.supported_mint, &depositor.pubkey(), 100_000)
            .await
            .unwrap();

        fixture
            .create_ata(&vault.vrt_mint, &depositor.pubkey())
            .await
            .unwrap();

        let depositor_vrt_token_account =
            get_associated_token_address(&depositor.pubkey(), &vault.vrt_mint);
        vault_program_client
            .mint_to(
                &vault_root.vault_pubkey,
                &vault.vrt_mint,
                &depositor,
                &get_associated_token_address(&depositor.pubkey(), &vault.supported_mint),
                &get_associated_token_address(&vault_root.vault_pubkey, &vault.supported_mint),
                &get_associated_token_address(&depositor.pubkey(), &vault.vrt_mint),
                &get_associated_token_address(&vault.fee_wallet, &vault.vrt_mint),
                None,
                100_000,
            )
            .await
            .unwrap();

        let depositor_ata = fixture
            .get_token_account(&depositor_vrt_token_account)
            .await
            .unwrap();
        assert_eq!(depositor_ata.amount, 99_000);

        let transaction_error = vault_program_client
            .do_enqueue_withdraw(&vault_root, &depositor, 49_500)
            .await
            .unwrap_err()
            .to_transaction_error()
            .unwrap();
        assert_eq!(
            transaction_error,
            TransactionError::InstructionError(
                0,
                InstructionError::Custom(VaultError::VaultDelegationListUnderflow as u32)
            )
        );
    }

    #[tokio::test]
    async fn test_enqueue_withdraw_with_fee_success() {
        let mut fixture = TestBuilder::new().await;

        let mut vault_program_client = fixture.vault_program_client();
        let mut restaking_program_client = fixture.restaking_program_client();

        let (_vault_config_admin, vault_root) = vault_program_client
            .setup_config_and_vault(100, 100)
            .await
            .unwrap();

        let _restaking_config_admin = restaking_program_client
            .do_initialize_config()
            .await
            .unwrap();

        let operator_root = restaking_program_client
            .do_initialize_operator()
            .await
            .unwrap();
        let ncn_root = restaking_program_client.do_initialize_ncn().await.unwrap();

        let restaking_config = restaking_program_client
            .get_config(&Config::find_program_address(&jito_restaking_program::id()).0)
            .await
            .unwrap();

        // restaking_program_client
        //     .do_initialize_operator_ncn_ticket(&operator_root, &ncn_root.ncn_pubkey)
        //     .await
        //     .unwrap();

        fixture
            .warp_slot_incremental(2 * restaking_config.epoch_length)
            .await
            .unwrap();

        restaking_program_client
            .do_initialize_ncn_operator_state(&ncn_root, &operator_root.operator_pubkey)
            .await
            .unwrap();

        restaking_program_client
            .do_initialize_ncn_vault_ticket(&ncn_root, &vault_root.vault_pubkey)
            .await
            .unwrap();
        restaking_program_client
            .operator_vault_opt_in(&operator_root, &vault_root.vault_pubkey)
            .await
            .unwrap();

        fixture
            .warp_slot_incremental(2 * restaking_config.epoch_length)
            .await
            .unwrap();

        vault_program_client
            .vault_ncn_opt_in(&vault_root, &ncn_root.ncn_pubkey)
            .await
            .unwrap();
        vault_program_client
            .do_initialize_vault_operator_ticket(&vault_root, &operator_root.operator_pubkey)
            .await
            .unwrap();

        fixture
            .warp_slot_incremental(2 * restaking_config.epoch_length)
            .await
            .unwrap();
        vault_program_client
            .do_full_vault_update(&vault_root.vault_pubkey, &[operator_root.operator_pubkey])
            .await
            .unwrap();

        let vault = vault_program_client
            .get_vault(&vault_root.vault_pubkey)
            .await
            .unwrap();

        let depositor = Keypair::new();
        fixture.transfer(&depositor.pubkey(), 100.0).await.unwrap();
        fixture
            .mint_to(&vault.supported_mint, &depositor.pubkey(), 100_000)
            .await
            .unwrap();

        fixture
            .create_ata(&vault.vrt_mint, &depositor.pubkey())
            .await
            .unwrap();

        vault_program_client
            .mint_to(
                &vault_root.vault_pubkey,
                &vault.vrt_mint,
                &depositor,
                &get_associated_token_address(&depositor.pubkey(), &vault.supported_mint),
                &get_associated_token_address(&vault_root.vault_pubkey, &vault.supported_mint),
                &get_associated_token_address(&depositor.pubkey(), &vault.vrt_mint),
                &get_associated_token_address(&vault.fee_wallet, &vault.vrt_mint),
                None,
                100_000,
            )
            .await
            .unwrap();

        let vault_vrt_account = fixture
            .get_token_account(&get_associated_token_address(
                &depositor.pubkey(),
                &vault.vrt_mint,
            ))
            .await
            .unwrap();
        assert_eq!(vault_vrt_account.amount, 99_000);

        let vault_fee_account = fixture
            .get_token_account(&get_associated_token_address(
                &vault.fee_wallet,
                &vault.vrt_mint,
            ))
            .await
            .unwrap();
        assert_eq!(vault_fee_account.amount, 1_000);

        vault_program_client
            .delegate(&vault_root, &operator_root.operator_pubkey, 100_000)
            .await
            .unwrap();

        // TODO (LB): test delegation brother

        // the user is withdrawing 99,000 VRT tokens, there is a 1% fee on withdraws, so
        // 98010 tokens will be undeleged for withdraw
        let VaultStakerWithdrawalTicketRoot { base } = vault_program_client
            .do_enqueue_withdraw(&vault_root, &depositor, 99_000)
            .await
            .unwrap();

        // TODO (LB): test delegation brother

        let vault_staker_withdrawal_ticket = vault_program_client
            .get_vault_staker_withdrawal_ticket(
                &vault_root.vault_pubkey,
                &depositor.pubkey(),
                &base,
            )
            .await
            .unwrap();
        assert_eq!(vault_staker_withdrawal_ticket.vrt_amount, 98_010);
        assert_eq!(
            vault_staker_withdrawal_ticket.withdraw_allocation_amount,
            98_010
        );
    }

    #[tokio::test]
    async fn test_enqueue_withdraw_with_reward_ok() {
        let mut fixture = TestBuilder::new().await;
        let mut vault_program_client = fixture.vault_program_client();
        let mut restaking_program_client = fixture.restaking_program_client();

        // Setup vault with initial deposit
        let (_vault_config_admin, vault_root) = vault_program_client
            .setup_config_and_vault(0, 0)
            .await
            .unwrap();
        let _restaking_config_admin = restaking_program_client
            .do_initialize_config()
            .await
            .unwrap();

        // Setup operator and NCN
        let operator_root = restaking_program_client
            .do_initialize_operator()
            .await
            .unwrap();
        let ncn_root = restaking_program_client.do_initialize_ncn().await.unwrap();

        let restaking_config = restaking_program_client
            .get_config(&Config::find_program_address(&jito_restaking_program::id()).0)
            .await
            .unwrap();

        // Setup necessary relationships
        // restaking_program_client
        //     .do_initialize_operator_ncn_ticket(&operator_root, &ncn_root.ncn_pubkey)
        //     .await
        //     .unwrap();

        fixture
            .warp_slot_incremental(2 * restaking_config.epoch_length)
            .await
            .unwrap();

        restaking_program_client
            .do_initialize_ncn_operator_state(&ncn_root, &operator_root.operator_pubkey)
            .await
            .unwrap();
        restaking_program_client
            .do_initialize_ncn_vault_ticket(&ncn_root, &vault_root.vault_pubkey)
            .await
            .unwrap();

        restaking_program_client
            .operator_vault_opt_in(&operator_root, &vault_root.vault_pubkey)
            .await
            .unwrap();

        fixture
            .warp_slot_incremental(2 * restaking_config.epoch_length)
            .await
            .unwrap();

        vault_program_client
            .vault_ncn_opt_in(&vault_root, &ncn_root.ncn_pubkey)
            .await
            .unwrap();

        vault_program_client
            .do_initialize_vault_operator_ticket(&vault_root, &operator_root.operator_pubkey)
            .await
            .unwrap();

        fixture
            .warp_slot_incremental(2 * restaking_config.epoch_length)
            .await
            .unwrap();

        let vault = vault_program_client
            .get_vault(&vault_root.vault_pubkey)
            .await
            .unwrap();

        // Initial deposit
        let depositor = Keypair::new();
        fixture.transfer(&depositor.pubkey(), 100.0).await.unwrap();
        fixture
            .mint_to(&vault.supported_mint, &depositor.pubkey(), 100_000)
            .await
            .unwrap();
        fixture
            .create_ata(&vault.vrt_mint, &depositor.pubkey())
            .await
            .unwrap();

        // Mint VRT tokens to depositor
        vault_program_client
            .mint_to(
                &vault_root.vault_pubkey,
                &vault.vrt_mint,
                &depositor,
                &get_associated_token_address(&depositor.pubkey(), &vault.supported_mint),
                &get_associated_token_address(&vault_root.vault_pubkey, &vault.supported_mint),
                &get_associated_token_address(&depositor.pubkey(), &vault.vrt_mint),
                &get_associated_token_address(&vault.fee_wallet, &vault.vrt_mint),
                None,
                100_000,
            )
            .await
            .unwrap();

        vault_program_client
            .do_full_vault_update(&vault_root.vault_pubkey, &[operator_root.operator_pubkey])
            .await
            .unwrap();

        // Delegate all funds to the operator
        vault_program_client
            .delegate(&vault_root, &operator_root.operator_pubkey, 100_000)
            .await
            .unwrap();

        // Simulate rewards by adding more tokens to the vault
        fixture
            .mint_to(&vault.supported_mint, &vault_root.vault_pubkey, 10_000)
            .await
            .unwrap();
        vault_program_client
            .do_full_vault_update(&vault_root.vault_pubkey, &[operator_root.operator_pubkey])
            .await
            .unwrap();

        // Enqueue withdrawal for half of the original deposit
        let withdraw_amount = 50_000;
        let VaultStakerWithdrawalTicketRoot { base } = vault_program_client
            .do_enqueue_withdraw(&vault_root, &depositor, withdraw_amount)
            .await
            .unwrap();

        // Verify the withdraw ticket
        let withdrawal_ticket = vault_program_client
            .get_vault_staker_withdrawal_ticket(
                &vault_root.vault_pubkey,
                &depositor.pubkey(),
                &base,
            )
            .await
            .unwrap();

        assert_eq!(withdrawal_ticket.vrt_amount, withdraw_amount);

        // The actual assets to be withdrawn should be more than the VRT amount due to rewards
        assert_eq!(withdrawal_ticket.withdraw_allocation_amount, 55_000);

        // TODO (LB): test delegation brother
    }
}
