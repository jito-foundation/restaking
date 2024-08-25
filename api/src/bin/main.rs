use axum::{Router};
use clap::Parser;
use solana_rpc_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::pubkey::{Pubkey};
use std::str::FromStr;
use std::sync::Arc;
use tracing::info;
use std::env;

use jito_restaking_api::common::{AppState, Args};
use jito_restaking_api::vault::routes as vault_routes;
use jito_restaking_api::restaking::routes as restaking_routes;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    // Load the .env file
    dotenv::from_path(&args.program_env)?;

    let restaking_program_id = args.restaking_program_id.unwrap_or_else(|| {
        Pubkey::from_str(&env::var("RESTAKING_PROGRAM_ID").expect("RESTAKING_PROGRAM_ID not found in program.env"))
            .expect("Failed to parse RESTAKING_PROGRAM_ID")
    });

    let vault_program_id = args.vault_program_id.unwrap_or_else(|| {
        Pubkey::from_str(&env::var("VAULT_PROGRAM_ID").expect("VAULT_PROGRAM_ID not found in program.env"))
            .expect("Failed to parse VAULT_PROGRAM_ID")
    });

    let rpc_client = RpcClient::new(args.rpc_url);
    let state = Arc::new(AppState {
        rpc_client,
        restaking_program_id,
        vault_program_id,
    });

    let app = Router::new()
        .merge(vault_routes::vault_routes())
        .merge(restaking_routes::restaking_routes())
        .with_state(state);

    let tcp_listener = tokio::net::TcpListener::bind(&args.addr).await?;
    info!("Server listening on {}", args.addr);
    axum::serve(tcp_listener, app.into_make_service())
        .await?;

    Ok(())
}