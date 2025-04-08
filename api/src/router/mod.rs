mod vaults;

use std::{sync::Arc, time::Duration};

use axum::{
    body::Body, error_handling::HandleErrorLayer, response::IntoResponse, routing::get, Router,
};
use http::StatusCode;
use solana_rpc_client::nonblocking::rpc_client::RpcClient;
use tower::{
    buffer::BufferLayer, limit::RateLimitLayer, load_shed::LoadShedLayer, timeout::TimeoutLayer,
    ServiceBuilder,
};
use tower_http::{
    trace::{DefaultOnResponse, TraceLayer},
    LatencyUnit,
};
use tracing::{info, instrument, Span};
use vaults::{
    tvl::{get_tvl, get_tvls},
    vault::{get_vault, list_vaults},
};

pub struct RouterState {
    pub rpc_client: RpcClient,
}

impl std::fmt::Debug for RouterState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RouterState")
            .field("rpc_client", &self.rpc_client.url())
            .finish()
    }
}

#[instrument]
pub fn get_routes(state: Arc<RouterState>) -> Router {
    let middleware = ServiceBuilder::new()
        .layer(HandleErrorLayer::new(crate::error::handle_error))
        .layer(BufferLayer::new(1000))
        .layer(RateLimitLayer::new(10000, Duration::from_secs(1)))
        .layer(TimeoutLayer::new(Duration::from_secs(20)))
        .layer(LoadShedLayer::new())
        .layer(
            TraceLayer::new_for_http()
                .on_request(|request: &http::Request<Body>, _span: &Span| {
                    info!("started {} {}", request.method(), request.uri().path())
                })
                .on_response(
                    DefaultOnResponse::new()
                        .level(tracing_core::Level::INFO)
                        .latency_unit(LatencyUnit::Millis),
                ),
        );

    let vault_routes = Router::new()
        .route("/", get(list_vaults))
        .route("/tvl", get(get_tvls))
        .route("/:vault_pubkey", get(get_vault))
        .route("/:vault_pubkey/tvl", get(get_tvl));

    let api_routes = Router::new()
        .route("/", get(root))
        .nest("/vaults", vault_routes);

    let app = Router::new().nest("/api/v1", api_routes).fallback(fallback);

    app.layer(middleware).with_state(state)
}

async fn root() -> impl IntoResponse {
    "Jito Restaking API"
}

async fn fallback() -> (StatusCode, &'static str) {
    (StatusCode::NOT_FOUND, "Not Found")
}
