#[cfg(test)]
mod tests {
    use jito_restaking_core::{
        config::Config, ncn::Ncn, ncn_vault_slasher_ticket::NcnVaultSlasherTicket,
        ncn_vault_ticket::NcnVaultTicket,
    };
    use solana_sdk::signature::{Keypair, Signer};

    use crate::fixtures::fixture::TestBuilder;

    #[tokio::test]
    async fn test_ncn_add_vault_slasher_ok() {
        let mut fixture = TestBuilder::new().await;
        let mut restaking_program_client = fixture.restaking_program_client();

        let mut vault_program_client = fixture.vault_program_client();

        let (_vault_config_admin, vault_root) = vault_program_client
            .setup_config_and_vault(0, 0)
            .await
            .unwrap();

        // Initialize config
        let config_admin = Keypair::new();
        let config = Config::find_program_address(&jito_restaking_program::id()).0;
        fixture
            .transfer(&config_admin.pubkey(), 10.0)
            .await
            .unwrap();
        restaking_program_client
            .initialize_config(&config, &config_admin)
            .await
            .unwrap();

        // Initialize NCN
        let ncn_admin = Keypair::new();
        let ncn_base = Keypair::new();
        fixture.transfer(&ncn_admin.pubkey(), 10.0).await.unwrap();
        let ncn_pubkey =
            Ncn::find_program_address(&jito_restaking_program::id(), &ncn_base.pubkey()).0;
        restaking_program_client
            .initialize_ncn(&config, &ncn_pubkey, &ncn_admin, &ncn_base)
            .await
            .unwrap();

        // NCN adds vault
        let ncn_vault_ticket = NcnVaultTicket::find_program_address(
            &jito_restaking_program::id(),
            &ncn_pubkey,
            &vault_root.vault_pubkey,
        )
        .0;
        restaking_program_client
            .ncn_add_vault(
                &config,
                &ncn_pubkey,
                &vault_root.vault_pubkey,
                &ncn_vault_ticket,
                &ncn_admin,
                &ncn_admin,
            )
            .await
            .unwrap();

        let config_account = restaking_program_client.get_config(&config).await.unwrap();
        fixture
            .warp_slot_incremental(2 * config_account.epoch_length)
            .await
            .unwrap();

        // NCN adds vault slasher
        let slasher = Keypair::new();
        let ncn_vault_slasher_ticket = NcnVaultSlasherTicket::find_program_address(
            &jito_restaking_program::id(),
            &ncn_pubkey,
            &vault_root.vault_pubkey,
            &slasher.pubkey(),
        )
        .0;
        let max_slashable_per_epoch = 1000;
        restaking_program_client
            .ncn_add_vault_slasher(
                &config,
                &ncn_pubkey,
                &vault_root.vault_pubkey,
                &slasher.pubkey(),
                &ncn_vault_ticket,
                &ncn_vault_slasher_ticket,
                &ncn_admin,
                &ncn_admin,
                max_slashable_per_epoch,
            )
            .await
            .unwrap();

        // Verify NCN state
        let ncn = restaking_program_client.get_ncn(&ncn_pubkey).await.unwrap();
        assert_eq!(ncn.slasher_count, 1);

        // Verify NCN vault slasher ticket
        let ticket = restaking_program_client
            .get_ncn_vault_slasher_ticket(&ncn_pubkey, &vault_root.vault_pubkey, &slasher.pubkey())
            .await
            .unwrap();
        assert_eq!(ticket.ncn, ncn_pubkey);
        assert_eq!(ticket.vault, vault_root.vault_pubkey);
        assert_eq!(ticket.slasher, slasher.pubkey());
        assert_eq!(ticket.max_slashable_per_epoch, max_slashable_per_epoch);
        assert_eq!(ticket.index, 0);
        assert_eq!(
            ticket.state.slot_added(),
            fixture.get_current_slot().await.unwrap()
        );
    }
}
