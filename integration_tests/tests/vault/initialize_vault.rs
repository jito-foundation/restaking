#[cfg(test)]
mod tests {
    use solana_sdk::signature::Signer;

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
        ) = vault_program_client.setup_vault(99, 100).await.unwrap();

        let vault = vault_program_client.get_vault(&vault_pubkey).await.unwrap();
        assert_eq!(vault.admin, vault_admin.pubkey());
        assert_eq!(vault.delegation_admin, vault_admin.pubkey());
        assert_eq!(vault.operator_admin, vault_admin.pubkey());
        assert_eq!(vault.ncn_admin, vault_admin.pubkey());
        assert_eq!(vault.slasher_admin, vault_admin.pubkey());
        assert_eq!(vault.fee_wallet, vault_admin.pubkey());
        assert_eq!(vault.mint_burn_authority, None);
        assert_eq!(vault.capacity, u64::MAX);
        assert_eq!(vault.vault_index, 0);
        assert_eq!(vault.lrt_supply, 0);
        assert_eq!(vault.tokens_deposited, 0);
        assert_eq!(vault.deposit_fee_bps(), 99);
        assert_eq!(vault.withdrawal_fee_bps(), 100);
        assert_eq!(vault.ncn_count(), 0);
        assert_eq!(vault.operator_count(), 0);
        assert_eq!(vault.slasher_count(), 0);
    }
}
