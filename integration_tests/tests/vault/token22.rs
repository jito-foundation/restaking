#[cfg(test)]
mod tests {
    use jito_vault_core::{
        config::Config, vault_ncn_slasher_operator_ticket::VaultNcnSlasherOperatorTicket,
        vault_ncn_slasher_ticket::VaultNcnSlasherTicket,
    };
    use jito_vault_sdk::error::VaultError;
    use solana_sdk::signature::{Keypair, Signer};

    use crate::fixtures::{
        fixture::{ConfiguredVault, TestBuilder},
        vault_client::assert_vault_error,
    };

    #[tokio::test]
    async fn test_slash_ok() {
        let mut fixture = TestBuilder::new().await;

        let token_program = spl_token_2022::id();

        const MAX_SLASH_AMOUNT: u64 = 100;
        const MINT_AMOUNT: u64 = 100_000;
        const DELEGATION_AMOUNT: u64 = 10_000;

        let deposit_fee_bps = 0;
        let withdraw_fee_bps = 0;
        let reward_fee_bps = 0;
        let num_operators = 1;
        let slasher_amounts = vec![MAX_SLASH_AMOUNT];

        let ConfiguredVault {
            mut vault_program_client,
            vault_config_admin,
            vault_root,
            ncn_root,
            operator_roots,
            slashers_amounts,
            ..
        } = fixture
            .setup_vault_with_ncn_and_operators(
                &token_program,
                deposit_fee_bps,
                withdraw_fee_bps,
                reward_fee_bps,
                num_operators,
                &slasher_amounts,
            )
            .await
            .unwrap();

        let depositor = Keypair::new();
        vault_program_client
            .configure_depositor(
                &vault_root,
                &depositor.pubkey(),
                &token_program,
                MINT_AMOUNT,
            )
            .await
            .unwrap();
        vault_program_client
            .do_mint_to(
                &vault_root,
                &depositor,
                &token_program,
                MINT_AMOUNT,
                MINT_AMOUNT,
            )
            .await
            .unwrap();

        let operator_root = &operator_roots[0];
        vault_program_client
            .do_add_delegation(
                &vault_root,
                &operator_root.operator_pubkey,
                DELEGATION_AMOUNT,
            )
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
        let operator_root_pubkeys: Vec<_> =
            operator_roots.iter().map(|r| r.operator_pubkey).collect();
        vault_program_client
            .do_full_vault_update(
                &vault_root.vault_pubkey,
                &operator_root_pubkeys,
                &token_program,
            )
            .await
            .unwrap();

        let vault = vault_program_client
            .get_vault(&vault_root.vault_pubkey)
            .await
            .unwrap();

        // configure slasher and slash
        let slasher = &slashers_amounts[0].0;
        fixture
            .create_ata(&vault.supported_mint, &slasher.pubkey(), &token_program)
            .await
            .unwrap();
        let epoch = fixture.get_current_slot().await.unwrap() / config.epoch_length();
        vault_program_client
            .initialize_vault_ncn_slasher_operator_ticket(
                &Config::find_program_address(&jito_vault_program::id()).0,
                &vault_root.vault_pubkey,
                &ncn_root.ncn_pubkey,
                &slasher.pubkey(),
                &operator_root.operator_pubkey,
                &VaultNcnSlasherTicket::find_program_address(
                    &jito_vault_program::id(),
                    &vault_root.vault_pubkey,
                    &ncn_root.ncn_pubkey,
                    &slasher.pubkey(),
                )
                .0,
                &VaultNcnSlasherOperatorTicket::find_program_address(
                    &jito_vault_program::id(),
                    &vault_root.vault_pubkey,
                    &ncn_root.ncn_pubkey,
                    &slasher.pubkey(),
                    &operator_root.operator_pubkey,
                    epoch,
                )
                .0,
                &vault_config_admin,
            )
            .await
            .unwrap();

        vault_program_client
            .do_slash(
                &vault_root,
                &ncn_root.ncn_pubkey,
                &slasher,
                &operator_root.operator_pubkey,
                &token_program,
                MAX_SLASH_AMOUNT,
            )
            .await
            .unwrap();

        let vault = vault_program_client
            .get_vault(&vault_root.vault_pubkey)
            .await
            .unwrap();
        assert_eq!(vault.tokens_deposited(), MINT_AMOUNT - MAX_SLASH_AMOUNT);
        assert_eq!(
            vault.delegation_state.total_security().unwrap(),
            DELEGATION_AMOUNT - MAX_SLASH_AMOUNT
        );

        let vault_operator_delegation = vault_program_client
            .get_vault_operator_delegation(&vault_root.vault_pubkey, &operator_root.operator_pubkey)
            .await
            .unwrap();

        assert_eq!(
            vault_operator_delegation
                .delegation_state
                .total_security()
                .unwrap(),
            DELEGATION_AMOUNT - MAX_SLASH_AMOUNT
        );

        let epoch = fixture.get_current_slot().await.unwrap() / config.epoch_length();
        let vault_ncn_slasher_operator_ticket = vault_program_client
            .get_vault_ncn_slasher_operator_ticket(
                &vault_root.vault_pubkey,
                &ncn_root.ncn_pubkey,
                &slasher.pubkey(),
                &operator_root.operator_pubkey,
                epoch,
            )
            .await
            .unwrap();
        assert_eq!(vault_ncn_slasher_operator_ticket.slashed(), 100);
        assert_eq!(vault_ncn_slasher_operator_ticket.epoch(), epoch);
        assert_eq!(
            vault_ncn_slasher_operator_ticket.vault,
            vault_root.vault_pubkey
        );
        assert_eq!(vault_ncn_slasher_operator_ticket.ncn, ncn_root.ncn_pubkey);
        assert_eq!(vault_ncn_slasher_operator_ticket.slasher, slasher.pubkey());
        assert_eq!(
            vault_ncn_slasher_operator_ticket.operator,
            operator_root.operator_pubkey
        );
    }

    #[tokio::test]
    async fn test_add_delegation_ok() {
        const AMOUNT_IN: u64 = 100_000;
        const MIN_AMOUNT_OUT: u64 = 100_000;
        let mut fixture = TestBuilder::new().await;

        let token_program = spl_token_2022::id();

        let deposit_fee_bps = 0;
        let withdraw_fee_bps = 0;
        let reward_fee_bps = 0;
        let num_operators = 1;
        let slasher_amounts = vec![];

        let ConfiguredVault {
            mut vault_program_client,
            vault_root,
            operator_roots,
            ..
        } = fixture
            .setup_vault_with_ncn_and_operators(
                &token_program,
                deposit_fee_bps,
                withdraw_fee_bps,
                reward_fee_bps,
                num_operators,
                &slasher_amounts,
            )
            .await
            .unwrap();

        // setup depositor, mint, deposit and delegate
        let depositor = Keypair::new();
        vault_program_client
            .configure_depositor(&vault_root, &depositor.pubkey(), &token_program, 100_000)
            .await
            .unwrap();
        vault_program_client
            .do_mint_to(
                &vault_root,
                &depositor,
                &token_program,
                AMOUNT_IN,
                MIN_AMOUNT_OUT,
            )
            .await
            .unwrap();

        vault_program_client
            .do_add_delegation(&vault_root, &operator_roots[0].operator_pubkey, AMOUNT_IN)
            .await
            .unwrap();

        let vault_operator_delegation = vault_program_client
            .get_vault_operator_delegation(
                &vault_root.vault_pubkey,
                &operator_roots[0].operator_pubkey,
            )
            .await
            .unwrap();
        assert_eq!(vault_operator_delegation.vault, vault_root.vault_pubkey);
        assert_eq!(
            vault_operator_delegation.operator,
            operator_roots[0].operator_pubkey
        );
        assert_eq!(
            vault_operator_delegation
                .delegation_state
                .total_security()
                .unwrap(),
            AMOUNT_IN
        );

        let vault = vault_program_client
            .get_vault(&vault_root.vault_pubkey)
            .await
            .unwrap();
        assert_eq!(
            vault.delegation_state,
            vault_operator_delegation.delegation_state
        );
        assert_eq!(vault.tokens_deposited(), AMOUNT_IN);
        assert_eq!(vault.vrt_supply(), AMOUNT_IN);
    }

    #[tokio::test]
    async fn test_add_delegation_over_delegate_fails() {
        const MINT_AMOUNT: u64 = 100_000;
        const MIN_AMOUNT_OUT: u64 = 100_000;
        let mut fixture = TestBuilder::new().await;

        let token_program = spl_token_2022::id();

        let deposit_fee_bps = 0;
        let withdraw_fee_bps = 0;
        let reward_fee_bps = 0;
        let num_operators = 1;
        let slasher_amounts = vec![];

        let ConfiguredVault {
            mut vault_program_client,
            vault_root,
            operator_roots,
            ..
        } = fixture
            .setup_vault_with_ncn_and_operators(
                &token_program,
                deposit_fee_bps,
                withdraw_fee_bps,
                reward_fee_bps,
                num_operators,
                &slasher_amounts,
            )
            .await
            .unwrap();

        // setup depositor, mint, deposit and delegate
        let depositor = Keypair::new();
        vault_program_client
            .configure_depositor(&vault_root, &depositor.pubkey(), &token_program, 100_000)
            .await
            .unwrap();
        vault_program_client
            .do_mint_to(
                &vault_root,
                &depositor,
                &token_program,
                MINT_AMOUNT,
                MIN_AMOUNT_OUT,
            )
            .await
            .unwrap();

        vault_program_client
            .do_add_delegation(&vault_root, &operator_roots[0].operator_pubkey, 50_000)
            .await
            .unwrap();

        let result = vault_program_client
            .do_add_delegation(&vault_root, &operator_roots[0].operator_pubkey, 50_001)
            .await;
        assert_vault_error(result, VaultError::VaultInsufficientFunds);
    }
}
