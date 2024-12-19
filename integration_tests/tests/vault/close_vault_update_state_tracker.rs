#[cfg(test)]
mod tests {
    use jito_vault_core::{
        config::Config, delegation_state::DelegationState, vault::BurnSummary,
        vault_operator_delegation::VaultOperatorDelegation,
        vault_update_state_tracker::VaultUpdateStateTracker,
    };
    use jito_vault_sdk::{error::VaultError, instruction::VaultAdminRole};
    use solana_sdk::{
        msg,
        pubkey::Pubkey,
        signature::{Keypair, Signer},
    };
    use spl_associated_token_account::get_associated_token_address;

    use crate::fixtures::{
        fixture::{ConfiguredVault, TestBuilder},
        vault_client::{assert_vault_error, VaultStakerWithdrawalTicketRoot},
    };

    #[tokio::test]
    async fn test_dos_vault_if_state_tracker_not_closed_when_additional_assets_need_unstaking_is_zero(
    ) {
        let mut fixture = TestBuilder::new().await;

        let deposit_fee_bps = 0;
        let withdrawal_fee_bps = 0;
        let reward_fee_bps = 0;
        let num_operators = 1;
        let slasher_amounts = vec![];

        let ConfiguredVault {
            mut vault_program_client,
            vault_root,
            operator_roots,
            ..
        } = fixture
            .setup_vault_with_ncn_and_operators(
                deposit_fee_bps,
                withdrawal_fee_bps,
                reward_fee_bps,
                num_operators,
                &slasher_amounts,
            )
            .await
            .unwrap();

        let depositor = Keypair::new();
        vault_program_client
            .configure_depositor(&vault_root, &depositor.pubkey(), 100_000)
            .await
            .unwrap();
        vault_program_client
            .do_mint_to(&vault_root, &depositor, 100_000, 100_000)
            .await
            .unwrap();

        vault_program_client
            .do_add_delegation(&vault_root, &operator_roots[0].operator_pubkey, 100_000)
            .await
            .unwrap();

        let config = vault_program_client
            .get_config(&Config::find_program_address(&jito_vault_program::id()).0)
            .await
            .unwrap();

        let VaultStakerWithdrawalTicketRoot { base: _ } = vault_program_client
            .do_enqueue_withdrawal(&vault_root, &depositor, 10_000)
            .await
            .unwrap();
        let vault = vault_program_client
            .get_vault(&vault_root.vault_pubkey)
            .await
            .unwrap();
        assert_eq!(vault.vrt_enqueued_for_cooldown_amount(), 10_000);
        assert_eq!(vault.vrt_cooling_down_amount(), 0);
        assert_eq!(vault.vrt_ready_to_claim_amount(), 0);

        fixture
            .warp_slot_incremental(config.epoch_length())
            .await
            .unwrap();

        let operator_pubkeys: Vec<_> = operator_roots
            .iter()
            .map(|root| root.operator_pubkey)
            .collect();

        //start of full vault update
        let slot = fixture.get_current_slot().await.unwrap();

        let ncn_epoch = slot / config.epoch_length();

        let vault_update_state_tracker = VaultUpdateStateTracker::find_program_address(
            &jito_vault_program::id(),
            &vault_root.vault_pubkey,
            ncn_epoch,
        )
        .0;
        //initialize vault_update_state_tracker
        vault_program_client
            .initialize_vault_update_state_tracker(
                &vault_root.vault_pubkey,
                &vault_update_state_tracker,
            )
            .await
            .unwrap();
        //crank update all operators so addoitional_assets_need_unstaking=0
        for i in 0..operator_pubkeys.len() {
            let operator_index = (i + (ncn_epoch as usize)) % operator_pubkeys.len();
            let operator = &operator_pubkeys[operator_index];
            vault_program_client
                .crank_vault_update_state_tracker(
                    &vault_root.vault_pubkey,
                    operator,
                    &VaultOperatorDelegation::find_program_address(
                        &jito_vault_program::id(),
                        &vault_root.vault_pubkey,
                        operator,
                    )
                    .0,
                    &vault_update_state_tracker,
                )
                .await
                .unwrap();
        }
        //fast forward to next epoch without closing update_state_tracker even though all operators have been updated
        fixture
            .warp_slot_incremental(config.epoch_length())
            .await
            .unwrap();
        //close the update state tracker(not necessary though)
        vault_program_client
            .close_vault_update_state_tracker(
                &vault_root.vault_pubkey,
                &vault_update_state_tracker,
                slot / config.epoch_length(),
            )
            .await
            .unwrap();

        //end of full vault update

        //initialize another update state tracker
        let slot = fixture.get_current_slot().await.unwrap();

        let ncn_epoch = slot / config.epoch_length();

        let vault_update_state_tracker = VaultUpdateStateTracker::find_program_address(
            &jito_vault_program::id(),
            &vault_root.vault_pubkey,
            ncn_epoch,
        )
        .0;
        //new epoch: initialize new vault update state tracker
        vault_program_client
            .initialize_vault_update_state_tracker(
                &vault_root.vault_pubkey,
                &vault_update_state_tracker,
            )
            .await
            .unwrap();
        //crank update all operators
        for i in 0..operator_pubkeys.len() {
            let operator_index = (i + (ncn_epoch as usize)) % operator_pubkeys.len();
            let operator = &operator_pubkeys[operator_index];
            vault_program_client
                .crank_vault_update_state_tracker(
                    &vault_root.vault_pubkey,
                    operator,
                    &VaultOperatorDelegation::find_program_address(
                        &jito_vault_program::id(),
                        &vault_root.vault_pubkey,
                        operator,
                    )
                    .0,
                    &vault_update_state_tracker,
                )
                .await
                .unwrap();
        }
        //closing vault_update_state_tracker fails cos additional_assets_need_unstaking>0 as there were no more delegations to decrement from the operators
        let result = vault_program_client
            .close_vault_update_state_tracker(
                &vault_root.vault_pubkey,
                &vault_update_state_tracker,
                slot / config.epoch_length(),
            )
            .await;

        let vault = vault_program_client
            .get_vault(&vault_root.vault_pubkey)
            .await
            .unwrap();

        assert_eq!(vault.additional_assets_need_unstaking(), 0);

        assert!(result.is_ok());

        // Previous result
        // //Error cos additional_assets_need_unstaking>0
        // assert_vault_error(
        //     result,
        //     VaultError::NonZeroAdditionalAssetsNeededForWithdrawalAtEndOfUpdate,
        // );
        // //Now that vault_update_state_tracker can't be closed, all other operations are impossible:withdrawals, delegations, cooldowns...

        // end of block
    }
}
