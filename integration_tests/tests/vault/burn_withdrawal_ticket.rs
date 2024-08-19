#[cfg(test)]
mod tests {
    use jito_vault_core::{config::Config, delegation_state::DelegationState};
    use jito_vault_sdk::error::VaultError;
    use solana_sdk::{signature::Keypair, signer::Signer};
    use spl_associated_token_account::get_associated_token_address;

    use crate::fixtures::{
        fixture::{ConfiguredVault, TestBuilder},
        vault_client::{assert_vault_error, VaultStakerWithdrawalTicketRoot},
    };

    /// One can't burn the withdraw ticket until a full epoch has passed
    #[tokio::test]
    async fn test_burn_withdrawal_ticket_same_epoch_fails() {
        const MINT_AMOUNT: u64 = 100_000;
        const MIN_AMOUNT_OUT: u64 = 100_000;

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
            .setup_vault_with_ncn_and_operators(0, 0, 1, &[])
            .await
            .unwrap();

        let vault = vault_program_client
            .get_vault(&vault_root.vault_pubkey)
            .await
            .unwrap();

        // Initial deposit + mint
        let depositor = Keypair::new();
        fixture.transfer(&depositor.pubkey(), 100.0).await.unwrap();
        fixture
            .mint_spl_to(
                &vault.supported_mint,
                &depositor.pubkey(),
                MINT_AMOUNT,
                &spl_token::id(),
            )
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
                MIN_AMOUNT_OUT,
            )
            .await
            .unwrap();

        let config = vault_program_client
            .get_config(&Config::find_program_address(&jito_vault_program::id()).0)
            .await
            .unwrap();
        fixture
            .warp_slot_incremental(2 * config.epoch_length)
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
            .do_enqueue_withdraw(&vault_root, &depositor, MINT_AMOUNT)
            .await
            .unwrap();

        let transaction_error = vault_program_client
            .do_burn_withdrawal_ticket(&vault_root, &depositor, &base, MINT_AMOUNT)
            .await;
        assert_vault_error(
            transaction_error,
            VaultError::VaultStakerWithdrawalTicketNotWithdrawable,
        );
    }

    /// One can't burn the withdraw ticket until a full epoch has passed
    #[tokio::test]
    async fn test_burn_withdrawal_ticket_next_epoch_fails() {
        const MINT_AMOUNT: u64 = 100_000;
        const MIN_AMOUNT_OUT: u64 = 100_000;

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
            .setup_vault_with_ncn_and_operators(0, 0, 1, &[])
            .await
            .unwrap();

        let vault = vault_program_client
            .get_vault(&vault_root.vault_pubkey)
            .await
            .unwrap();

        // Initial deposit + mint
        let depositor = Keypair::new();
        fixture.transfer(&depositor.pubkey(), 100.0).await.unwrap();
        fixture
            .mint_spl_to(
                &vault.supported_mint,
                &depositor.pubkey(),
                MINT_AMOUNT,
                &spl_token::id(),
            )
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
                MIN_AMOUNT_OUT,
            )
            .await
            .unwrap();

        let config = vault_program_client
            .get_config(&Config::find_program_address(&jito_vault_program::id()).0)
            .await
            .unwrap();
        fixture
            .warp_slot_incremental(2 * config.epoch_length)
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
            .do_enqueue_withdraw(&vault_root, &depositor, MINT_AMOUNT)
            .await
            .unwrap();

        let config = vault_program_client
            .get_config(&Config::find_program_address(&jito_vault_program::id()).0)
            .await
            .unwrap();
        fixture
            .warp_slot_incremental(config.epoch_length)
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
            .do_burn_withdrawal_ticket(&vault_root, &depositor, &base, MINT_AMOUNT)
            .await;
        assert_vault_error(
            transaction_error,
            VaultError::VaultStakerWithdrawalTicketNotWithdrawable,
        );
    }

    /// Tests basic withdraw ticket with no rewards or slashing incidents
    #[tokio::test]
    async fn test_burn_withdrawal_ticket_basic_success() {
        const MINT_AMOUNT: u64 = 100_000;
        const MIN_AMOUNT_OUT: u64 = 100_000;

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
            .setup_vault_with_ncn_and_operators(0, 0, 1, &[])
            .await
            .unwrap();

        let vault = vault_program_client
            .get_vault(&vault_root.vault_pubkey)
            .await
            .unwrap();

        // Initial deposit + mint
        let depositor = Keypair::new();
        fixture.transfer(&depositor.pubkey(), 100.0).await.unwrap();
        fixture
            .mint_spl_to(
                &vault.supported_mint,
                &depositor.pubkey(),
                MINT_AMOUNT,
                &spl_token::id(),
            )
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
                MIN_AMOUNT_OUT,
            )
            .await
            .unwrap();

        let config = vault_program_client
            .get_config(&Config::find_program_address(&jito_vault_program::id()).0)
            .await
            .unwrap();
        fixture
            .warp_slot_incremental(2 * config.epoch_length)
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
            .do_enqueue_withdraw(&vault_root, &depositor, MINT_AMOUNT)
            .await
            .unwrap();

        vault_program_client
            .do_cooldown_delegation(
                &vault_root,
                &operator_roots[0].operator_pubkey,
                MINT_AMOUNT,
                true,
            )
            .await
            .unwrap();

        let config = vault_program_client
            .get_config(&Config::find_program_address(&jito_vault_program::id()).0)
            .await
            .unwrap();
        fixture
            .warp_slot_incremental(config.epoch_length)
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
            .warp_slot_incremental(config.epoch_length)
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
            .do_burn_withdrawal_ticket(&vault_root, &depositor, &base, MINT_AMOUNT)
            .await
            .unwrap();

        let vault = vault_program_client
            .get_vault(&vault_root.vault_pubkey)
            .await
            .unwrap();
        assert_eq!(vault.tokens_deposited, 0);
        assert_eq!(vault.vrt_supply, 0);
        assert_eq!(
            vault.delegation_state,
            DelegationState {
                staked_amount: 0,
                enqueued_for_cooldown_amount: 0,
                cooling_down_amount: 0,
                enqueued_for_withdraw_amount: 0,
                cooling_down_for_withdraw_amount: 0,
            }
        );
        assert_eq!(vault.vrt_enqueued_for_cooldown_amount, 0);
        assert_eq!(vault.vrt_ready_to_claim_amount, 0);
        assert_eq!(vault.vrt_cooling_down_amount, 0);

        let depositor_token_account = fixture
            .get_token_account(&get_associated_token_address(
                &depositor.pubkey(),
                &vault.supported_mint,
            ))
            .await
            .unwrap();
        assert_eq!(depositor_token_account.amount, MINT_AMOUNT);
    }

    /// Tests basic withdraw ticket with no rewards or slashing incidents
    #[tokio::test]
    async fn test_burn_withdrawal_ticket_slippage_fails() {
        const MINT_AMOUNT: u64 = 100_000;
        const MIN_AMOUNT_OUT: u64 = 100_000;

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
            .setup_vault_with_ncn_and_operators(0, 0, 1, &[])
            .await
            .unwrap();

        let vault = vault_program_client
            .get_vault(&vault_root.vault_pubkey)
            .await
            .unwrap();

        // Initial deposit + mint
        let depositor = Keypair::new();
        fixture.transfer(&depositor.pubkey(), 100.0).await.unwrap();
        fixture
            .mint_spl_to(
                &vault.supported_mint,
                &depositor.pubkey(),
                MINT_AMOUNT,
                &spl_token::id(),
            )
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
                MIN_AMOUNT_OUT,
            )
            .await
            .unwrap();

        let config = vault_program_client
            .get_config(&Config::find_program_address(&jito_vault_program::id()).0)
            .await
            .unwrap();
        fixture
            .warp_slot_incremental(2 * config.epoch_length)
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
            .do_enqueue_withdraw(&vault_root, &depositor, MINT_AMOUNT)
            .await
            .unwrap();

        let config = vault_program_client
            .get_config(&Config::find_program_address(&jito_vault_program::id()).0)
            .await
            .unwrap();
        fixture
            .warp_slot_incremental(config.epoch_length)
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
            .warp_slot_incremental(config.epoch_length)
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
            .do_burn_withdrawal_ticket(&vault_root, &depositor, &base, MINT_AMOUNT)
            .await;
        assert_vault_error(result, VaultError::VaultUnderflow);
    }

    // /// The user withdrew at some ratio of the vault, but rewards were accrued so the amount of
    // /// assets the user gets back shall be larger than the amount set aside for withdrawal.
    // /// The rewards were not staked, so they can be fully withdrawn from the vault.
    // #[tokio::test]
    // #[ignore]
    // async fn test_burn_withdrawal_ticket_with_unstaked_rewards() {
    //     let mut fixture = TestBuilder::new().await;
    //     let mut vault_program_client = fixture.vault_program_client();
    //     let mut restaking_program_client = fixture.restaking_program_client();
    //
    //     let PreparedWithdrawalTicket {
    //         vault_root,
    //         ncn_root: _,
    //         operator_root,
    //         depositor,
    //         withdrawal_ticket_base,
    //         slasher: _,
    //     } = setup_withdrawal_ticket(
    //         &mut fixture,
    //         &mut vault_program_client,
    //         &mut restaking_program_client,
    //         0,
    //         0,
    //         1000,
    //         1000,
    //         1000,
    //         1000,
    //         100,
    //     )
    //     .await;
    //
    //     // send 100 tokens to vault as rewards, increasing value of it by 10%
    //     let vault = vault_program_client
    //         .get_vault(&vault_root.vault_pubkey)
    //         .await
    //         .unwrap();
    //     fixture
    //         .mint_to(&vault.supported_mint, &vault_root.vault_pubkey, 100)
    //         .await
    //         .unwrap();
    //
    //     let config = vault_program_client
    //         .get_config(&Config::find_program_address(&jito_vault_program::id()).0)
    //         .await
    //         .unwrap();
    //     fixture
    //         .warp_slot_incremental(2 * config.epoch_length)
    //         .await
    //         .unwrap();
    //     vault_program_client
    //         .do_full_vault_update(&vault_root.vault_pubkey, &[operator_root.operator_pubkey])
    //         .await
    //         .unwrap();
    //
    //     vault_program_client
    //         .do_burn_withdrawal_ticket(&vault_root, &depositor, &withdrawal_ticket_base)
    //         .await
    //         .unwrap();
    //
    //     // user should have 1100 tokens
    //     let depositor_token_account = fixture
    //         .get_token_account(&get_associated_token_address(
    //             &depositor.pubkey(),
    //             &vault.supported_mint,
    //         ))
    //         .await
    //         .unwrap();
    //     assert_eq!(depositor_token_account.amount, 1100);
    // }
    //
    // /// The user withdrew at some ratio of the vault, but rewards were accrued so the amount of
    // /// assets the user gets back shall be larger than the amount set aside for withdrawal. However,
    // /// those rewards were staked, so the user can't receive them. In this case, they shall receive
    // /// back the amount set aside for withdraw and the excess VRT tokens.
    // #[tokio::test]
    // #[ignore]
    // async fn test_burn_withdrawal_ticket_with_staked_rewards() {
    //     let mut fixture = TestBuilder::new().await;
    //     let mut vault_program_client = fixture.vault_program_client();
    //     let mut restaking_program_client = fixture.restaking_program_client();
    //
    //     let PreparedWithdrawalTicket {
    //         vault_root,
    //         ncn_root: _,
    //         operator_root,
    //         depositor,
    //         withdrawal_ticket_base,
    //         slasher: _,
    //     } = setup_withdrawal_ticket(
    //         &mut fixture,
    //         &mut vault_program_client,
    //         &mut restaking_program_client,
    //         0,
    //         0,
    //         1000,
    //         1000,
    //         1000,
    //         1000,
    //         100,
    //     )
    //     .await;
    //
    //     let vault = vault_program_client
    //         .get_vault(&vault_root.vault_pubkey)
    //         .await
    //         .unwrap();
    //
    //     // send 100 tokens to vault as rewards, increasing value of it by 10%
    //     // but delegate those to the operator. they won't be available for withdraw
    //     fixture
    //         .mint_to(&vault.supported_mint, &vault_root.vault_pubkey, 100)
    //         .await
    //         .unwrap();
    //
    //     vault_program_client
    //         .do_full_vault_update(&vault_root.vault_pubkey, &[operator_root.operator_pubkey])
    //         .await
    //         .unwrap();
    //     vault_program_client
    //         .do_add_delegation(&vault_root, &operator_root.operator_pubkey, 100)
    //         .await
    //         .unwrap();
    //
    //     let config = vault_program_client
    //         .get_config(&Config::find_program_address(&jito_vault_program::id()).0)
    //         .await
    //         .unwrap();
    //
    //     fixture
    //         .warp_slot_incremental(2 * config.epoch_length)
    //         .await
    //         .unwrap();
    //     vault_program_client
    //         .do_full_vault_update(&vault_root.vault_pubkey, &[operator_root.operator_pubkey])
    //         .await
    //         .unwrap();
    //
    //     vault_program_client
    //         .do_burn_withdrawal_ticket(&vault_root, &depositor, &withdrawal_ticket_base)
    //         .await
    //         .unwrap();
    //
    //     // user should have 1000 tokens and should also get back excess VRT tokens
    //     let depositor_token_account = fixture
    //         .get_token_account(&get_associated_token_address(
    //             &depositor.pubkey(),
    //             &vault.supported_mint,
    //         ))
    //         .await
    //         .unwrap();
    //     assert_eq!(depositor_token_account.amount, 1000);
    //
    //     let depositor_vrt_token_account = fixture
    //         .get_token_account(&get_associated_token_address(
    //             &depositor.pubkey(),
    //             &vault.vrt_mint,
    //         ))
    //         .await
    //         .unwrap();
    //     assert_eq!(depositor_vrt_token_account.amount, 91);
    //
    //     let vault_token_account = fixture
    //         .get_token_account(&get_associated_token_address(
    //             &vault_root.vault_pubkey,
    //             &vault.supported_mint,
    //         ))
    //         .await
    //         .unwrap();
    //     assert_eq!(vault_token_account.amount, 100);
    // }
    //
    // // /// The user withdrew at some ratio of the vault, but a slashing took place while the withdrawal ticket
    // // /// was maturing. The user gets back less than they originally anticipated and the amount of withdrawal
    // // /// set aside is reduced to 0.
    // // ///
    // // /// This test is more complicated because the withdrawal amount reserved stored in the vault delegation list
    // // /// won't match the withdrawal amount reserved in the withdrawal ticket.
    // // #[tokio::test]
    // // async fn test_burn_withdrawal_ticket_with_slashing_before_update() {
    // //     let mut fixture = TestBuilder::new().await;
    // //     let mut vault_program_client = fixture.vault_program_client();
    // //     let mut restaking_program_client = fixture.restaking_program_client();
    // //
    // //     let PreparedWithdrawalTicket {
    // //         vault_root,
    // //         ncn_root,
    // //         operator_root,
    // //         depositor,
    // //         withdrawal_ticket_base,
    // //         slasher,
    // //     } = setup_withdrawal_ticket(
    // //         &mut fixture,
    // //         &mut vault_program_client,
    // //         &mut restaking_program_client,
    // //         0,
    // //         0,
    // //         1000,
    // //         1000,
    // //         1000,
    // //         1000,
    // //         100,
    // //     )
    // //     .await;
    // //
    // //     let vault = vault_program_client
    // //         .get_vault(&vault_root.vault_pubkey)
    // //         .await
    // //         .unwrap();
    // //
    // //     let config = vault_program_client
    // //         .get_config(&Config::find_program_address(&jito_vault_program::id()).0)
    // //         .await
    // //         .unwrap();
    // //
    // //     vault_program_client
    // //         .setup_vault_ncn_slasher_operator_ticket(
    // //             &vault_root,
    // //             &ncn_root.ncn_pubkey,
    // //             &slasher.pubkey(),
    // //             &operator_root.operator_pubkey,
    // //         )
    // //         .await
    // //         .unwrap();
    // //
    // //     vault_program_client
    // //         .do_update_vault(&vault_root.vault_pubkey)
    // //         .await
    // //         .unwrap();
    // //
    // //     vault_program_client
    // //         .do_slash(
    // //             &vault_root,
    // //             &ncn_root.ncn_pubkey,
    // //             &slasher,
    // //             &operator_root.operator_pubkey,
    // //             100,
    // //         )
    // //         .await
    // //         .unwrap();
    // //
    // //     let config = vault_program_client
    // //         .get_config(&Config::find_program_address(&jito_vault_program::id()).0)
    // //         .await
    // //         .unwrap();
    // //     fixture
    // //         .warp_slot_incremental(2 * config.epoch_length)
    // //         .await
    // //         .unwrap();
    // //
    // //     vault_program_client
    // //         .do_update_vault(&vault_root.vault_pubkey)
    // //         .await
    // //         .unwrap();
    // //
    // //     vault_program_client
    // //         .do_burn_withdrawal_ticket(&vault_root, &depositor, &withdrawal_ticket_base)
    // //         .await
    // //         .unwrap();
    // // }
    //
    // /// The user withdrew at some ratio of the vault, but a slashing took place after the withdrawal ticket
    // /// had matured. The user gets back less than they originally anticipated and the amount of withdrawal
    // /// set aside is reduced to 0.
    // #[tokio::test]
    // #[ignore]
    // async fn test_burn_withdrawal_ticket_with_slashing_after_update() {
    //     let mut fixture = TestBuilder::new().await;
    //     let mut vault_program_client = fixture.vault_program_client();
    //     let mut restaking_program_client = fixture.restaking_program_client();
    //
    //     let PreparedWithdrawalTicket {
    //         vault_root,
    //         ncn_root,
    //         operator_root,
    //         depositor,
    //         withdrawal_ticket_base,
    //         slasher,
    //     } = setup_withdrawal_ticket(
    //         &mut fixture,
    //         &mut vault_program_client,
    //         &mut restaking_program_client,
    //         0,
    //         0,
    //         1000,
    //         1000,
    //         1000,
    //         900,
    //         100,
    //     )
    //     .await;
    //
    //     let vault = vault_program_client
    //         .get_vault(&vault_root.vault_pubkey)
    //         .await
    //         .unwrap();
    //
    //     let config = vault_program_client
    //         .get_config(&Config::find_program_address(&jito_vault_program::id()).0)
    //         .await
    //         .unwrap();
    //     fixture
    //         .warp_slot_incremental(2 * config.epoch_length)
    //         .await
    //         .unwrap();
    //
    //     vault_program_client
    //         .do_full_vault_update(&vault_root.vault_pubkey, &[operator_root.operator_pubkey])
    //         .await
    //         .unwrap();
    //
    //     vault_program_client
    //         .setup_vault_ncn_slasher_operator_ticket(
    //             &vault_root,
    //             &ncn_root.ncn_pubkey,
    //             &slasher.pubkey(),
    //             &operator_root.operator_pubkey,
    //         )
    //         .await
    //         .unwrap();
    //     vault_program_client
    //         .do_slash(
    //             &vault_root,
    //             &ncn_root.ncn_pubkey,
    //             &slasher,
    //             &operator_root.operator_pubkey,
    //             100,
    //         )
    //         .await
    //         .unwrap();
    //
    //     vault_program_client
    //         .do_burn_withdrawal_ticket(&vault_root, &depositor, &withdrawal_ticket_base)
    //         .await
    //         .unwrap();
    //
    //     let depositor_token_account = fixture
    //         .get_token_account(&get_associated_token_address(
    //             &depositor.pubkey(),
    //             &vault.supported_mint,
    //         ))
    //         .await
    //         .unwrap();
    //     assert_eq!(depositor_token_account.amount, 810);
    //
    //     let depositor_vrt_token_account = fixture
    //         .get_token_account(&get_associated_token_address(
    //             &depositor.pubkey(),
    //             &vault.vrt_mint,
    //         ))
    //         .await
    //         .unwrap();
    //     assert_eq!(depositor_vrt_token_account.amount, 100);
    //
    //     let vault_token_account = fixture
    //         .get_token_account(&get_associated_token_address(
    //             &vault_root.vault_pubkey,
    //             &vault.supported_mint,
    //         ))
    //         .await
    //         .unwrap();
    //     assert_eq!(vault_token_account.amount, 90);
    // }
}
