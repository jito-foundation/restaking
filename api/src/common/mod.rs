use axum::response::{IntoResponse, Response};
use axum::Json;
use axum::http::StatusCode;
use clap::Parser;
use serde_json::json;
use solana_rpc_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::pubkey::{Pubkey, ParsePubkeyError};
use std::net::SocketAddr;
use std::path::PathBuf;
use thiserror::Error;

pub struct AppState {
    pub rpc_client: RpcClient,
    pub restaking_program_id: Pubkey,
    pub vault_program_id: Pubkey,
}

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Failed to parse pubkey: {0}")]
    PubkeyParseError(#[from] ParsePubkeyError),
    #[error("RPC error: {0}")]
    RpcError(#[from] solana_rpc_client_api::client_error::Error),
    #[error("Deserialization error: {0}")]
    DeserializationError(String),
    #[error("Internal server error: {0}")]
    InternalError(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            ApiError::PubkeyParseError(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            ApiError::RpcError(_) => (StatusCode::SERVICE_UNAVAILABLE, self.to_string()),
            ApiError::DeserializationError(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            ApiError::InternalError(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        };

        let body = Json(json!({
            "error": error_message,
        }));

        (status, body).into_response()
    }
}

#[derive(Parser)]
pub struct Args {
    #[clap(short, long, help = "RPC URL")]
    pub rpc_url: String,

    #[clap(short, long, help = "Address to bind to", default_value = "127.0.0.1:3000")]
    pub addr: SocketAddr,

    #[clap(long, help = "Restaking program ID")]
    pub restaking_program_id: Option<Pubkey>,

    #[clap(long, help = "Vault program ID")]
    pub vault_program_id: Option<Pubkey>,

    #[clap(long, help = "Path to program.env file", default_value = "config/program.env")]
    pub program_env: PathBuf,
}