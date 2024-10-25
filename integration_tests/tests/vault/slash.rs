#[cfg(test)]
mod tests {
    use jito_vault_core::{
        config::Config, vault::Vault,
        vault_ncn_slasher_operator_ticket::VaultNcnSlasherOperatorTicket,
        vault_ncn_slasher_ticket::VaultNcnSlasherTicket,
    };
    use jito_vault_sdk::error::VaultError;
    use solana_sdk::signature::{Keypair, Signer};

    use crate::fixtures::{
        fixture::{ConfiguredVault, TestBuilder},
        vault_client::assert_vault_error,
    };

    const MAX_SLASH_AMOUNT: u64 = 100;
    const MINT_AMOUNT: u64 = 100_000;
    const DELEGATION_AMOUNT: u64 = 10_000;

    const ZERO_DEPOSIT_FEE_BPS: u16 = 0;
    const ZERO_WITHDRAW_FEE_BPS: u16 = 0;
    const ZERO_REWARD_FEE_BPS: u16 = 0;
    const ONE_OPERATOR: u16 = 1;
    const SLASHER_AMOUNTS: &[u64] = &[MAX_SLASH_AMOUNT];

    #[tokio::test]
    async fn test_slash_ok() {
        let mut fixture = TestBuilder::new().await;

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
                ZERO_DEPOSIT_FEE_BPS,
                ZERO_WITHDRAW_FEE_BPS,
                ZERO_REWARD_FEE_BPS,
                ONE_OPERATOR,
                SLASHER_AMOUNTS,
            )
            .await
            .unwrap();

        let depositor = Keypair::new();
        vault_program_client
            .configure_depositor(&vault_root, &depositor.pubkey(), MINT_AMOUNT)
            .await
            .unwrap();
        vault_program_client
            .do_mint_to(&vault_root, &depositor, MINT_AMOUNT, MINT_AMOUNT)
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
            .do_full_vault_update(&vault_root.vault_pubkey, &operator_root_pubkeys)
            .await
            .unwrap();

        let vault = vault_program_client
            .get_vault(&vault_root.vault_pubkey)
            .await
            .unwrap();

        // configure slasher and slash
        let slasher = &slashers_amounts[0].0;
        fixture
            .create_ata(&vault.supported_mint, &slasher.pubkey())
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
                slasher,
                &operator_root.operator_pubkey,
                MAX_SLASH_AMOUNT,
            )
            .await
            .unwrap();

        let vault = vault_program_client
            .get_vault(&vault_root.vault_pubkey)
            .await
            .unwrap();
        assert_eq!(
            vault.tokens_deposited() - Vault::INITIALIZATION_TOKEN_AMOUNT,
            MINT_AMOUNT - MAX_SLASH_AMOUNT
        );
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
    async fn test_slash_vault_is_paused_fails() {
        let mut fixture = TestBuilder::new().await;

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
                ZERO_DEPOSIT_FEE_BPS,
                ZERO_WITHDRAW_FEE_BPS,
                ZERO_REWARD_FEE_BPS,
                ONE_OPERATOR,
                SLASHER_AMOUNTS,
            )
            .await
            .unwrap();

        let depositor = Keypair::new();
        vault_program_client
            .configure_depositor(&vault_root, &depositor.pubkey(), MINT_AMOUNT)
            .await
            .unwrap();
        vault_program_client
            .do_mint_to(&vault_root, &depositor, MINT_AMOUNT, MINT_AMOUNT)
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
            .do_full_vault_update(&vault_root.vault_pubkey, &operator_root_pubkeys)
            .await
            .unwrap();

        let vault = vault_program_client
            .get_vault(&vault_root.vault_pubkey)
            .await
            .unwrap();

        // configure slasher and slash
        let slasher = &slashers_amounts[0].0;
        fixture
            .create_ata(&vault.supported_mint, &slasher.pubkey())
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
            .set_is_paused(&vault_root.vault_pubkey, &vault_root.vault_admin, true)
            .await
            .unwrap();

        let test_error = vault_program_client
            .do_slash(
                &vault_root,
                &ncn_root.ncn_pubkey,
                &slasher,
                &operator_root.operator_pubkey,
                MAX_SLASH_AMOUNT,
            )
            .await;

        assert_vault_error(test_error, VaultError::VaultIsPaused);
    }
}
