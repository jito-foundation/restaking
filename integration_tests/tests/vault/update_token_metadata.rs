#[cfg(test)]
mod tests {
    use jito_vault_sdk::{
        error::VaultError,
        inline_mpl_token_metadata::{self, pda::find_metadata_account},
    };
    use solana_program::pubkey::Pubkey;
    use solana_sdk::signature::Keypair;

    use crate::fixtures::{
        fixture::TestBuilder,
        vault_client::{assert_vault_error, VaultProgramClient, VaultRoot},
    };

    async fn setup() -> (TestBuilder, VaultProgramClient, Pubkey, Keypair) {
        let mut fixture = TestBuilder::new().await;

        let mut vault_program_client = fixture.vault_program_client();

        let deposit_fee_bps = 99;
        let withdrawal_fee_bps = 100;
        let reward_fee_bps = 101;

        let (
            _config_admin,
            VaultRoot {
                vault_pubkey,
                vault_admin,
            },
        ) = vault_program_client
            .setup_config_and_vault(deposit_fee_bps, withdrawal_fee_bps, reward_fee_bps)
            .await
            .unwrap();

        let vault = vault_program_client.get_vault(&vault_pubkey).await.unwrap();

        // Create token metadata
        let name = "restaking JTO";
        let symbol = "rJTO";
        let uri = "https://www.jito.network/restaking/";

        let metadata_pubkey =
            inline_mpl_token_metadata::pda::find_metadata_account(&vault.vrt_mint).0;

        // Getting errors: RpcError(DeadlineExceeded)
        // https://solana.stackexchange.com/questions/3114/bpf-test-crashes-if-duration-10s
        fixture.warp_slot_incremental(10000).await.unwrap();

        vault_program_client
            .create_token_metadata(
                &vault_pubkey,
                &vault_admin,
                &vault.vrt_mint,
                &vault_admin,
                &metadata_pubkey,
                name.to_string(),
                symbol.to_string(),
                uri.to_string(),
            )
            .await
            .unwrap();

        (fixture, vault_program_client, vault_pubkey, vault_admin)
    }

    #[tokio::test]
    async fn success_update_token_metadata() {
        let (mut fixture, mut vault_program_client, vault_pubkey, vault_admin) = setup().await;

        let updated_name = "updated_name";
        let updated_symbol = "USYM";
        let updated_uri = "updated_uri";

        let vault = vault_program_client.get_vault(&vault_pubkey).await.unwrap();

        let metadata_pubkey = find_metadata_account(&vault.vrt_mint).0;

        fixture.warp_slot_incremental(10000).await.unwrap();

        vault_program_client
            .update_token_metadata(
                &vault_pubkey,
                &vault_admin,
                &vault.vrt_mint,
                &metadata_pubkey,
                updated_name.to_string(),
                updated_symbol.to_string(),
                updated_uri.to_string(),
            )
            .await
            .unwrap();

        let token_metadata = vault_program_client
            .get_token_metadata(&vault.vrt_mint)
            .await
            .unwrap();

        assert!(token_metadata.name.starts_with(updated_name));
        assert!(token_metadata.symbol.starts_with(updated_symbol));
        assert!(token_metadata.uri.starts_with(updated_uri));
    }

    #[tokio::test]
    async fn test_wrong_admin_signed() {
        let (mut fixture, mut vault_program_client, vault_pubkey, _) = setup().await;

        let updated_name = "updated_name";
        let updated_symbol = "USYM";
        let updated_uri = "updated_uri";

        let vault = vault_program_client.get_vault(&vault_pubkey).await.unwrap();

        let metadata_pubkey = find_metadata_account(&vault.vrt_mint).0;

        let bad_admin = Keypair::new();

        fixture.warp_slot_incremental(10000).await.unwrap();

        let response = vault_program_client
            .update_token_metadata(
                &vault_pubkey,
                &bad_admin,
                &vault.vrt_mint,
                &metadata_pubkey,
                updated_name.to_string(),
                updated_symbol.to_string(),
                updated_uri.to_string(),
            )
            .await;

        assert_vault_error(response, VaultError::VaultAdminInvalid);
    }
}
