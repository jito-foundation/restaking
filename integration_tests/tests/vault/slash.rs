#[cfg(test)]
mod tests {
    use jito_restaking_core::config::Config;
    use solana_sdk::signature::{Keypair, Signer};
    use spl_associated_token_account::get_associated_token_address;

    use crate::fixtures::fixture::TestBuilder;

    #[tokio::test]
    async fn test_slash_ok() {
        let mut fixture = TestBuilder::new().await;

        let mut restaking_program_client = fixture.restaking_program_client();
        let mut vault_program_client = fixture.vault_program_client();

        let (_config_admin, vault_root) = vault_program_client
            .setup_config_and_vault(100, 100)
            .await
            .unwrap();

        let _restaking_config_admin = restaking_program_client
            .do_initialize_config()
            .await
            .unwrap();

        let ncn_root = restaking_program_client.do_initialize_ncn().await.unwrap();
        let operator_root = restaking_program_client
            .do_initialize_operator()
            .await
            .unwrap();

        let restaking_config = restaking_program_client
            .get_config(&Config::find_program_address(&jito_restaking_program::id()).0)
            .await
            .unwrap();

        let slasher = Keypair::new();
        fixture.transfer(&slasher.pubkey(), 1.0).await.unwrap();

        restaking_program_client
            .do_initialize_ncn_vault_ticket(&ncn_root, &vault_root.vault_pubkey)
            .await
            .unwrap();
        restaking_program_client
            .operator_vault_opt_in(&operator_root, &vault_root.vault_pubkey)
            .await
            .unwrap();

        fixture
            .warp_slot_incremental(2 * restaking_config.epoch_length)
            .await
            .unwrap();

        vault_program_client
            .vault_ncn_opt_in(&vault_root, &ncn_root.ncn_pubkey)
            .await
            .unwrap();

        // NCN <-> Operator
        // operator needs to opt-in first
        restaking_program_client
            .do_initialize_operator_ncn_ticket(&operator_root, &ncn_root.ncn_pubkey)
            .await
            .unwrap();
        fixture
            .warp_slot_incremental(2 * restaking_config.epoch_length)
            .await
            .unwrap();
        restaking_program_client
            .ncn_operator_opt_in(&ncn_root, &operator_root.operator_pubkey)
            .await
            .unwrap();

        // Vault <-> Operator
        // operator needs to opt-in first
        vault_program_client
            .vault_operator_opt_in(&vault_root, &operator_root.operator_pubkey)
            .await
            .unwrap();

        fixture
            .warp_slot_incremental(2 * restaking_config.epoch_length)
            .await
            .unwrap();

        restaking_program_client
            .do_ncn_vault_slasher_opt_in(
                &ncn_root,
                &vault_root.vault_pubkey,
                &slasher.pubkey(),
                100,
            )
            .await
            .unwrap();

        fixture
            .warp_slot_incremental(2 * restaking_config.epoch_length)
            .await
            .unwrap();

        vault_program_client
            .vault_ncn_vault_slasher_opt_in(&vault_root, &ncn_root.ncn_pubkey, &slasher.pubkey())
            .await
            .unwrap();

        fixture
            .warp_slot_incremental(2 * restaking_config.epoch_length)
            .await
            .unwrap();

        let vault = vault_program_client
            .get_vault(&vault_root.vault_pubkey)
            .await
            .unwrap();

        let depositor = Keypair::new();
        fixture.transfer(&depositor.pubkey(), 1.0).await.unwrap();
        fixture
            .mint_to(&vault.supported_mint, &depositor.pubkey(), 100_000)
            .await
            .unwrap();

        let vault = vault_program_client
            .get_vault(&vault_root.vault_pubkey)
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
                100_000,
            )
            .await
            .unwrap();

        // user has 99_000 because 100 bips deposit fee

        vault_program_client
            .do_update_vault(&vault_root.vault_pubkey)
            .await
            .unwrap();
        vault_program_client
            .delegate(&vault_root, &operator_root.operator_pubkey, 10_000)
            .await
            .unwrap();

        {
            let vault_delegation_list = vault_program_client
                .get_vault_delegation_list(&vault_root.vault_pubkey)
                .await
                .unwrap();

            assert_eq!(
                vault_delegation_list.delegations[0].operator,
                operator_root.operator_pubkey
            );
            assert_eq!(vault_delegation_list.delegations[0].staked_amount, 10_000);
        }

        fixture
            .create_ata(&vault.supported_mint, &slasher.pubkey())
            .await
            .unwrap();

        vault_program_client
            .setup_vault_ncn_slasher_operator_ticket(
                &vault_root,
                &ncn_root.ncn_pubkey,
                &slasher.pubkey(),
                &operator_root.operator_pubkey,
            )
            .await
            .unwrap();

        vault_program_client
            .do_slash(
                &vault_root,
                &ncn_root.ncn_pubkey,
                &slasher,
                &operator_root.operator_pubkey,
                100,
            )
            .await
            .unwrap();

        let vault = vault_program_client
            .get_vault(&vault_root.vault_pubkey)
            .await
            .unwrap();
        assert_eq!(vault.tokens_deposited, 99_900);

        {
            let delegation_list = vault_program_client
                .get_vault_delegation_list(&vault_root.vault_pubkey)
                .await
                .unwrap();
            assert_eq!(
                delegation_list.delegations[0].operator,
                operator_root.operator_pubkey
            );
            assert_eq!(delegation_list.delegations[0].staked_amount, 9_900);
        }

        let epoch = fixture.get_current_slot().await.unwrap() / restaking_config.epoch_length;
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
