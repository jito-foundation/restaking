#[cfg(test)]
mod tests {
    use jito_vault_core::config::Config;
    use jito_vault_sdk::error::VaultError;
    use solana_program::pubkey::Pubkey;
    use solana_sdk::{
        instruction::InstructionError, signature::Signer, transaction::TransactionError,
    };

    use crate::fixtures::{fixture::TestBuilder, vault_client::VaultRoot};

    #[tokio::test]
    async fn test_initialize_vault_ok() {
        let fixture = TestBuilder::new().await;

        let mut vault_program_client = fixture.vault_program_client();

        let (
            _config_admin,
            VaultRoot {
                vault_pubkey,
                vault_admin,
            },
        ) = vault_program_client
            .setup_config_and_vault(99, 100, 0)
            .await
            .unwrap();

        let vault = vault_program_client.get_vault(&vault_pubkey).await.unwrap();
        assert_eq!(vault.admin, vault_admin.pubkey());
        assert_eq!(vault.delegation_admin, vault_admin.pubkey());
        assert_eq!(vault.operator_admin, vault_admin.pubkey());
        assert_eq!(vault.ncn_admin, vault_admin.pubkey());
        assert_eq!(vault.slasher_admin, vault_admin.pubkey());
        assert_eq!(vault.fee_wallet, vault_admin.pubkey());
        assert_eq!(vault.mint_burn_admin, Pubkey::default());
        assert_eq!(vault.capacity(), u64::MAX);
        assert_eq!(vault.vault_index(), 0);
        assert_eq!(vault.vrt_supply(), 0);
        assert_eq!(vault.tokens_deposited(), 0);
        assert_eq!(vault.deposit_fee_bps(), 99);
        assert_eq!(vault.withdrawal_fee_bps(), 100);
        assert_eq!(vault.ncn_count(), 0);
        assert_eq!(vault.operator_count(), 0);
        assert_eq!(vault.slasher_count(), 0);
    }

    #[tokio::test]
    async fn test_initialize_vault_deposit_fee_bps_too_high() {
        let fixture = TestBuilder::new().await;

        let mut vault_program_client = fixture.vault_program_client();

        vault_program_client.do_initialize_config().await.unwrap();

        let config = vault_program_client
            .get_config(&Config::find_program_address(&jito_vault_program::id()).0)
            .await
            .unwrap();

        let err = vault_program_client
            .do_initialize_vault(10001, 100, 100)
            .await
            .unwrap_err()
            .to_transaction_error()
            .unwrap();
        assert_eq!(
            err,
            TransactionError::InstructionError(
                0,
                InstructionError::Custom(VaultError::VaultFeeCapExceeded as u32)
            )
        );

        let err = vault_program_client
            .do_initialize_vault(config.deposit_withdrawal_fee_cap_bps() + 1, 0, 0)
            .await
            .unwrap_err()
            .to_transaction_error()
            .unwrap();
        assert_eq!(
            err,
            TransactionError::InstructionError(
                0,
                InstructionError::Custom(VaultError::VaultFeeCapExceeded as u32)
            )
        );
    }

    #[tokio::test]
    async fn test_initialize_vault_withdrawal_fee_bps_too_high() {
        let fixture = TestBuilder::new().await;

        let mut vault_program_client = fixture.vault_program_client();

        vault_program_client.do_initialize_config().await.unwrap();

        let config = vault_program_client
            .get_config(&Config::find_program_address(&jito_vault_program::id()).0)
            .await
            .unwrap();

        let err = vault_program_client
            .do_initialize_vault(100, 10001, 100)
            .await
            .unwrap_err()
            .to_transaction_error()
            .unwrap();
        assert_eq!(
            err,
            TransactionError::InstructionError(
                0,
                InstructionError::Custom(VaultError::VaultFeeCapExceeded as u32)
            )
        );

        let err = vault_program_client
            .do_initialize_vault(0, config.deposit_withdrawal_fee_cap_bps() + 1, 0)
            .await
            .unwrap_err()
            .to_transaction_error()
            .unwrap();
        assert_eq!(
            err,
            TransactionError::InstructionError(
                0,
                InstructionError::Custom(VaultError::VaultFeeCapExceeded as u32)
            )
        );
    }

    #[tokio::test]
    async fn test_initialize_vault_with_invalid_reward_fee_bps() {
        let fixture = TestBuilder::new().await;

        let mut vault_program_client = fixture.vault_program_client();

        vault_program_client.do_initialize_config().await.unwrap();

        let err = vault_program_client
            .do_initialize_vault(0, 0, 10001)
            .await
            .unwrap_err()
            .to_transaction_error()
            .unwrap();
        assert_eq!(
            err,
            TransactionError::InstructionError(
                0,
                InstructionError::Custom(VaultError::VaultFeeCapExceeded as u32)
            )
        );
    }
}
