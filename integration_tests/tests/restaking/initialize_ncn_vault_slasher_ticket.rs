#[cfg(test)]
mod tests {
    use jito_jsm_core::slot_toggle::SlotToggleState;
    use jito_restaking_core::{
        config::Config, ncn_vault_slasher_ticket::NcnVaultSlasherTicket,
        ncn_vault_ticket::NcnVaultTicket,
    };
    use jito_restaking_sdk::error::RestakingError;
    use solana_program::{instruction::InstructionError, pubkey::Pubkey};
    use solana_sdk::{
        signature::{Keypair, Signer},
        transaction::TransactionError,
    };

    use crate::fixtures::fixture::TestBuilder;

    #[tokio::test]
    async fn test_initialize_ncn_vault_slasher_ticket_ok() {
        let mut fixture = TestBuilder::new().await;
        let mut restaking_program_client = fixture.restaking_program_client();

        let _restaking_config_admin = restaking_program_client
            .do_initialize_config()
            .await
            .unwrap();
        let mut vault_program_client = fixture.vault_program_client();
        let (_vault_config_admin, vault_root) = vault_program_client
            .setup_config_and_vault(0, 0)
            .await
            .unwrap();

        let ncn_root = restaking_program_client.do_initialize_ncn().await.unwrap();
        restaking_program_client
            .do_initialize_ncn_vault_ticket(&ncn_root, &vault_root.vault_pubkey)
            .await
            .unwrap();

        let slasher = Keypair::new();
        restaking_program_client
            .do_initialize_ncn_vault_slasher_ticket(
                &ncn_root,
                &vault_root.vault_pubkey,
                &slasher.pubkey(),
                100,
            )
            .await
            .unwrap();

        // Verify NCN state
        let ncn = restaking_program_client
            .get_ncn(&ncn_root.ncn_pubkey)
            .await
            .unwrap();
        assert_eq!(ncn.slasher_count, 1);

        // Verify NCN vault slasher ticket
        let ticket = restaking_program_client
            .get_ncn_vault_slasher_ticket(
                &ncn_root.ncn_pubkey,
                &vault_root.vault_pubkey,
                &slasher.pubkey(),
            )
            .await
            .unwrap();
        assert_eq!(ticket.ncn, ncn_root.ncn_pubkey);
        assert_eq!(ticket.vault, vault_root.vault_pubkey);
        assert_eq!(ticket.slasher, slasher.pubkey());
        assert_eq!(ticket.max_slashable_per_epoch, 100);
        assert_eq!(ticket.index, 0);
        let slot = fixture.get_current_slot().await.unwrap();
        let config = restaking_program_client
            .get_config(&Config::find_program_address(&jito_restaking_program::id()).0)
            .await
            .unwrap();
        assert_eq!(
            ticket.state.state(slot, config.epoch_length),
            SlotToggleState::Inactive
        );
    }

    #[tokio::test]
    async fn test_initialize_ncn_vault_slasher_ticket_bad_pda_fails() {
        let fixture = TestBuilder::new().await;
        let mut restaking_program_client = fixture.restaking_program_client();

        let _restaking_config_admin = restaking_program_client
            .do_initialize_config()
            .await
            .unwrap();
        let ncn_root = restaking_program_client.do_initialize_ncn().await.unwrap();

        let mut vault_program_client = fixture.vault_program_client();
        let (_vault_config_admin, vault_root) = vault_program_client
            .setup_config_and_vault(0, 0)
            .await
            .unwrap();

        restaking_program_client
            .do_initialize_ncn_vault_ticket(&ncn_root, &vault_root.vault_pubkey)
            .await
            .unwrap();
        let slasher = Keypair::new();
        let transaction_error = restaking_program_client
            .initialize_ncn_vault_slasher_ticket(
                &Config::find_program_address(&jito_restaking_program::id()).0,
                &ncn_root.ncn_pubkey,
                &vault_root.vault_pubkey,
                &slasher.pubkey(),
                &NcnVaultTicket::find_program_address(
                    &jito_restaking_program::id(),
                    &ncn_root.ncn_pubkey,
                    &vault_root.vault_pubkey,
                )
                .0,
                &Pubkey::new_unique(),
                &ncn_root.ncn_admin,
                &ncn_root.ncn_admin,
                100,
            )
            .await
            .unwrap_err()
            .to_transaction_error()
            .unwrap();
        assert_eq!(
            transaction_error,
            TransactionError::InstructionError(0, InstructionError::InvalidAccountData)
        );
    }

    #[tokio::test]
    async fn test_initialize_ncn_vault_slasher_ticket_bad_admin_fails() {
        let mut fixture = TestBuilder::new().await;
        let mut restaking_program_client = fixture.restaking_program_client();

        let _restaking_config_admin = restaking_program_client
            .do_initialize_config()
            .await
            .unwrap();
        let ncn_root = restaking_program_client.do_initialize_ncn().await.unwrap();

        let mut vault_program_client = fixture.vault_program_client();
        let (_vault_config_admin, vault_root) = vault_program_client
            .setup_config_and_vault(0, 0)
            .await
            .unwrap();

        restaking_program_client
            .do_initialize_ncn_vault_ticket(&ncn_root, &vault_root.vault_pubkey)
            .await
            .unwrap();

        // wait 2 epochs to activate the vault
        let config = restaking_program_client
            .get_config(&Config::find_program_address(&jito_restaking_program::id()).0)
            .await
            .unwrap();
        fixture
            .warp_slot_incremental(2 * config.epoch_length)
            .await
            .unwrap();

        let slasher = Keypair::new();
        fixture.transfer(&slasher.pubkey(), 10.0).await.unwrap();
        let transaction_error = restaking_program_client
            .initialize_ncn_vault_slasher_ticket(
                &Config::find_program_address(&jito_restaking_program::id()).0,
                &ncn_root.ncn_pubkey,
                &vault_root.vault_pubkey,
                &slasher.pubkey(),
                &NcnVaultTicket::find_program_address(
                    &jito_restaking_program::id(),
                    &ncn_root.ncn_pubkey,
                    &vault_root.vault_pubkey,
                )
                .0,
                &NcnVaultSlasherTicket::find_program_address(
                    &jito_restaking_program::id(),
                    &ncn_root.ncn_pubkey,
                    &vault_root.vault_pubkey,
                    &slasher.pubkey(),
                )
                .0,
                &Keypair::new(),
                &ncn_root.ncn_admin,
                100,
            )
            .await
            .unwrap_err()
            .to_transaction_error()
            .unwrap();
        assert_eq!(
            transaction_error,
            TransactionError::InstructionError(
                0,
                InstructionError::Custom(RestakingError::NcnSlasherAdminInvalid as u32)
            )
        );
    }
}
