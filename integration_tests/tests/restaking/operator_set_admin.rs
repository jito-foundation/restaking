#[cfg(test)]
mod tests {
    use jito_restaking_sdk::error::RestakingError;
    use solana_sdk::{signature::Keypair, signer::Signer};

    use crate::fixtures::{
        fixture::TestBuilder,
        restaking_client::{assert_restaking_error, OperatorRoot, RestakingProgramClient},
    };

    async fn setup() -> (RestakingProgramClient, OperatorRoot) {
        let fixture = TestBuilder::new().await;
        let mut restaking_program_client = fixture.restaking_program_client();

        restaking_program_client
            .do_initialize_config()
            .await
            .unwrap();

        let _ncn_root = restaking_program_client.do_initialize_ncn().await.unwrap();
        let operator_root = restaking_program_client
            .do_initialize_operator()
            .await
            .unwrap();

        (restaking_program_client, operator_root)
    }

    #[tokio::test]
    async fn test_operator_set_admin_with_bad_admin() {
        let (mut restaking_program_client, operator_root) = setup().await;

        let bad_admin = Keypair::new();
        restaking_program_client
            ._airdrop(&bad_admin.pubkey(), 10.0)
            .await
            .unwrap();

        let new_admin = Keypair::new();
        let response = restaking_program_client
            .operator_set_admin(&operator_root.operator_pubkey, &bad_admin, &new_admin)
            .await;

        assert_restaking_error(response, RestakingError::OperatorAdminInvalid);
    }

    #[tokio::test]
    async fn test_operator_set_admin() {
        let (mut restaking_program_client, operator_root) = setup().await;

        let new_admin = Keypair::new();
        restaking_program_client
            .operator_set_admin(
                &operator_root.operator_pubkey,
                &operator_root.operator_admin,
                &new_admin,
            )
            .await
            .unwrap();

        let operator = restaking_program_client
            .get_operator(&operator_root.operator_pubkey)
            .await
            .unwrap();

        assert_eq!(operator.admin, new_admin.pubkey());
    }

    #[tokio::test]
    async fn test_operator_update_secondary_admin() {
        let (mut restaking_program_client, operator_root) = setup().await;

        let new_admin = Keypair::new();
        restaking_program_client
            .operator_set_admin(
                &operator_root.operator_pubkey,
                &operator_root.operator_admin,
                &new_admin,
            )
            .await
            .unwrap();

        let operator = restaking_program_client
            .get_operator(&operator_root.operator_pubkey)
            .await
            .unwrap();

        assert_eq!(operator.ncn_admin, new_admin.pubkey());
        assert_eq!(operator.vault_admin, new_admin.pubkey());
        assert_eq!(operator.voter, new_admin.pubkey());
        assert_eq!(operator.withdrawal_admin, new_admin.pubkey());
        assert_eq!(operator.withdrawal_fee_wallet, new_admin.pubkey());
    }
}
