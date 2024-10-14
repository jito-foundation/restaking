#[cfg(test)]
mod tests {
    use jito_vault_core::config::Config;
    use jito_vault_sdk::{error::VaultError, instruction::VaultAdminRole};
    use solana_sdk::{
        pubkey::Pubkey,
        signature::{Keypair, Signer},
    };

    use crate::fixtures::{
        fixture::TestBuilder,
        vault_client::{assert_vault_error, VaultRoot},
    };

    #[tokio::test]
    async fn test_set_secondary_admin_with_bad_admin() {
        let fixture = TestBuilder::new().await;

        let mut vault_program_client = fixture.vault_program_client();

        let deposit_fee_bps = 99;
        let withdrawal_fee_bps = 100;
        let reward_fee_bps = 101;
        let epoch_withdraw_cap_bps = 102;

        let (
            _config_admin,
            VaultRoot {
                vault_pubkey,
                vault_admin,
            },
        ) = vault_program_client
            .setup_config_and_vault(
                deposit_fee_bps,
                withdrawal_fee_bps,
                reward_fee_bps,
                epoch_withdraw_cap_bps,
            )
            .await
            .unwrap();

        let config_pubkey = Config::find_program_address(&jito_vault_program::id()).0;
        let vault = vault_program_client.get_vault(&vault_pubkey).await.unwrap();

        assert_eq!(vault.admin, vault_admin.pubkey());

        let bad_admin = Keypair::new();
        vault_program_client
            .airdrop(&bad_admin.pubkey(), 10.0)
            .await
            .unwrap();

        let new_admin = Pubkey::new_unique();
        let response = vault_program_client
            .set_secondary_admin(
                &config_pubkey,
                &vault_pubkey,
                &bad_admin,
                &new_admin,
                VaultAdminRole::CapacityAdmin,
            )
            .await;

        assert_vault_error(response, VaultError::VaultAdminInvalid);
    }

    #[tokio::test]
    async fn test_set_secondary_admin() {
        let fixture = TestBuilder::new().await;

        let mut vault_program_client = fixture.vault_program_client();

        let deposit_fee_bps = 99;
        let withdrawal_fee_bps = 100;
        let reward_fee_bps = 101;
        let epoch_withdraw_cap_bps = 102;

        let (
            _config_admin,
            VaultRoot {
                vault_pubkey,
                vault_admin,
            },
        ) = vault_program_client
            .setup_config_and_vault(
                deposit_fee_bps,
                withdrawal_fee_bps,
                reward_fee_bps,
                epoch_withdraw_cap_bps,
            )
            .await
            .unwrap();

        let config_pubkey = Config::find_program_address(&jito_vault_program::id()).0;
        let vault = vault_program_client.get_vault(&vault_pubkey).await.unwrap();

        assert_eq!(vault.admin, vault_admin.pubkey());

        {
            // Capacity Admin
            let new_admin = Pubkey::new_unique();
            vault_program_client
                .set_secondary_admin(
                    &config_pubkey,
                    &vault_pubkey,
                    &vault_admin,
                    &new_admin,
                    VaultAdminRole::CapacityAdmin,
                )
                .await
                .unwrap();

            let vault = vault_program_client.get_vault(&vault_pubkey).await.unwrap();
            assert_eq!(vault.capacity_admin, new_admin);
        }

        {
            // Delegation Admin
            let new_admin = Pubkey::new_unique();
            vault_program_client
                .set_secondary_admin(
                    &config_pubkey,
                    &vault_pubkey,
                    &vault_admin,
                    &new_admin,
                    VaultAdminRole::DelegationAdmin,
                )
                .await
                .unwrap();

            let vault = vault_program_client.get_vault(&vault_pubkey).await.unwrap();
            assert_eq!(vault.delegation_admin, new_admin);
        }

        {
            // Fee Admin
            let new_admin = Pubkey::new_unique();
            vault_program_client
                .set_secondary_admin(
                    &config_pubkey,
                    &vault_pubkey,
                    &vault_admin,
                    &new_admin,
                    VaultAdminRole::FeeAdmin,
                )
                .await
                .unwrap();

            let vault = vault_program_client.get_vault(&vault_pubkey).await.unwrap();
            assert_eq!(vault.fee_admin, new_admin);
        }

        {
            // Fee Wallet
            let new_admin = Pubkey::new_unique();
            vault_program_client
                .set_secondary_admin(
                    &config_pubkey,
                    &vault_pubkey,
                    &vault_admin,
                    &new_admin,
                    VaultAdminRole::FeeWallet,
                )
                .await
                .unwrap();

            let vault = vault_program_client.get_vault(&vault_pubkey).await.unwrap();
            assert_eq!(vault.fee_wallet, new_admin);
        }

        {
            // Mint Burn
            let new_admin = Pubkey::new_unique();
            vault_program_client
                .set_secondary_admin(
                    &config_pubkey,
                    &vault_pubkey,
                    &vault_admin,
                    &new_admin,
                    VaultAdminRole::MintBurnAdmin,
                )
                .await
                .unwrap();

            let vault = vault_program_client.get_vault(&vault_pubkey).await.unwrap();
            assert_eq!(vault.mint_burn_admin, new_admin);
        }

        {
            // NCN Admin
            let new_admin = Pubkey::new_unique();
            vault_program_client
                .set_secondary_admin(
                    &config_pubkey,
                    &vault_pubkey,
                    &vault_admin,
                    &new_admin,
                    VaultAdminRole::NcnAdmin,
                )
                .await
                .unwrap();

            let vault = vault_program_client.get_vault(&vault_pubkey).await.unwrap();
            assert_eq!(vault.ncn_admin, new_admin);
        }

        {
            // Operator Admin
            let new_admin = Pubkey::new_unique();
            vault_program_client
                .set_secondary_admin(
                    &config_pubkey,
                    &vault_pubkey,
                    &vault_admin,
                    &new_admin,
                    VaultAdminRole::OperatorAdmin,
                )
                .await
                .unwrap();

            let vault = vault_program_client.get_vault(&vault_pubkey).await.unwrap();
            assert_eq!(vault.operator_admin, new_admin);
        }

        {
            // Slasher Admin
            let new_admin = Pubkey::new_unique();
            vault_program_client
                .set_secondary_admin(
                    &config_pubkey,
                    &vault_pubkey,
                    &vault_admin,
                    &new_admin,
                    VaultAdminRole::SlasherAdmin,
                )
                .await
                .unwrap();

            let vault = vault_program_client.get_vault(&vault_pubkey).await.unwrap();
            assert_eq!(vault.slasher_admin, new_admin);
        }

        {
            // WithdrawalAdmin
            let new_admin = Pubkey::new_unique();
            vault_program_client
                .set_secondary_admin(
                    &config_pubkey,
                    &vault_pubkey,
                    &vault_admin,
                    &new_admin,
                    VaultAdminRole::DelegateAssetAdmin,
                )
                .await
                .unwrap();

            let vault = vault_program_client.get_vault(&vault_pubkey).await.unwrap();
            assert_eq!(vault.delegate_asset_admin, new_admin);
        }
    }
}
