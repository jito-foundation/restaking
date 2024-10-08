#[cfg(test)]
mod tests {
    use jito_vault_sdk::error::VaultError;
    use solana_sdk::{signature::Keypair, signer::Signer};
    use spl_associated_token_account::get_associated_token_address;

    use crate::fixtures::{
        fixture::{ConfiguredVault, TestBuilder},
        vault_client::assert_vault_error,
    };

    /// Tests basic burn functionality
    #[tokio::test]
    async fn test_burn_basic_success() {
        const MINT_AMOUNT: u64 = 100_000;
        const BURN_AMOUNT: u64 = 50_000;

        let deposit_fee_bps = 0;
        let withdraw_fee_bps = 0;
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

        // Burn
        vault_program_client
            .do_burn(&vault_root, &depositor, BURN_AMOUNT, BURN_AMOUNT)
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

    /// Tests burn with fees
    #[tokio::test]
    async fn test_burn_with_fees() {
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

        assert_eq!(vault.tokens_deposited(), MINT_AMOUNT - expected_out);
        assert_eq!(vault.vrt_supply(), MINT_AMOUNT - BURN_AMOUNT);

        let depositor_token_account = fixture
            .get_token_account(&get_associated_token_address(
                &depositor.pubkey(),
                &vault.supported_mint,
            ))
            .await
            .unwrap();
        assert_eq!(depositor_token_account.amount, expected_out);
    }

    /// Tests burn with slippage protection
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

    /// Tests burn with program fee
    #[tokio::test]
    async fn test_burn_with_program_fee() {
        const MINT_AMOUNT: u64 = 100_000;
        const BURN_AMOUNT: u64 = 50_000;
        const PROGRAM_FEE_BPS: u16 = 50; // 0.5%

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

        // Set program fee
        vault_program_client
            .set_program_fee(&vault_root, PROGRAM_FEE_BPS)
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
        assert_eq!(vault.vrt_supply(), MINT_AMOUNT - BURN_AMOUNT);

        // Check depositor received correct amount
        let depositor_token_account = fixture
            .get_token_account(&get_associated_token_address(
                &depositor.pubkey(),
                &vault.supported_mint,
            ))
            .await
            .unwrap();
        assert_eq!(depositor_token_account.amount, expected_out);

        // Check program fee wallet received correct amount
        let config = vault_program_client
            .get_config(&vault_root.config_pubkey)
            .await
            .unwrap();
        let program_fee_wallet = fixture
            .get_token_account(&get_associated_token_address(
                &config.program_fee_wallet,
                &vault.supported_mint,
            ))
            .await
            .unwrap();
        assert_eq!(program_fee_wallet.amount, program_fee);
    }

    #[tokio::test]
    async fn test_burn_incorrect_program_fee_account() {
        const MINT_AMOUNT: u64 = 100_000;
        const BURN_AMOUNT: u64 = 50_000;

        let deposit_fee_bps = 0;
        let withdraw_fee_bps = 0;
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

        let vault = vault_program_client
            .get_vault(&vault_root.vault_pubkey)
            .await
            .unwrap();

        // Create a random pubkey to use as an incorrect program fee wallet
        let incorrect_program_fee_wallet = Pubkey::new_unique();
        fixture
            .create_ata(&vault.supported_mint, &incorrect_program_fee_wallet)
            .await
            .unwrap();

        // Attempt to burn with incorrect program fee account
        let result = vault_program_client
            .burn(
                &vault_root.config_pubkey,
                &vault_root.vault_pubkey,
                &get_associated_token_address(&vault_root.vault_pubkey, &vault.supported_mint),
                &vault.vrt_mint,
                &depositor.pubkey(),
                &get_associated_token_address(&depositor.pubkey(), &vault.supported_mint),
                &get_associated_token_address(&depositor.pubkey(), &vault.vrt_mint),
                &get_associated_token_address(&vault.fee_wallet, &vault.supported_mint),
                &get_associated_token_address(&incorrect_program_fee_wallet, &vault.supported_mint),
                BURN_AMOUNT,
                0, // min_amount_out
            )
            .await;

        assert_vault_error(result, VaultError::InvalidProgramFeeAccount);
    }
}
