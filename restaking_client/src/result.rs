use thiserror::Error;

pub type Result<T> = std::result::Result<T, RpcClientError>;

#[derive(Debug, Error)]
pub enum RpcClientError {}
