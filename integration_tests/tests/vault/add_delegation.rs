#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_add_delegation_ok() {}

    #[tokio::test]
    async fn test_add_delegation_vault_out_of_date_fails() {}

    #[tokio::test]
    async fn test_add_delegation_vault_operator_ticket_not_active_fails() {}

    #[tokio::test]
    async fn test_add_delegation_wrong_admin_fails() {}

    #[tokio::test]
    async fn test_add_delegation_over_delegate_fails() {}
}
