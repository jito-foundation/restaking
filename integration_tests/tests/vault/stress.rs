#[cfg(test)]
mod tests {
    use crate::fixtures::fixture::TestBuilder;
    use crate::fixtures::restaking_client::OperatorRoot;
    use jito_restaking_core::config::Config as RestakingConfig;
    use jito_restaking_core::ncn_operator_ticket::NcnOperatorTicket;
    use jito_restaking_core::operator::Operator;
    use jito_restaking_core::operator_ncn_ticket::OperatorNcnTicket;
    use jito_restaking_core::operator_vault_ticket::OperatorVaultTicket;
    use jito_restaking_sdk::sdk::{
        initialize_ncn_operator_ticket, initialize_operator, initialize_operator_ncn_ticket,
        initialize_operator_vault_ticket,
    };
    use jito_vault_core::config::Config as VaultConfig;
    use jito_vault_core::vault_delegation_list::VaultDelegationList;
    use jito_vault_core::vault_operator_ticket::VaultOperatorTicket;
    use jito_vault_core::vault_staker_withdrawal_ticket::VaultStakerWithdrawalTicket;
    use jito_vault_sdk::sdk::{add_delegation, initialize_vault_operator_ticket};

    use solana_sdk::signature::{Keypair, Signer};
    use solana_sdk::transaction::Transaction;
    use spl_associated_token_account::get_associated_token_address;

    #[tokio::test]
    async fn test_update_max_delegations_ok() {
        const NUM_OPERATORS: usize = 500;
        const MINT_AMOUNT: u64 = 100_000;

        let mut fixture = TestBuilder::new().await;
        let mut vault_client = fixture.vault_program_client();
        let mut restaking_client = fixture.restaking_program_client();

        let restaking_config_admin = restaking_client.do_initialize_config().await.unwrap();
        let (_vault_admin, vault_root) = vault_client
            .do_initialize_config_and_vault(0, 0)
            .await
            .unwrap();
        let ncn_root = restaking_client.do_initialize_ncn().await.unwrap();

        let restaking_config = restaking_client
            .get_config(&RestakingConfig::find_program_address(&jito_restaking_program::id()).0)
            .await
            .unwrap();

        // ncn <> vault
        restaking_client
            .do_initialize_ncn_vault_ticket(&ncn_root, &vault_root.vault_pubkey)
            .await
            .unwrap();
        fixture
            .warp_slot_incremental(2 * restaking_config.epoch_length)
            .await
            .unwrap();
        vault_client
            .do_initialize_vault_ncn_ticket(&vault_root, &ncn_root.ncn_pubkey)
            .await
            .unwrap();

        let operator_admin = Keypair::new();
        fixture
            .transfer(&operator_admin.pubkey(), 1000.0)
            .await
            .unwrap();

        let operator_roots = (0..NUM_OPERATORS)
            .map(|_| {
                let operator_base = Keypair::new();
                let operator_pubkey = Operator::find_program_address(
                    &jito_restaking_program::id(),
                    &operator_base.pubkey(),
                )
                .0;
                OperatorRoot {
                    operator_pubkey,
                    operator_admin: operator_admin.insecure_clone(),
                    operator_base,
                }
            })
            .collect::<Vec<_>>();

        let restaking_config_pubkey =
            RestakingConfig::find_program_address(&jito_restaking_program::id()).0;
        for operator_roots in operator_roots.chunks(32) {
            let ixs = operator_roots
                .iter()
                .map(|operator_root| {
                    initialize_operator(
                        &jito_restaking_program::id(),
                        &restaking_config_pubkey,
                        &operator_root.operator_pubkey,
                        &operator_root.operator_admin.pubkey(),
                        &operator_root.operator_base.pubkey(),
                    )
                })
                .collect::<Vec<_>>();
            let blockhash = restaking_client.latest_blockhash().await.unwrap();
            let mut signing_keys = vec![&restaking_client.payer, &operator_admin];
            signing_keys.extend(
                operator_roots
                    .iter()
                    .map(|operator_root| &operator_root.operator_base),
            );
            restaking_client
                .process_transaction(&Transaction::new_signed_with_payer(
                    &ixs,
                    Some(&restaking_client.payer.pubkey()),
                    &signing_keys,
                    blockhash,
                ))
                .await
                .unwrap();
        }

        // operator -> ncn and operator -> vault
        let mut ixs: Vec<_> = operator_roots
            .iter()
            .map(|operator_root| {
                initialize_operator_ncn_ticket(
                    &jito_restaking_program::id(),
                    &restaking_config_pubkey,
                    &operator_root.operator_pubkey,
                    &ncn_root.ncn_pubkey,
                    &OperatorNcnTicket::find_program_address(
                        &jito_restaking_program::id(),
                        &operator_root.operator_pubkey,
                        &ncn_root.ncn_pubkey,
                    )
                    .0,
                    &operator_admin.pubkey(),
                    &operator_admin.pubkey(),
                )
            })
            .collect();

        ixs.extend(
            operator_roots
                .iter()
                .map(|operator_root| {
                    initialize_operator_vault_ticket(
                        &jito_restaking_program::id(),
                        &restaking_config_pubkey,
                        &operator_root.operator_pubkey,
                        &vault_root.vault_pubkey,
                        &OperatorVaultTicket::find_program_address(
                            &jito_restaking_program::id(),
                            &operator_root.operator_pubkey,
                            &vault_root.vault_pubkey,
                        )
                        .0,
                        &operator_admin.pubkey(),
                        &operator_admin.pubkey(),
                    )
                })
                .collect::<Vec<_>>(),
        );
        for ixs_chunk in ixs.chunks(32) {
            let blockhash = restaking_client.latest_blockhash().await.unwrap();
            restaking_client
                .process_transaction(&Transaction::new_signed_with_payer(
                    &ixs_chunk,
                    Some(&restaking_client.payer.pubkey()),
                    &[&restaking_client.payer, &operator_admin],
                    blockhash,
                ))
                .await
                .unwrap();
        }

        fixture
            .warp_slot_incremental(2 * restaking_config.epoch_length)
            .await
            .unwrap();

        // ncn -> operator
        let ixs: Vec<_> = operator_roots
            .iter()
            .map(|root| {
                initialize_ncn_operator_ticket(
                    &jito_restaking_program::id(),
                    &restaking_config_pubkey,
                    &ncn_root.ncn_pubkey,
                    &root.operator_pubkey,
                    &NcnOperatorTicket::find_program_address(
                        &jito_restaking_program::id(),
                        &ncn_root.ncn_pubkey,
                        &root.operator_pubkey,
                    )
                    .0,
                    &OperatorNcnTicket::find_program_address(
                        &jito_restaking_program::id(),
                        &root.operator_pubkey,
                        &ncn_root.ncn_pubkey,
                    )
                    .0,
                    &ncn_root.ncn_admin.pubkey(),
                    &ncn_root.ncn_admin.pubkey(),
                )
            })
            .collect();
        for ixs_chunk in ixs.chunks(32) {
            let blockhash = restaking_client.latest_blockhash().await.unwrap();
            restaking_client
                .process_transaction(&Transaction::new_signed_with_payer(
                    &ixs_chunk,
                    Some(&restaking_client.payer.pubkey()),
                    &[&restaking_client.payer, &ncn_root.ncn_admin],
                    blockhash,
                ))
                .await
                .unwrap();
        }

        // vault -> operator
        let vault_config_pubkey = VaultConfig::find_program_address(&jito_vault_program::id()).0;
        let ixs: Vec<_> = operator_roots
            .iter()
            .map(|operator_root| {
                initialize_vault_operator_ticket(
                    &jito_vault_program::id(),
                    &vault_config_pubkey,
                    &vault_root.vault_pubkey,
                    &operator_root.operator_pubkey,
                    &OperatorVaultTicket::find_program_address(
                        &jito_restaking_program::id(),
                        &operator_root.operator_pubkey,
                        &vault_root.vault_pubkey,
                    )
                    .0,
                    &VaultOperatorTicket::find_program_address(
                        &jito_vault_program::id(),
                        &vault_root.vault_pubkey,
                        &operator_root.operator_pubkey,
                    )
                    .0,
                    &vault_root.vault_admin.pubkey(),
                    &vault_root.vault_admin.pubkey(),
                )
            })
            .collect();
        for ixs_chunk in ixs.chunks(24) {
            let blockhash = restaking_client.latest_blockhash().await.unwrap();
            restaking_client
                .process_transaction(&Transaction::new_signed_with_payer(
                    &ixs_chunk,
                    Some(&restaking_client.payer.pubkey()),
                    &[&restaking_client.payer, &vault_root.vault_admin],
                    blockhash,
                ))
                .await
                .unwrap();
        }

        // let those bake
        fixture
            .warp_slot_incremental(2 * restaking_config.epoch_length)
            .await
            .unwrap();

        let vault = vault_client
            .get_vault(&vault_root.vault_pubkey)
            .await
            .unwrap();

        // mint tokens to depositor
        let depositor = Keypair::new();
        fixture.transfer(&depositor.pubkey(), 1.0).await.unwrap();
        fixture
            .mint_to(&vault.supported_mint, &depositor.pubkey(), MINT_AMOUNT)
            .await
            .unwrap();

        // depositor ATA for VRT
        fixture
            .create_ata(&vault.vrt_mint, &depositor.pubkey())
            .await
            .unwrap();

        vault_client
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

        vault_client
            .do_update_vault(&vault_root.vault_pubkey)
            .await
            .unwrap();

        let amount_per_operator = MINT_AMOUNT / NUM_OPERATORS as u64;
        let ixs: Vec<_> = operator_roots
            .iter()
            .map(|root| {
                add_delegation(
                    &jito_vault_program::id(),
                    &vault_config_pubkey,
                    &vault_root.vault_pubkey,
                    &root.operator_pubkey,
                    &VaultOperatorTicket::find_program_address(
                        &jito_vault_program::id(),
                        &vault_root.vault_pubkey,
                        &root.operator_pubkey,
                    )
                    .0,
                    &VaultDelegationList::find_program_address(
                        &jito_vault_program::id(),
                        &vault_root.vault_pubkey,
                    )
                    .0,
                    &vault_root.vault_admin.pubkey(),
                    &vault_root.vault_admin.pubkey(),
                    amount_per_operator,
                )
            })
            .collect();
        for ix_chunk in ixs.chunks(12) {
            let blockhash = restaking_client.latest_blockhash().await.unwrap();
            restaking_client
                .process_transaction(&Transaction::new_signed_with_payer(
                    &ix_chunk,
                    Some(&restaking_client.payer.pubkey()),
                    &[&restaking_client.payer, &vault_root.vault_admin],
                    blockhash,
                ))
                .await
                .unwrap();
        }

        fixture
            .warp_slot_incremental(2 * restaking_config.epoch_length)
            .await
            .unwrap();

        vault_client
            .do_update_vault(&vault_root.vault_pubkey)
            .await
            .unwrap();

        let vault_staker_withdrawal_ticket_base = Keypair::new();
        let vault_staker_withdrawal_pubkey = VaultStakerWithdrawalTicket::find_program_address(
            &jito_vault_program::id(),
            &vault_root.vault_pubkey,
            &depositor.pubkey(),
            &vault_staker_withdrawal_ticket_base.pubkey(),
        )
        .0;
        fixture
            .create_ata(&vault.vrt_mint, &vault_staker_withdrawal_pubkey)
            .await
            .unwrap();
        vault_client
            .enqueue_withdraw(
                &vault_config_pubkey,
                &vault_root.vault_pubkey,
                &VaultDelegationList::find_program_address(
                    &jito_vault_program::id(),
                    &vault_root.vault_pubkey,
                )
                .0,
                &vault_staker_withdrawal_pubkey,
                &get_associated_token_address(&vault_staker_withdrawal_pubkey, &vault.vrt_mint),
                &get_associated_token_address(&vault.fee_wallet, &vault.vrt_mint),
                &depositor,
                &get_associated_token_address(&depositor.pubkey(), &vault.vrt_mint),
                &vault_staker_withdrawal_ticket_base,
                MINT_AMOUNT - 1,
            )
            .await
            .unwrap_err();
    }
}
