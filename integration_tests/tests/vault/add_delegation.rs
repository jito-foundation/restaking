#[cfg(test)]
mod tests {

    #[tokio::test]
    async fn test_add_delegation_ok() {
        // let mut fixture = TestBuilder::new().await;
        // let ConfiguredVault {
        //     vault_program_client,
        //     restaking_program_client,
        //     vault_config_admin,
        //     vault_root,
        //     restaking_config_admin,
        //     ncn_root,
        //     operator_roots,
        //     slashers_amounts,
        // } = fixture
        //     .setup_vault_with_ncn_and_operators(0, 0, 1, &[])
        //     .await
        //     .unwrap();
    }

    #[tokio::test]
    async fn test_add_delegation_vault_out_of_date_fails() {}

    #[tokio::test]
    async fn test_add_delegation_vault_operator_delegation_not_active_fails() {}

    #[tokio::test]
    async fn test_add_delegation_wrong_admin_fails() {}

    #[tokio::test]
    async fn test_add_delegation_over_delegate_fails() {}
}
