use error::JitoRestakingApiError;

pub mod error;
pub mod router;

pub type Result<T> = std::result::Result<T, JitoRestakingApiError>;
