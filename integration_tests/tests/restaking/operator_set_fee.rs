#[cfg(test)]
mod tests {
    use jito_restaking_core::config::Config;
    use solana_sdk::instruction::InstructionError;

    use crate::fixtures::{
        assert_ix_error,
        fixture::TestBuilder,
        restaking_client::{OperatorRoot, RestakingProgramClient},
    };

    async fn setup() -> (RestakingProgramClient, OperatorRoot) {
        let fixture = TestBuilder::new().await;
        let mut restaking_program_client = fixture.restaking_program_client();

        restaking_program_client
            .do_initialize_config()
            .await
            .unwrap();

        let _ncn_root = restaking_program_client.do_initialize_ncn().await.unwrap();
        let operator_root: OperatorRoot = restaking_program_client
            .do_initialize_operator()
            .await
            .unwrap();

        (restaking_program_client, operator_root)
    }

    #[tokio::test]
    async fn test_operator_set_fee_ok() {
        let initial_fee_bps = 0;
        let (mut restaking_program_client, operator_root) = setup().await;

        // Check initial fee
        let operator = restaking_program_client
            .get_operator(&operator_root.operator_pubkey)
            .await
            .unwrap();
        assert_eq!(operator.operator_fee_bps, initial_fee_bps.into());

        let restaking_config_pubkey = Config::find_program_address(&jito_restaking_program::id()).0;

        let new_fee_bps = 2000;
        restaking_program_client
            .operator_set_fee(
                &restaking_config_pubkey,
                &operator_root.operator_pubkey,
                &operator_root.operator_admin,
                new_fee_bps,
            )
            .await
            .unwrap();

        let updated_operator = restaking_program_client
            .get_operator(&operator_root.operator_pubkey)
            .await
            .unwrap();
        assert_eq!(updated_operator.operator_fee_bps, new_fee_bps.into());
    }

    #[tokio::test]
    async fn test_operator_set_fee_exceeds_max() {
        let (mut restaking_program_client, operator_root) = setup().await;

        let restaking_config_pubkey = Config::find_program_address(&jito_restaking_program::id()).0;

        let invalid_fee_bps = 10001; // Exceeds maximum allowed fee
        let result = restaking_program_client
            .operator_set_fee(
                &restaking_config_pubkey,
                &operator_root.operator_pubkey,
                &operator_root.operator_admin,
                invalid_fee_bps,
            )
            .await;

        assert_ix_error(result, InstructionError::InvalidArgument);
    }
}
