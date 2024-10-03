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

    const DEPOSIT_FEE_BPS: u16 = 0;
    const WITHDRAW_FEE_BPS: u16 = 0;
    const REWARD_FEE_BPS: u16 = 0;
    const EPOCH_WITHDRAW_CAP_BPS: u16 = 5000; // 50%
    const NUM_OPERATORS: u16 = 1;
    const MINT_AMOUNT: u64 = 100_000;
    const HALF_MINT_AMOUNT: u64 = MINT_AMOUNT / 2;

    #[tokio::test]
    async fn test_burn_ok() {
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
                DEPOSIT_FEE_BPS,
                WITHDRAW_FEE_BPS,
                REWARD_FEE_BPS,
                EPOCH_WITHDRAW_CAP_BPS,
                NUM_OPERATORS,
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

        let depositor_vrt_token_account = fixture
            .get_token_account(&get_associated_token_address(
                &depositor.pubkey(),
                &vault.vrt_mint,
            ))
            .await
            .unwrap();
        assert_eq!(depositor_vrt_token_account.amount, HALF_MINT_AMOUNT);

        let depositor_supported_token_account = fixture
            .get_token_account(&get_associated_token_address(
                &depositor.pubkey(),
                &vault.supported_mint,
            ))
            .await
            .unwrap();
        assert_eq!(depositor_supported_token_account.amount, HALF_MINT_AMOUNT);
    }

    #[tokio::test]
    async fn test_burn_epoch_withdraw_cap_exceed_fails() {
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
                DEPOSIT_FEE_BPS,
                WITHDRAW_FEE_BPS,
                REWARD_FEE_BPS,
                EPOCH_WITHDRAW_CAP_BPS,
                NUM_OPERATORS,
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

        let test_error = vault_program_client
            .do_burn(&vault_root, &depositor, HALF_MINT_AMOUNT, HALF_MINT_AMOUNT)
            .await;

        assert_vault_error(test_error, VaultError::VaultWithdrawalLimitExceeded);
    }
}
