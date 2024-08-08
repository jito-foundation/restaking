#[cfg(test)]
mod tests {
    use jito_vault_core::config::Config;
    use solana_program::pubkey::Pubkey;
    use solana_sdk::signature::{Keypair, Signer};
    use spl_associated_token_account::get_associated_token_address;

    use crate::fixtures::{
        fixture::TestBuilder,
        restaking_client::{AvsRoot, OperatorRoot, RestakingProgramClient},
        vault_client::{VaultProgramClient, VaultRoot, VaultStakerWithdrawalTicketRoot},
    };

    struct PreparedWithdrawalTicket {
        vault_root: VaultRoot,
        avs_root: AvsRoot,
        operator_root: OperatorRoot,
        depositor: Keypair,
        withdrawal_ticket_base: Pubkey,
    }

    async fn setup_withdrawal_ticket(
        fixture: &mut TestBuilder,
        vault_program_client: &mut VaultProgramClient,
        restaking_program_client: &mut RestakingProgramClient,
        deposit_fee_bps: u16,
        withdraw_fee_bps: u16,
        mint_amount: u64,
        deposit_amount: u64,
        delegate_amount: u64,
        withdrawal_amount: u64,
    ) -> PreparedWithdrawalTicket {
        // Setup vault with initial deposit
        let (_vault_config_admin, vault_root) = vault_program_client
            .setup_vault(deposit_fee_bps, withdraw_fee_bps)
            .await
            .unwrap();
        let _restaking_config_admin = restaking_program_client.setup_config().await.unwrap();

        // Setup operator and AVS
        let operator_root = restaking_program_client.setup_operator().await.unwrap();
        let avs_root = restaking_program_client.setup_avs().await.unwrap();

        // Setup necessary relationships
        restaking_program_client
            .operator_avs_opt_in(&operator_root, &avs_root.avs_pubkey)
            .await
            .unwrap();
        restaking_program_client
            .avs_operator_opt_in(&avs_root, &operator_root.operator_pubkey)
            .await
            .unwrap();
        restaking_program_client
            .avs_vault_opt_in(&avs_root, &vault_root.vault_pubkey)
            .await
            .unwrap();
        restaking_program_client
            .operator_vault_opt_in(&operator_root, &vault_root.vault_pubkey)
            .await
            .unwrap();
        vault_program_client
            .vault_avs_opt_in(&vault_root, &avs_root.avs_pubkey)
            .await
            .unwrap();
        vault_program_client
            .vault_operator_opt_in(&vault_root, &operator_root.operator_pubkey)
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
            .mint_to(&vault.supported_mint(), &depositor.pubkey(), mint_amount)
            .await
            .unwrap();
        fixture
            .create_ata(&vault.lrt_mint(), &depositor.pubkey())
            .await
            .unwrap();

        // Mint LRT tokens to depositor
        vault_program_client
            .mint_to(
                &vault_root.vault_pubkey,
                &vault.lrt_mint(),
                &depositor,
                &get_associated_token_address(&depositor.pubkey(), &vault.supported_mint()),
                &get_associated_token_address(&vault_root.vault_pubkey, &vault.supported_mint()),
                &get_associated_token_address(&depositor.pubkey(), &vault.lrt_mint()),
                &get_associated_token_address(&vault.fee_owner(), &vault.lrt_mint()),
                None,
                deposit_amount,
            )
            .await
            .unwrap();

        // Delegate all funds to the operator
        vault_program_client
            .delegate(&vault_root, &operator_root.operator_pubkey, delegate_amount)
            .await
            .unwrap();

        let VaultStakerWithdrawalTicketRoot { base } = vault_program_client
            .do_enqueue_withdraw(&vault_root, &depositor, withdrawal_amount)
            .await
            .unwrap();

        PreparedWithdrawalTicket {
            vault_root,
            avs_root,
            operator_root,
            depositor,
            withdrawal_ticket_base: base,
        }
    }

    #[tokio::test]
    async fn test_burn_withdrawal_ticket_same_epoch_fails() {
        let mut fixture = TestBuilder::new().await;
        let mut vault_program_client = fixture.vault_program_client();
        let mut restaking_program_client = fixture.restaking_program_client();

        let PreparedWithdrawalTicket {
            vault_root,
            avs_root: _,
            operator_root: _,
            depositor,
            withdrawal_ticket_base,
        } = setup_withdrawal_ticket(
            &mut fixture,
            &mut vault_program_client,
            &mut restaking_program_client,
            0,
            0,
            1000,
            100,
            100,
            100,
        )
        .await;

        // TODO (LB): check error type
        vault_program_client
            .do_burn_withdrawal_ticket(&vault_root, &depositor, &withdrawal_ticket_base)
            .await
            .unwrap_err();
    }

    #[tokio::test]
    async fn test_burn_withdrawal_ticket_next_epoch_fails() {
        let mut fixture = TestBuilder::new().await;
        let mut vault_program_client = fixture.vault_program_client();
        let mut restaking_program_client = fixture.restaking_program_client();

        let PreparedWithdrawalTicket {
            vault_root,
            avs_root: _,
            operator_root: _,
            depositor,
            withdrawal_ticket_base,
        } = setup_withdrawal_ticket(
            &mut fixture,
            &mut vault_program_client,
            &mut restaking_program_client,
            0,
            0,
            1000,
            100,
            100,
            100,
        )
        .await;

        let config = vault_program_client
            .get_config(&Config::find_program_address(&jito_vault_program::id()).0)
            .await
            .unwrap();

        fixture
            .warp_slot_incremental(config.epoch_length())
            .await
            .unwrap();

        vault_program_client
            .do_update_vault(&vault_root.vault_pubkey)
            .await
            .unwrap();

        vault_program_client
            .do_burn_withdrawal_ticket(&vault_root, &depositor, &withdrawal_ticket_base)
            .await
            .unwrap_err();
    }

    #[tokio::test]
    async fn test_burn_withdrawal_ticket_basic_success() {
        let mut fixture = TestBuilder::new().await;
        let mut vault_program_client = fixture.vault_program_client();
        let mut restaking_program_client = fixture.restaking_program_client();

        let PreparedWithdrawalTicket {
            vault_root,
            avs_root: _,
            operator_root: _,
            depositor,
            withdrawal_ticket_base,
        } = setup_withdrawal_ticket(
            &mut fixture,
            &mut vault_program_client,
            &mut restaking_program_client,
            0,
            0,
            1000,
            1000,
            1000,
            1000,
        )
        .await;

        let config = vault_program_client
            .get_config(&Config::find_program_address(&jito_vault_program::id()).0)
            .await
            .unwrap();

        fixture
            .warp_slot_incremental(2 * config.epoch_length())
            .await
            .unwrap();

        vault_program_client
            .do_update_vault(&vault_root.vault_pubkey)
            .await
            .unwrap();

        vault_program_client
            .do_burn_withdrawal_ticket(&vault_root, &depositor, &withdrawal_ticket_base)
            .await
            .unwrap();

        let vault_delegation_list = vault_program_client
            .get_vault_delegation_list(&vault_root.vault_pubkey)
            .await
            .unwrap();
        assert_eq!(vault_delegation_list.withdrawable_reserve_amount(), 0);

        let vault = vault_program_client
            .get_vault(&vault_root.vault_pubkey)
            .await
            .unwrap();

        let depositor_token_account = fixture
            .get_token_account(&get_associated_token_address(
                &depositor.pubkey(),
                &vault.supported_mint(),
            ))
            .await
            .unwrap();
        assert_eq!(depositor_token_account.amount, 1000);
    }

    #[tokio::test]
    async fn test_burn_withdrawal_ticket_with_unstaked_rewards() {
        let mut fixture = TestBuilder::new().await;
        let mut vault_program_client = fixture.vault_program_client();
        let mut restaking_program_client = fixture.restaking_program_client();

        let PreparedWithdrawalTicket {
            vault_root,
            avs_root: _,
            operator_root: _,
            depositor,
            withdrawal_ticket_base,
        } = setup_withdrawal_ticket(
            &mut fixture,
            &mut vault_program_client,
            &mut restaking_program_client,
            0,
            0,
            1000,
            1000,
            1000,
            1000,
        )
        .await;

        let vault = vault_program_client
            .get_vault(&vault_root.vault_pubkey)
            .await
            .unwrap();

        // send 100 tokens to vault as rewards, increasing value of it by 10%
        fixture
            .mint_to(&vault.supported_mint(), &vault_root.vault_pubkey, 100)
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
            .do_update_vault(&vault_root.vault_pubkey)
            .await
            .unwrap();

        vault_program_client
            .do_burn_withdrawal_ticket(&vault_root, &depositor, &withdrawal_ticket_base)
            .await
            .unwrap();

        let vault_delegation_list = vault_program_client
            .get_vault_delegation_list(&vault_root.vault_pubkey)
            .await
            .unwrap();
        assert_eq!(vault_delegation_list.withdrawable_reserve_amount(), 0);

        // user should have 1100 tokens
        let depositor_token_account = fixture
            .get_token_account(&get_associated_token_address(
                &depositor.pubkey(),
                &vault.supported_mint(),
            ))
            .await
            .unwrap();
        assert_eq!(depositor_token_account.amount, 1100);
    }

    #[tokio::test]
    async fn test_burn_withdrawal_ticket_with_staked_rewards() {
        let mut fixture = TestBuilder::new().await;
        let mut vault_program_client = fixture.vault_program_client();
        let mut restaking_program_client = fixture.restaking_program_client();

        let PreparedWithdrawalTicket {
            vault_root,
            avs_root: _,
            operator_root,
            depositor,
            withdrawal_ticket_base,
        } = setup_withdrawal_ticket(
            &mut fixture,
            &mut vault_program_client,
            &mut restaking_program_client,
            0,
            0,
            1000,
            1000,
            1000,
            1000,
        )
        .await;

        let vault = vault_program_client
            .get_vault(&vault_root.vault_pubkey)
            .await
            .unwrap();

        // send 100 tokens to vault as rewards, increasing value of it by 10%
        // but delegate those to the operator. they won't be available for withdraw
        fixture
            .mint_to(&vault.supported_mint(), &vault_root.vault_pubkey, 100)
            .await
            .unwrap();

        vault_program_client
            .do_update_vault(&vault_root.vault_pubkey)
            .await
            .unwrap();
        vault_program_client
            .delegate(&vault_root, &operator_root.operator_pubkey, 100)
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
            .do_update_vault(&vault_root.vault_pubkey)
            .await
            .unwrap();

        vault_program_client
            .do_burn_withdrawal_ticket(&vault_root, &depositor, &withdrawal_ticket_base)
            .await
            .unwrap();

        let vault_delegation_list = vault_program_client
            .get_vault_delegation_list(&vault_root.vault_pubkey)
            .await
            .unwrap();
        assert_eq!(vault_delegation_list.withdrawable_reserve_amount(), 0);

        // user should have 1000 tokens and should also get back excess LRT tokens
        let depositor_token_account = fixture
            .get_token_account(&get_associated_token_address(
                &depositor.pubkey(),
                &vault.supported_mint(),
            ))
            .await
            .unwrap();
        assert_eq!(depositor_token_account.amount, 1000);

        let depositor_lrt_token_account = fixture
            .get_token_account(&get_associated_token_address(
                &depositor.pubkey(),
                &vault.lrt_mint(),
            ))
            .await
            .unwrap();
        assert_eq!(depositor_lrt_token_account.amount, 91);

        let vault_token_account = fixture
            .get_token_account(&get_associated_token_address(
                &vault_root.vault_pubkey,
                &vault.supported_mint(),
            ))
            .await
            .unwrap();
        assert_eq!(vault_token_account.amount, 100);
    }

    // #[tokio::test]
    // async fn test_burn_withdrawal_ticket_with_slashing_before_update() {
    //     let mut fixture = TestBuilder::new().await;
    //     let mut vault_program_client = fixture.vault_program_client();
    //     let mut restaking_program_client = fixture.restaking_program_client();
    //
    //     let PreparedWithdrawalTicket {
    //         vault_config_admin,
    //         vault_root,
    //         avs_root,
    //         operator_root,
    //         depositor,
    //         withdrawal_ticket_base,
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
    //     )
    //     .await;
    //
    //     let vault = vault_program_client
    //         .get_vault(&vault_root.vault_pubkey)
    //         .await
    //         .unwrap();
    //
    //     // create slasher w/ token account
    //     let slasher = Keypair::new();
    //     fixture.transfer(&slasher.pubkey(), 100.0).await.unwrap();
    //     fixture
    //         .create_ata(&vault.supported_mint(), &slasher.pubkey())
    //         .await
    //         .unwrap();
    //
    //     // do all the opt-in stuff
    //     restaking_program_client
    //         .avs_vault_slasher_opt_in(&avs_root, &vault_root.vault_pubkey, &slasher.pubkey(), 100)
    //         .await
    //         .unwrap();
    //     vault_program_client
    //         .vault_avs_vault_slasher_opt_in(&vault_root, &avs_root.avs_pubkey, &slasher.pubkey())
    //         .await
    //         .unwrap();
    //
    //     vault_program_client
    //         .setup_vault_avs_slasher_operator_ticket(
    //             &vault_root,
    //             &avs_root.avs_pubkey,
    //             &slasher.pubkey(),
    //             &operator_root.operator_pubkey,
    //         )
    //         .await
    //         .unwrap();
    //     vault_program_client
    //         .do_slash(
    //             &vault_root,
    //             &avs_root.avs_pubkey,
    //             &slasher,
    //             &operator_root.operator_pubkey,
    //             100,
    //         )
    //         .await
    //         .unwrap();
    //
    //     let config = vault_program_client
    //         .get_config(&Config::find_program_address(&jito_vault_program::id()).0)
    //         .await
    //         .unwrap();
    //     fixture
    //         .warp_slot_incremental(2 * config.epoch_length())
    //         .await
    //         .unwrap();
    //
    //     vault_program_client
    //         .do_update_vault(&vault_root.vault_pubkey)
    //         .await
    //         .unwrap();
    //
    //     vault_program_client
    //         .do_burn_withdrawal_ticket(&vault_root, &depositor, &withdrawal_ticket_base)
    //         .await
    //         .unwrap();
    // }

    #[tokio::test]
    async fn test_burn_withdrawal_ticket_with_slashing_after_update() {
        let mut fixture = TestBuilder::new().await;
        let mut vault_program_client = fixture.vault_program_client();
        let mut restaking_program_client = fixture.restaking_program_client();

        let PreparedWithdrawalTicket {
            vault_root,
            avs_root,
            operator_root,
            depositor,
            withdrawal_ticket_base,
        } = setup_withdrawal_ticket(
            &mut fixture,
            &mut vault_program_client,
            &mut restaking_program_client,
            0,
            0,
            1000,
            1000,
            1000,
            900,
        )
        .await;

        let vault = vault_program_client
            .get_vault(&vault_root.vault_pubkey)
            .await
            .unwrap();

        // create slasher w/ token account
        let slasher = Keypair::new();
        fixture.transfer(&slasher.pubkey(), 100.0).await.unwrap();
        fixture
            .create_ata(&vault.supported_mint(), &slasher.pubkey())
            .await
            .unwrap();

        // do all the opt-in stuff
        restaking_program_client
            .avs_vault_slasher_opt_in(&avs_root, &vault_root.vault_pubkey, &slasher.pubkey(), 100)
            .await
            .unwrap();
        vault_program_client
            .vault_avs_vault_slasher_opt_in(&vault_root, &avs_root.avs_pubkey, &slasher.pubkey())
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
            .do_update_vault(&vault_root.vault_pubkey)
            .await
            .unwrap();

        vault_program_client
            .setup_vault_avs_slasher_operator_ticket(
                &vault_root,
                &avs_root.avs_pubkey,
                &slasher.pubkey(),
                &operator_root.operator_pubkey,
            )
            .await
            .unwrap();
        vault_program_client
            .do_slash(
                &vault_root,
                &avs_root.avs_pubkey,
                &slasher,
                &operator_root.operator_pubkey,
                100,
            )
            .await
            .unwrap();

        vault_program_client
            .do_burn_withdrawal_ticket(&vault_root, &depositor, &withdrawal_ticket_base)
            .await
            .unwrap();

        let vault_delegation_list = vault_program_client
            .get_vault_delegation_list(&vault_root.vault_pubkey)
            .await
            .unwrap();
        assert_eq!(vault_delegation_list.withdrawable_reserve_amount(), 0);

        let depositor_token_account = fixture
            .get_token_account(&get_associated_token_address(
                &depositor.pubkey(),
                &vault.supported_mint(),
            ))
            .await
            .unwrap();
        assert_eq!(depositor_token_account.amount, 810);

        let depositor_lrt_token_account = fixture
            .get_token_account(&get_associated_token_address(
                &depositor.pubkey(),
                &vault.lrt_mint(),
            ))
            .await
            .unwrap();
        assert_eq!(depositor_lrt_token_account.amount, 100);

        let vault_token_account = fixture
            .get_token_account(&get_associated_token_address(
                &vault_root.vault_pubkey,
                &vault.supported_mint(),
            ))
            .await
            .unwrap();
        assert_eq!(vault_token_account.amount, 90);
    }
}
