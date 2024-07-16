use jito_vault_core::{config::Config, vault::Vault, vault_delegation_list::VaultDelegationList};
use solana_sdk::signature::{Keypair, Signer};

use crate::fixtures::fixture::TestBuilder;

#[tokio::test]
async fn test_initialize_vault_ok() {
    let mut fixture = TestBuilder::new().await;
    let mut vault_program_client = fixture.vault_program_client();

    let backing_token_mint = Keypair::new();
    fixture
        .create_token_mint(&backing_token_mint)
        .await
        .unwrap();

    let config_pubkey = Config::find_program_address(&jito_vault_program::id()).0;
    let config_admin = Keypair::new();

    fixture.transfer(&config_admin.pubkey(), 1.0).await.unwrap();

    vault_program_client
        .initialize_config(&config_pubkey, &config_admin)
        .await
        .unwrap();

    let vault_base = Keypair::new();
    let vault_pubkey =
        Vault::find_program_address(&jito_vault_program::id(), &vault_base.pubkey()).0;
    let vault_delegation_list =
        VaultDelegationList::find_program_address(&jito_vault_program::id(), &vault_pubkey).0;
    let lrt_mint = Keypair::new();
    let vault_admin = Keypair::new();

    fixture.transfer(&vault_admin.pubkey(), 1.0).await.unwrap();

    vault_program_client
        .initialize_vault(
            &config_pubkey,
            &vault_pubkey,
            &vault_delegation_list,
            &lrt_mint,
            &backing_token_mint,
            &vault_admin,
            &vault_base,
            99,
            100,
        )
        .await
        .unwrap();

    let vault = vault_program_client.get_vault(&vault_pubkey).await.unwrap();
    assert_eq!(vault.base(), vault_base.pubkey());
    assert_eq!(vault.lrt_mint(), lrt_mint.pubkey());
    assert_eq!(vault.supported_mint(), backing_token_mint.pubkey());
    assert_eq!(vault.admin(), vault_admin.pubkey());
    assert_eq!(vault.delegation_admin(), vault_admin.pubkey());
    assert_eq!(vault.operator_admin(), vault_admin.pubkey());
    assert_eq!(vault.avs_admin(), vault_admin.pubkey());
    assert_eq!(vault.slasher_admin(), vault_admin.pubkey());
    assert_eq!(vault.fee_owner(), vault_admin.pubkey());
    assert_eq!(vault.mint_burn_authority(), None);
    assert_eq!(vault.capacity(), u64::MAX);
    assert_eq!(vault.vault_index(), 0);
    assert_eq!(vault.lrt_supply(), 0);
    assert_eq!(vault.tokens_deposited(), 0);
    assert_eq!(vault.deposit_fee_bps(), 99);
    assert_eq!(vault.withdrawal_fee_bps(), 100);
    assert_eq!(vault.avs_count(), 0);
    assert_eq!(vault.operator_count(), 0);
    assert_eq!(vault.slasher_count(), 0);
}
