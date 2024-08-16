#[cfg(test)]
mod tests {
    use jito_vault_core::{
        config::Config, vault_ncn_slasher_operator_ticket::VaultNcnSlasherOperatorTicket,
        vault_ncn_slasher_ticket::VaultNcnSlasherTicket,
    };
    use solana_sdk::signature::{Keypair, Signer};
    use spl_associated_token_account::get_associated_token_address;

    use crate::fixtures::fixture::{ConfiguredVault, TestBuilder};

    #[tokio::test]
    async fn test_slash_ok() {
        let mut fixture = TestBuilder::new().await;

        const MAX_SLASH_AMOUNT: u64 = 100;
        const MINT_AMOUNT: u64 = 100_000;
        const DELEGATION_AMOUNT: u64 = 10_000;

        let ConfiguredVault {
            mut vault_program_client,
            vault_config_admin,
            vault_root,
            ncn_root,
            operator_roots,
            slashers_amounts,
            ..
        } = fixture
            .setup_vault_with_ncn_and_operators(0, 0, 1, &[MAX_SLASH_AMOUNT])
            .await
            .unwrap();

        let depositor = Keypair::new();
        fixture.transfer(&depositor.pubkey(), 1.0).await.unwrap();
        let vault = vault_program_client
            .get_vault(&vault_root.vault_pubkey)
            .await
            .unwrap();
        fixture
            .mint_to(&vault.supported_mint, &depositor.pubkey(), MINT_AMOUNT)
            .await
            .unwrap();

        // depositor ATA for VRT
        fixture
            .create_ata(&vault.vrt_mint, &depositor.pubkey())
            .await
            .unwrap();

        vault_program_client
            .mint_to(
                &vault_root.vault_pubkey,
                &vault.vrt_mint,
                &depositor,
                &get_associated_token_address(&depositor.pubkey(), &vault.supported_mint),
                &get_associated_token_address(&vault_root.vault_pubkey, &vault.supported_mint),
                &get_associated_token_address(&depositor.pubkey(), &vault.vrt_mint),
                &get_associated_token_address(&vault.fee_wallet, &vault.vrt_mint),
                None,
                MINT_AMOUNT,
            )
            .await
            .unwrap();

        let operator_root = &operator_roots[0];
        vault_program_client
            .do_add_delegation(
                &vault_root,
                &operator_root.operator_pubkey,
                DELEGATION_AMOUNT,
            )
            .await
            .unwrap();

        let config = vault_program_client
            .get_config(&Config::find_program_address(&jito_vault_program::id()).0)
            .await
            .unwrap();
        fixture
            .warp_slot_incremental(2 * config.epoch_length)
            .await
            .unwrap();
        let operator_root_pubkeys: Vec<_> =
            operator_roots.iter().map(|r| r.operator_pubkey).collect();
        vault_program_client
            .do_full_vault_update(&vault_root.vault_pubkey, &operator_root_pubkeys)
            .await
            .unwrap();

        // configure slasher and slash
        let slasher = &slashers_amounts[0].0;
        fixture
            .create_ata(&vault.supported_mint, &slasher.pubkey())
            .await
            .unwrap();
        let epoch = fixture.get_current_slot().await.unwrap() / config.epoch_length;
        vault_program_client
            .initialize_vault_ncn_slasher_operator_ticket(
                &Config::find_program_address(&jito_vault_program::id()).0,
                &vault_root.vault_pubkey,
                &ncn_root.ncn_pubkey,
                &slasher.pubkey(),
                &operator_root.operator_pubkey,
                &VaultNcnSlasherTicket::find_program_address(
                    &jito_vault_program::id(),
                    &vault_root.vault_pubkey,
                    &ncn_root.ncn_pubkey,
                    &slasher.pubkey(),
                )
                .0,
                &VaultNcnSlasherOperatorTicket::find_program_address(
                    &jito_vault_program::id(),
                    &vault_root.vault_pubkey,
                    &ncn_root.ncn_pubkey,
                    &slasher.pubkey(),
                    &operator_root.operator_pubkey,
                    epoch,
                )
                .0,
                &vault_config_admin,
            )
            .await
            .unwrap();

        vault_program_client
            .do_slash(
                &vault_root,
                &ncn_root.ncn_pubkey,
                &slasher,
                &operator_root.operator_pubkey,
                MAX_SLASH_AMOUNT,
            )
            .await
            .unwrap();

        let vault = vault_program_client
            .get_vault(&vault_root.vault_pubkey)
            .await
            .unwrap();
        assert_eq!(vault.tokens_deposited, MINT_AMOUNT - MAX_SLASH_AMOUNT);
        assert_eq!(vault.amount_delegated, DELEGATION_AMOUNT - MAX_SLASH_AMOUNT);

        let vault_operator_delegation = vault_program_client
            .get_vault_operator_delegation(&vault_root.vault_pubkey, &operator_root.operator_pubkey)
            .await
            .unwrap();

        assert_eq!(
            vault_operator_delegation.total_security().unwrap(),
            DELEGATION_AMOUNT - MAX_SLASH_AMOUNT
        );

        let epoch = fixture.get_current_slot().await.unwrap() / config.epoch_length;
        let vault_ncn_slasher_operator_ticket = vault_program_client
            .get_vault_ncn_slasher_operator_ticket(
                &vault_root.vault_pubkey,
                &ncn_root.ncn_pubkey,
                &slasher.pubkey(),
                &operator_root.operator_pubkey,
                epoch,
            )
            .await
            .unwrap();
        assert_eq!(vault_ncn_slasher_operator_ticket.slashed, 100);
        assert_eq!(vault_ncn_slasher_operator_ticket.epoch, epoch);
        assert_eq!(
            vault_ncn_slasher_operator_ticket.vault,
            vault_root.vault_pubkey
        );
        assert_eq!(vault_ncn_slasher_operator_ticket.ncn, ncn_root.ncn_pubkey);
        assert_eq!(vault_ncn_slasher_operator_ticket.slasher, slasher.pubkey());
        assert_eq!(
            vault_ncn_slasher_operator_ticket.operator,
            operator_root.operator_pubkey
        );
    }
}
