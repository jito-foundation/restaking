#[cfg(test)]
mod tests {
    use jito_vault_core::{
        config::Config, vault_operator_delegation::VaultOperatorDelegation,
        vault_update_state_tracker::VaultUpdateStateTracker,
    };
    use solana_sdk::{signature::Keypair, signer::Signer};

    use crate::fixtures::{
        fixture::{ConfiguredVault, TestBuilder},
        vault_client::VaultRoot,
    };

    const MINT_AMOUNT: u64 = 100_000;
    const DEPOSIT_FEE_BPS: u16 = 0;
    const WITHDRAW_FEE_BPS: u16 = 0;
    const REWARD_FEE_BPS: u16 = 1000; // 10%
    const EPOCH_WITHDRAW_CAP_BPS: u16 = 2500; // 25%

    async fn setup() -> (TestBuilder, VaultRoot) {
        let num_operators = 1;
        let slasher_amounts = vec![];

        let mut fixture = TestBuilder::new().await;
        let ConfiguredVault {
            mut vault_program_client,
            restaking_program_client: _,
            vault_config_admin: _,
            vault_root,
            restaking_config_admin: _,
            ncn_root: _,
            operator_roots,
            slashers_amounts: _,
        } = fixture
            .setup_vault_with_ncn_and_operators(
                DEPOSIT_FEE_BPS,
                WITHDRAW_FEE_BPS,
                REWARD_FEE_BPS,
                EPOCH_WITHDRAW_CAP_BPS,
                num_operators,
                &slasher_amounts,
            )
            .await
            .unwrap();

        // Initial deposit + mint
        let depositor = Keypair::new();
        vault_program_client
            .configure_depositor(&vault_root, &depositor.pubkey(), MINT_AMOUNT)
            .await
            .unwrap();

        // Reward vault instead of staking
        vault_program_client
            .create_and_fund_reward_vault(&vault_root.vault_pubkey, &depositor, MINT_AMOUNT)
            .await
            .unwrap();

        let config = vault_program_client
            .get_config(&Config::find_program_address(&jito_vault_program::id()).0)
            .await
            .unwrap();
        fixture
            .warp_slot_incremental(config.epoch_length())
            .await
            .unwrap();

        let slot = fixture.get_current_slot().await.unwrap();
        let vault_update_state_tracker = VaultUpdateStateTracker::find_program_address(
            &jito_vault_program::id(),
            &vault_root.vault_pubkey,
            slot / config.epoch_length(),
        )
        .0;
        vault_program_client
            .initialize_vault_update_state_tracker(
                &vault_root.vault_pubkey,
                &vault_update_state_tracker,
            )
            .await
            .unwrap();

        for operator in operator_roots {
            vault_program_client
                .crank_vault_update_state_tracker(
                    &vault_root.vault_pubkey,
                    &operator.operator_pubkey,
                    &VaultOperatorDelegation::find_program_address(
                        &jito_vault_program::id(),
                        &vault_root.vault_pubkey,
                        &operator.operator_pubkey,
                    )
                    .0,
                    &vault_update_state_tracker,
                )
                .await
                .unwrap();
        }

        vault_program_client
            .close_vault_update_state_tracker(
                &vault_root.vault_pubkey,
                &vault_update_state_tracker,
                slot / config.epoch_length(),
            )
            .await
            .unwrap();

        (fixture, vault_root)
    }

    #[tokio::test]
    async fn test_update_vault_balance_ok() {
        let (fixture, vault_root) = setup().await;
        let mut vault_program_client = fixture.vault_program_client();

        vault_program_client
            .update_vault_balance(&vault_root.vault_pubkey)
            .await
            .unwrap();

        let vault = vault_program_client
            .get_vault(&vault_root.vault_pubkey)
            .await
            .unwrap();

        let reward_fee_account = vault_program_client
            .get_reward_fee_token_account(&vault_root.vault_pubkey)
            .await
            .unwrap();

        assert_eq!(vault.tokens_deposited(), MINT_AMOUNT);
        assert_eq!(reward_fee_account.amount, MINT_AMOUNT / 10);
        assert_eq!(vault.vrt_supply(), MINT_AMOUNT / 10);
    }
}
