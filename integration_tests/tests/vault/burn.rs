#[cfg(test)]
mod tests {
    use jito_vault_core::config::Config;
    use jito_vault_sdk::error::VaultError;
    use solana_sdk::{signature::Keypair, signer::Signer};
    use spl_associated_token_account::get_associated_token_address;

    use crate::fixtures::{
        fixture::{ConfiguredVault, TestBuilder},
        vault_client::assert_vault_error,
    };

    #[tokio::test]
    async fn test_burn_basic_success() {
        const MINT_AMOUNT: u64 = 100_000;
        const BURN_AMOUNT: u64 = 50_000;

        let deposit_fee_bps = 0;
        let withdraw_fee_bps = 0;
        let reward_fee_bps = 0;
        let epoch_withdraw_cap_bps = 5_000; // 50%
        let num_operators = 1;
        let slasher_amounts = vec![];

        let mut fixture = TestBuilder::new().await;
        let ConfiguredVault {
            mut vault_program_client,
            vault_root,
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
            .do_burn(&vault_root, &depositor, HALF_MINT_AMOUNT, HALF_MINT_AMOUNT)
            .await
            .unwrap();

        let vault = vault_program_client
            .get_vault(&vault_root.vault_pubkey)
            .await
            .unwrap();

        assert_eq!(vault.tokens_deposited(), MINT_AMOUNT - BURN_AMOUNT);
        assert_eq!(vault.vrt_supply(), MINT_AMOUNT - BURN_AMOUNT);

        let depositor_token_account = fixture
            .get_token_account(&get_associated_token_address(
                &depositor.pubkey(),
                &vault.supported_mint,
            ))
            .await
            .unwrap();
        assert_eq!(depositor_token_account.amount, BURN_AMOUNT);
    }

    #[tokio::test]
    async fn test_burn_with_fees() {
        const MINT_AMOUNT: u64 = 100_000;
        const BURN_AMOUNT: u64 = 50_000;

        let deposit_fee_bps = 0;
        let withdraw_fee_bps = 100; // 1%
        let reward_fee_bps = 0;
        let epoch_withdraw_cap_bps = 5_000; // 50%
        let num_operators = 1;
        let slasher_amounts = vec![];

        let mut fixture = TestBuilder::new().await;
        let ConfiguredVault {
            mut vault_program_client,
            vault_root,
            ..
        } = fixture
            .setup_vault_with_ncn_and_operators(
                deposit_fee_bps,
                withdraw_fee_bps,
                reward_fee_bps,
                epoch_withdraw_cap_bps,
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
        // Burn
        let expected_out = BURN_AMOUNT - (BURN_AMOUNT * withdraw_fee_bps as u64 / 10000);
        vault_program_client
            .do_burn(&vault_root, &depositor, BURN_AMOUNT, expected_out)
            .await
            .unwrap();

        let vault = vault_program_client
            .get_vault(&vault_root.vault_pubkey)
            .await
            .unwrap();

        // VRT and tokens deposited are 1:1 here
        assert_eq!(vault.tokens_deposited(), MINT_AMOUNT - expected_out);
        assert_eq!(vault.vrt_supply(), MINT_AMOUNT - expected_out);

        let depositor_token_account = fixture
            .get_token_account(&get_associated_token_address(
                &depositor.pubkey(),
                &vault.supported_mint,
            ))
            .await
            .unwrap();
        assert_eq!(depositor_token_account.amount, expected_out);
    }

    #[tokio::test]
    async fn test_burn_slippage_fails() {
        const MINT_AMOUNT: u64 = 100_000;
        const BURN_AMOUNT: u64 = 50_000;

        let deposit_fee_bps = 0;
        let withdraw_fee_bps = 100; // 1%
        let reward_fee_bps = 0;
        let num_operators = 1;
        let slasher_amounts = vec![];

        let mut fixture = TestBuilder::new().await;
        let ConfiguredVault {
            mut vault_program_client,
            vault_root,
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

        // Burn with too high min_amount_out
        let result = vault_program_client
            .do_burn(&vault_root, &depositor, BURN_AMOUNT, BURN_AMOUNT)
            .await;

        assert_vault_error(result, VaultError::SlippageError);
    }

    #[tokio::test]
    async fn test_burn_with_program_fee() {
        const MINT_AMOUNT: u64 = 100_000;
        const BURN_AMOUNT: u64 = 50_000;
        const PROGRAM_FEE_BPS: u16 = 10; // 0.1%

        let deposit_fee_bps = 0;
        let withdraw_fee_bps = 100; // 1%
        let reward_fee_bps = 0;
        let num_operators = 1;
        let slasher_amounts = vec![];

        let mut fixture = TestBuilder::new().await;
        let ConfiguredVault {
            mut vault_program_client,
            vault_root,
            vault_config_admin,
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

        // Burn
        let withdraw_fee = BURN_AMOUNT * withdraw_fee_bps as u64 / 10000;
        let program_fee = BURN_AMOUNT * PROGRAM_FEE_BPS as u64 / 10000;
        let expected_out = BURN_AMOUNT - withdraw_fee - program_fee;
        vault_program_client
            .do_burn(&vault_root, &depositor, BURN_AMOUNT, expected_out)
            .await
            .unwrap();

        let vault = vault_program_client
            .get_vault(&vault_root.vault_pubkey)
            .await
            .unwrap();

        // Check vault state
        assert_eq!(vault.tokens_deposited(), MINT_AMOUNT - expected_out);
        assert_eq!(vault.vrt_supply(), MINT_AMOUNT - expected_out);

        // Check depositor received correct amount
        let depositor_token_account = fixture
            .get_token_account(&get_associated_token_address(
                &depositor.pubkey(),
                &vault.supported_mint,
            ))
            .await
            .unwrap();
        assert_eq!(depositor_token_account.amount, expected_out);

        let config_pubkey = Config::find_program_address(&jito_vault_program::id()).0;

        // Check program fee wallet received correct amount
        let config = vault_program_client
            .get_config(&config_pubkey)
            .await
            .unwrap();
        let program_fee_wallet = fixture
            .get_token_account(&get_associated_token_address(
                &config.program_fee_wallet,
                &vault.vrt_mint,
            ))
            .await
            .unwrap();
        assert_eq!(program_fee_wallet.amount, program_fee);
    }
}
