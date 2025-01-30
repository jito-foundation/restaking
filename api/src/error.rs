use std::convert::Infallible;

use axum::{
    response::{IntoResponse, Response},
    BoxError, Json,
};
use http::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::json;
use solana_program::pubkey::ParsePubkeyError;
use solana_rpc_client_api::client_error::Error as RpcError;
use thiserror::Error;
use tracing::error;

#[derive(Debug, Serialize, Deserialize)]
pub struct Error {
    pub error: String,
}

#[derive(Error, Debug)]
pub enum JitoRestakingApiError {
    #[error("Rpc Error")]
    RpcError(#[from] RpcError),

    #[error("Parse Pubkey Error")]
    ParsePubkeyError(#[from] ParsePubkeyError),

    #[error("Anchor Error")]
    AnchorError(#[from] anchor_lang::error::Error),

    #[error("Reqwest Error")]
    ReqwestError(#[from] reqwest::Error),

    #[error("Internal Error")]
    InternalError,
}

impl IntoResponse for JitoRestakingApiError {
    fn into_response(self) -> Response {
        let status = StatusCode::INTERNAL_SERVER_ERROR;
        let error_message = self.get_error_message();

        self.log_error();

        (
            status,
            Json(Error {
                error: error_message.to_string(),
            }),
        )
            .into_response()
    }
}

impl JitoRestakingApiError {
    /// Helper function to map errors to their message
    const fn get_error_message(&self) -> &'static str {
        match self {
            Self::RpcError(_) => "Rpc error",
            Self::ParsePubkeyError(_) => "Pubkey parse error",
            Self::AnchorError(_) | Self::ReqwestError(_) | Self::InternalError => {
                "Internal Server Error"
            }
        }
    }

    /// Logs the error, extracting the error details separately
    fn log_error(&self) {
        match self {
            Self::RpcError(e) => self.log("Rpc error", e),
            Self::ParsePubkeyError(e) => self.log("Parse pubkey error", e),
            Self::AnchorError(e) => self.log("Anchor error", e),
            Self::ReqwestError(e) => self.log("Reqwest error", e),
            Self::InternalError => self.log("Internal server error", ""),
        }
    }

    /// Helper function to log messages
    fn log<T: std::fmt::Display>(&self, prefix: &str, err: T) {
        error!("{}: {}", prefix, err);
    }
}

pub async fn handle_error(error: BoxError) -> Result<impl IntoResponse, Infallible> {
    if error.is::<tower::timeout::error::Elapsed>() {
        return Ok((
            StatusCode::REQUEST_TIMEOUT,
            Json(json!({
                "code" : 408,
                "error" : "Request Timeout",
            })),
        ));
    };
    if error.is::<tower::load_shed::error::Overloaded>() {
        return Ok((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({
                "code" : 503,
                "error" : "Service Unavailable",
            })),
        ));
    }

    Ok((
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({
            "code" : 500,
            "error" : "Internal Server Error",
        })),
    ))
}
