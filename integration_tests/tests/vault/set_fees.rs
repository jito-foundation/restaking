#[cfg(test)]
mod tests {
    use jito_vault_core::{config::Config, vault::Vault};
    use solana_sdk::signature::{Keypair, Signer};

    use crate::fixtures::{fixture::TestBuilder, vault_client::VaultRoot};

    #[tokio::test]
    async fn test_initialize_vault_with_bad_fees() {
        let fixture = TestBuilder::new().await;

        let mut vault_program_client = fixture.vault_program_client();

        let _config_admin = vault_program_client.setup_config().await.unwrap();
        let vault_base = Keypair::new();

        let vault_pubkey =
            Vault::find_program_address(&jito_vault_program::id(), &vault_base.pubkey()).0;

        let lrt_mint = Keypair::new();
        let vault_admin = Keypair::new();
        let token_mint = Keypair::new();

        vault_program_client
            ._airdrop(&vault_admin.pubkey(), 100.0)
            .await
            .unwrap();
        vault_program_client
            ._create_token_mint(&token_mint)
            .await
            .unwrap();

        let config_address = Config::find_program_address(&jito_vault_program::id()).0;

        let config = vault_program_client
            .get_config(&config_address)
            .await
            .unwrap();

        let deposit_fee_bps = config.fee_cap_bps + 1;
        let withdraw_fee_bps = config.fee_cap_bps + 1;

        let result = vault_program_client
            .initialize_vault(
                &Config::find_program_address(&jito_vault_program::id()).0,
                &vault_pubkey,
                &lrt_mint,
                &token_mint,
                &vault_admin,
                &vault_base,
                deposit_fee_bps,
                withdraw_fee_bps,
            )
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_set_fees_ok() {
        let mut fixture = TestBuilder::new().await;

        let mut vault_program_client = fixture.vault_program_client();

        let deposit_fee_bps = 99;
        let withdrawal_fee_bps = 100;

        let (
            _config_admin,
            VaultRoot {
                vault_pubkey,
                vault_admin,
            },
        ) = vault_program_client
            .setup_config_and_vault(deposit_fee_bps, withdrawal_fee_bps)
            .await
            .unwrap();

        let vault = vault_program_client.get_vault(&vault_pubkey).await.unwrap();
        let config_pubkey = Config::find_program_address(&jito_vault_program::id()).0;
        let config = vault_program_client
            .get_config(&config_pubkey)
            .await
            .unwrap();

        assert_eq!(vault.deposit_fee_bps(), deposit_fee_bps);
        assert_eq!(vault.withdrawal_fee_bps(), withdrawal_fee_bps);

        // Test Setting the Fees
        fixture
            .warp_slot_incremental(config.epoch_length * 2)
            .await
            .unwrap();

        let deposit_fee_bps = 199;
        let withdrawal_fee_bps = 200;

        vault_program_client
            .set_fees(
                &config_pubkey,
                &vault_pubkey,
                &vault_admin,
                deposit_fee_bps,
                withdrawal_fee_bps,
            )
            .await
            .unwrap();

        let vault = vault_program_client.get_vault(&vault_pubkey).await.unwrap();
        assert_eq!(vault.deposit_fee_bps(), deposit_fee_bps);
        assert_eq!(vault.withdrawal_fee_bps(), withdrawal_fee_bps);

        // Should not be able to set fees again
        let result = vault_program_client
            .set_fees(
                &config_pubkey,
                &vault_pubkey,
                &vault_admin,
                deposit_fee_bps,
                withdrawal_fee_bps,
            )
            .await;

        assert!(result.is_err());

        fixture
            .warp_slot_incremental(config.epoch_length * 2)
            .await
            .unwrap();

        let deposit_fee_bps = 299;
        let withdrawal_fee_bps = 300;

        vault_program_client
            .set_fees(
                &config_pubkey,
                &vault_pubkey,
                &vault_admin,
                deposit_fee_bps,
                withdrawal_fee_bps,
            )
            .await
            .unwrap();

        let vault = vault_program_client.get_vault(&vault_pubkey).await.unwrap();
        assert_eq!(vault.deposit_fee_bps(), deposit_fee_bps);
        assert_eq!(vault.withdrawal_fee_bps(), withdrawal_fee_bps);
    }
}
