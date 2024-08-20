use thiserror::Error;

pub type Result<T> = std::result::Result<T, VaultClientError>;

#[derive(Debug, Error)]
pub enum VaultClientError {}
