#[cfg(test)]
mod tests {
    use crate::fixtures::fixture::TestBuilder;
    use jito_vault_core::config::Config;
    use jito_vault_sdk::sdk::set_config_program_fee_wallet;
    use solana_program::pubkey::Pubkey;
    use solana_sdk::signer::Signer;
    use solana_sdk::transaction::Transaction;
    use std::str::FromStr;

    #[tokio::test]
    async fn test_force_overwrite_config_ok() {
        let fixture = TestBuilder::new().await;
        let mut vault_program_client = fixture.vault_program_client();

        let config_admin = vault_program_client.do_initialize_config().await.unwrap();

        let mut config_before = vault_program_client
            .get_config(&Config::find_program_address(&jito_vault_program::id()).0)
            .await
            .unwrap();

        let blockhash = vault_program_client
            .banks_client
            .get_latest_blockhash()
            .await
            .unwrap();
        vault_program_client
            ._process_transaction(&Transaction::new_signed_with_payer(
                &[set_config_program_fee_wallet(
                    &jito_vault_program::id(),
                    &Config::find_program_address(&jito_vault_program::id()).0,
                )],
                Some(&config_admin.pubkey()),
                &[&config_admin],
                blockhash,
            ))
            .await
            .unwrap();

        let config_after = vault_program_client
            .get_config(&Config::find_program_address(&jito_vault_program::id()).0)
            .await
            .unwrap();

        config_before.program_fee_wallet =
            Pubkey::from_str("5eosrve6LktMZgVNszYzebgmmC7BjLK8NoWyRQtcmGTF").unwrap();

        assert_eq!(config_before, config_after);
    }
}
