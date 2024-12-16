#[cfg(test)]
mod tests {
    use jito_vault_core::{config::Config, vault::Vault};
    use solana_sdk::signature::{Keypair, Signer};

    use crate::fixtures::fixture::{ConfiguredVault, TestBuilder};

    #[tokio::test]
    async fn test_reward_fee() {
        let mut fixture = TestBuilder::new().await;

        const MINT_AMOUNT: u64 = 1000;
        // Match's unit test in vault.rs: test_calculate_reward_fee
        const EXPECTED_FEE: u64 = 92;

        let deposit_fee_bps = 0;
        let withdrawal_fee_bps = 0;
        let reward_fee_bps = 1000; // 10%
        let num_operators = 1;
        let slasher_amounts = vec![];

        let ConfiguredVault {
            mut vault_program_client,
            vault_root,
            operator_roots,
            ..
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

        let rewarder = Keypair::new();
        vault_program_client
            .configure_depositor(&vault_root, &rewarder.pubkey(), MINT_AMOUNT * 2)
            .await
            .unwrap();

        // Mint some initial supply
        vault_program_client
            .do_mint_to(&vault_root, &rewarder, MINT_AMOUNT, MINT_AMOUNT)
            .await
            .unwrap();

        // Reward vault instead of staking
        vault_program_client
            .create_and_fund_reward_vault(&vault_root.vault_pubkey, &rewarder, MINT_AMOUNT)
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

        let reward_fee_account = vault_program_client
            .get_reward_fee_token_account(&vault_root.vault_pubkey)
            .await
            .unwrap();

        assert_eq!(
            MINT_AMOUNT * 2,
            vault.tokens_deposited() - Vault::INITIALIZATION_TOKEN_AMOUNT,
        );
        assert_eq!(EXPECTED_FEE, reward_fee_account.amount);
        assert_eq!(
            MINT_AMOUNT + EXPECTED_FEE,
            vault.vrt_supply() - Vault::INITIALIZATION_TOKEN_AMOUNT,
        );
    }

    #[tokio::test]
    async fn test_100_percent_reward_fee() {
        let mut fixture = TestBuilder::new().await;

        const MINT_AMOUNT: u64 = 100_000;

        let deposit_fee_bps = 0;
        let withdrawal_fee_bps = 0;
        let reward_fee_bps = 10_000; // 100%
        let num_operators = 1;
        let slasher_amounts = vec![];

        let ConfiguredVault {
            mut vault_program_client,
            vault_root,
            operator_roots,
            ..
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

        let rewarder = Keypair::new();
        vault_program_client
            .configure_depositor(&vault_root, &rewarder.pubkey(), MINT_AMOUNT * 2)
            .await
            .unwrap();

        // Mint some initial supply
        vault_program_client
            .do_mint_to(&vault_root, &rewarder, MINT_AMOUNT, MINT_AMOUNT)
            .await
            .unwrap();

        // Reward vault instead of staking
        vault_program_client
            .create_and_fund_reward_vault(&vault_root.vault_pubkey, &rewarder, MINT_AMOUNT)
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

        let reward_fee_account = vault_program_client
            .get_reward_fee_token_account(&vault_root.vault_pubkey)
            .await
            .unwrap();

        assert_eq!(
            MINT_AMOUNT * 2,
            vault.tokens_deposited() - Vault::INITIALIZATION_TOKEN_AMOUNT,
        );
        assert_eq!(MINT_AMOUNT, reward_fee_account.amount);
        assert_eq!(
            MINT_AMOUNT * 2,
            vault.vrt_supply() - Vault::INITIALIZATION_TOKEN_AMOUNT,
        );
    }

    #[tokio::test]
    async fn test_0_percent_reward_fee() {
        let mut fixture = TestBuilder::new().await;

        const MINT_AMOUNT: u64 = 100_000;

        let deposit_fee_bps = 0;
        let withdraw_fee_bps = 0;
        let reward_fee_bps = 0; // 0%
        let num_operators = 1;
        let slasher_amounts = vec![];

        let ConfiguredVault {
            mut vault_program_client,
            vault_root,
            operator_roots,
            ..
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

        let rewarder = Keypair::new();
        vault_program_client
            .configure_depositor(&vault_root, &rewarder.pubkey(), MINT_AMOUNT)
            .await
            .unwrap();

        // Reward vault instead of staking
        vault_program_client
            .create_and_fund_reward_vault(&vault_root.vault_pubkey, &rewarder, MINT_AMOUNT)
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

        let reward_fee_account = vault_program_client
            .get_reward_fee_token_account(&vault_root.vault_pubkey)
            .await
            .unwrap();

        assert_eq!(
            MINT_AMOUNT,
            vault.tokens_deposited() - Vault::INITIALIZATION_TOKEN_AMOUNT
        );
        assert_eq!(0, reward_fee_account.amount);
        assert_eq!(0, vault.vrt_supply() - Vault::INITIALIZATION_TOKEN_AMOUNT);
    }

    #[tokio::test]
    async fn test_reward_with_non_zero_balance() {
        let mut fixture = TestBuilder::new().await;

        const MINT_AMOUNT: u64 = 100_000;
        const EXPECTED_FEE: u64 = 5500;

        let deposit_fee_bps = 0;
        let withdrawal_fee_bps = 0;
        let reward_fee_bps = 1000; // 10%
        let num_operators = 1;
        let slasher_amounts = vec![];

        let ConfiguredVault {
            mut vault_program_client,
            vault_root,
            operator_roots,
            ..
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

        let depositor = Keypair::new();
        vault_program_client
            .configure_depositor(&vault_root, &depositor.pubkey(), MINT_AMOUNT * 2)
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

        // go to next epoch to force update
        fixture
            .warp_slot_incremental(config.epoch_length())
            .await
            .unwrap();

        let operator_root_pubkeys: Vec<_> =
            operator_roots.iter().map(|r| r.operator_pubkey).collect();
        vault_program_client
            .do_full_vault_update(&vault_root.vault_pubkey, &operator_root_pubkeys)
            .await
            .unwrap();

        vault_program_client
            .create_and_fund_reward_vault(&vault_root.vault_pubkey, &depositor, MINT_AMOUNT)
            .await
            .unwrap();

        // go to next epoch to force update
        fixture
            .warp_slot_incremental(config.epoch_length())
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

        let reward_fee_account = vault_program_client
            .get_reward_fee_token_account(&vault_root.vault_pubkey)
            .await
            .unwrap();

        assert_eq!(
            MINT_AMOUNT * 2,
            vault.tokens_deposited() - Vault::INITIALIZATION_TOKEN_AMOUNT,
        );
        assert_eq!(EXPECTED_FEE, reward_fee_account.amount);
        assert_eq!(
            MINT_AMOUNT + EXPECTED_FEE,
            vault.vrt_supply() - Vault::INITIALIZATION_TOKEN_AMOUNT,
        );
    }
}
