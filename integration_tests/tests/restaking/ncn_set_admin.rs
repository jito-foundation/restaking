#[cfg(test)]
mod tests {
    use jito_restaking_sdk::error::RestakingError;
    use solana_sdk::{signature::Keypair, signer::Signer};

    use crate::fixtures::{
        fixture::TestBuilder,
        restaking_client::{assert_restaking_error, NcnRoot, RestakingProgramClient},
    };

    async fn setup() -> (RestakingProgramClient, NcnRoot) {
        let fixture = TestBuilder::new().await;
        let mut restaking_program_client = fixture.restaking_program_client();

        restaking_program_client
            .do_initialize_config()
            .await
            .unwrap();

        let ncn_root = restaking_program_client.do_initialize_ncn().await.unwrap();

        (restaking_program_client, ncn_root)
    }

    #[tokio::test]
    async fn test_ncn_set_admin_with_bad_admin() {
        let (mut restaking_program_client, ncn_root) = setup().await;

        let bad_admin = Keypair::new();
        restaking_program_client
            ._airdrop(&bad_admin.pubkey(), 10.0)
            .await
            .unwrap();

        let new_admin = Keypair::new();
        let response = restaking_program_client
            .ncn_set_admin(&ncn_root.ncn_pubkey, &bad_admin, &new_admin)
            .await;

        assert_restaking_error(response, RestakingError::NcnAdminInvalid);
    }

    #[tokio::test]
    async fn test_ncn_set_admin() {
        let (mut restaking_program_client, ncn_root) = setup().await;

        let new_admin = Keypair::new();
        restaking_program_client
            .ncn_set_admin(&ncn_root.ncn_pubkey, &ncn_root.ncn_admin, &new_admin)
            .await
            .unwrap();

        let ncn = restaking_program_client
            .get_ncn(&ncn_root.ncn_pubkey)
            .await
            .unwrap();

        assert_eq!(ncn.admin, new_admin.pubkey());
    }

    #[tokio::test]
    async fn test_ncn_update_secondary_admin() {
        let (mut restaking_program_client, ncn_root) = setup().await;

        let new_admin = Keypair::new();
        restaking_program_client
            .ncn_set_admin(&ncn_root.ncn_pubkey, &ncn_root.ncn_admin, &new_admin)
            .await
            .unwrap();

        let ncn = restaking_program_client
            .get_ncn(&ncn_root.ncn_pubkey)
            .await
            .unwrap();

        assert_eq!(ncn.operator_admin, new_admin.pubkey());
        assert_eq!(ncn.vault_admin, new_admin.pubkey());
        assert_eq!(ncn.slasher_admin, new_admin.pubkey());
        assert_eq!(ncn.delegate_admin, new_admin.pubkey());
    }
}
