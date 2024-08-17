#[cfg(test)]
mod tests {
    use jito_vault_core::config::Config;
    use solana_sdk::signature::{Keypair, Signer};
    use spl_associated_token_account::get_associated_token_address;

    use crate::fixtures::{
        fixture::{ConfiguredVault, TestBuilder},
        vault_client::VaultStakerWithdrawalTicketRoot,
    };

    #[tokio::test]
    async fn test_enqueue_withdraw_with_fee_success() {
        const MINT_AMOUNT: u64 = 100_000;
        const DEPOSIT_FEE_BPS: u16 = 100;
        const WITHDRAW_FEE_BPS: u16 = 100;

        let mut fixture = TestBuilder::new().await;
        let ConfiguredVault {
            mut vault_program_client,
            vault_root,
            operator_roots,
            ..
        } = fixture
            .setup_vault_with_ncn_and_operators(DEPOSIT_FEE_BPS, WITHDRAW_FEE_BPS, 1, &[])
            .await
            .unwrap();

        let vault = vault_program_client
            .get_vault(&vault_root.vault_pubkey)
            .await
            .unwrap();

        let depositor = Keypair::new();
        fixture.transfer(&depositor.pubkey(), 100.0).await.unwrap();
        fixture
            .mint_to(&vault.supported_mint, &depositor.pubkey(), MINT_AMOUNT)
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
                MINT_AMOUNT,
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
        assert_eq!(
            vault_vrt_account.amount,
            MINT_AMOUNT * (10_000 - DEPOSIT_FEE_BPS) as u64 / 10_000
        );

        let vault_fee_account = fixture
            .get_token_account(&get_associated_token_address(
                &vault.fee_wallet,
                &vault.vrt_mint,
            ))
            .await
            .unwrap();
        assert_eq!(
            vault_fee_account.amount,
            MINT_AMOUNT * DEPOSIT_FEE_BPS as u64 / 10_000
        );

        // let vault operator ticket warmup
        let config = vault_program_client
            .get_config(&Config::find_program_address(&jito_vault_program::id()).0)
            .await
            .unwrap();
        fixture
            .warp_slot_incremental(2 * config.epoch_length)
            .await
            .unwrap();

        let operator_root_pubkeys: Vec<_> = operator_roots
            .iter()
            .map(|root| root.operator_pubkey)
            .collect();
        vault_program_client
            .do_full_vault_update(&vault_root.vault_pubkey, &operator_root_pubkeys)
            .await
            .unwrap();

        let operator_root = operator_roots.first().unwrap();
        vault_program_client
            .do_add_delegation(&vault_root, &operator_root.operator_pubkey, MINT_AMOUNT)
            .await
            .unwrap();

        let vault_operator_delegation = vault_program_client
            .get_vault_operator_delegation(&vault_root.vault_pubkey, &operator_root.operator_pubkey)
            .await
            .unwrap();
        assert_eq!(
            vault_operator_delegation.delegation_state.staked_amount,
            MINT_AMOUNT
        );

        // the user is withdrawing 99,000 VRT tokens, there is a 1% fee on withdraws, so
        // 98010 tokens will be undeleged for withdraw
        let amount_to_dequeue = MINT_AMOUNT * (10_000 - WITHDRAW_FEE_BPS) as u64 / 10_000;
        let VaultStakerWithdrawalTicketRoot { base } = vault_program_client
            .do_enqueue_withdraw(&vault_root, &depositor, amount_to_dequeue)
            .await
            .unwrap();

        let user_vrt_in_withdrawal_ticket =
            amount_to_dequeue * (10_000 - WITHDRAW_FEE_BPS) as u64 / 10_000;
        let vault_staker_withdrawal_ticket = vault_program_client
            .get_vault_staker_withdrawal_ticket(
                &vault_root.vault_pubkey,
                &depositor.pubkey(),
                &base,
            )
            .await
            .unwrap();
        assert_eq!(
            vault_staker_withdrawal_ticket.vrt_amount,
            user_vrt_in_withdrawal_ticket
        );

        let vault = vault_program_client
            .get_vault(&vault_root.vault_pubkey)
            .await
            .unwrap();
        assert_eq!(vault.vrt_pending_withdrawal, user_vrt_in_withdrawal_ticket);
    }

    // #[tokio::test]
    // async fn test_enqueue_withdraw_with_reward_ok() {
    //     let mut fixture = TestBuilder::new().await;
    //     let mut vault_program_client = fixture.vault_program_client();
    //     let mut restaking_program_client = fixture.restaking_program_client();
    //
    //     // Setup vault with initial deposit
    //     let (_vault_config_admin, vault_root) = vault_program_client
    //         .setup_config_and_vault(0, 0)
    //         .await
    //         .unwrap();
    //     let _restaking_config_admin = restaking_program_client
    //         .do_initialize_config()
    //         .await
    //         .unwrap();
    //
    //     // Setup operator and NCN
    //     let operator_root = restaking_program_client
    //         .do_initialize_operator()
    //         .await
    //         .unwrap();
    //     let ncn_root = restaking_program_client.do_initialize_ncn().await.unwrap();
    //
    //     let restaking_config = restaking_program_client
    //         .get_config(&Config::find_program_address(&jito_restaking_program::id()).0)
    //         .await
    //         .unwrap();
    //
    //     // Setup necessary relationships
    //     // restaking_program_client
    //     //     .do_initialize_operator_ncn_ticket(&operator_root, &ncn_root.ncn_pubkey)
    //     //     .await
    //     //     .unwrap();
    //
    //     fixture
    //         .warp_slot_incremental(2 * restaking_config.epoch_length)
    //         .await
    //         .unwrap();
    //
    //     restaking_program_client
    //         .do_initialize_ncn_operator_state(&ncn_root, &operator_root.operator_pubkey)
    //         .await
    //         .unwrap();
    //     restaking_program_client
    //         .do_initialize_ncn_vault_ticket(&ncn_root, &vault_root.vault_pubkey)
    //         .await
    //         .unwrap();
    //
    //     restaking_program_client
    //         .do_initialize_operator_vault_ticket(&operator_root, &vault_root.vault_pubkey)
    //         .await
    //         .unwrap();
    //
    //     fixture
    //         .warp_slot_incremental(2 * restaking_config.epoch_length)
    //         .await
    //         .unwrap();
    //
    //     vault_program_client
    //         .do_initialize_vault_ncn_ticket(&vault_root, &ncn_root.ncn_pubkey)
    //         .await
    //         .unwrap();
    //
    //     vault_program_client
    //         .do_initialize_vault_operator_delegation(&vault_root, &operator_root.operator_pubkey)
    //         .await
    //         .unwrap();
    //
    //     fixture
    //         .warp_slot_incremental(2 * restaking_config.epoch_length)
    //         .await
    //         .unwrap();
    //     vault_program_client
    //         .do_full_vault_update(&vault_root.vault_pubkey, &[operator_root.operator_pubkey])
    //         .await
    //         .unwrap();
    //
    //     let vault = vault_program_client
    //         .get_vault(&vault_root.vault_pubkey)
    //         .await
    //         .unwrap();
    //
    //     // Initial deposit
    //     let depositor = Keypair::new();
    //     fixture.transfer(&depositor.pubkey(), 100.0).await.unwrap();
    //     fixture
    //         .mint_to(&vault.supported_mint, &depositor.pubkey(), 100_000)
    //         .await
    //         .unwrap();
    //     fixture
    //         .create_ata(&vault.vrt_mint, &depositor.pubkey())
    //         .await
    //         .unwrap();
    //
    //     // Mint VRT tokens to depositor
    //     vault_program_client
    //         .mint_to(
    //             &vault_root.vault_pubkey,
    //             &vault.vrt_mint,
    //             &depositor,
    //             &get_associated_token_address(&depositor.pubkey(), &vault.supported_mint),
    //             &get_associated_token_address(&vault_root.vault_pubkey, &vault.supported_mint),
    //             &get_associated_token_address(&depositor.pubkey(), &vault.vrt_mint),
    //             &get_associated_token_address(&vault.fee_wallet, &vault.vrt_mint),
    //             None,
    //             100_000,
    //         )
    //         .await
    //         .unwrap();
    //
    //     // Delegate all funds to the operator
    //     vault_program_client
    //         .do_add_delegation(&vault_root, &operator_root.operator_pubkey, 100_000)
    //         .await
    //         .unwrap();
    //
    //     // Simulate rewards by adding more tokens to the vault
    //     fixture
    //         .mint_to(&vault.supported_mint, &vault_root.vault_pubkey, 10_000)
    //         .await
    //         .unwrap();
    //
    //     // Enqueue withdrawal for half of the original deposit
    //     let withdraw_amount = 50_000;
    //     let VaultStakerWithdrawalTicketRoot { base } = vault_program_client
    //         .do_enqueue_withdraw(&vault_root, &depositor, withdraw_amount)
    //         .await
    //         .unwrap();
    //
    //     // Verify the withdraw ticket
    //     let withdrawal_ticket = vault_program_client
    //         .get_vault_staker_withdrawal_ticket(
    //             &vault_root.vault_pubkey,
    //             &depositor.pubkey(),
    //             &base,
    //         )
    //         .await
    //         .unwrap();
    //
    //     assert_eq!(withdrawal_ticket.vrt_amount, withdraw_amount);
    //
    //     // TODO (LB): test delegation brother
    // }
}
