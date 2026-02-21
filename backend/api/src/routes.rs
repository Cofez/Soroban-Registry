use axum::{
    routing::get,
    Router,
};

use crate::{handlers, state::AppState};

pub fn observability_routes() -> Router<AppState> {
    Router::new()
}

pub fn contract_routes() -> Router<AppState> {
    Router::new()
        .route("/api/contracts", get(handlers::list_contracts))
        .route("/api/contracts/:id", get(handlers::get_contract))
        .route("/api/contracts/:id/abi", get(handlers::get_contract_abi))
}

pub fn publisher_routes() -> Router<AppState> {
    Router::new()
}

pub fn health_routes() -> Router<AppState> {
    Router::new()
        .route("/health", get(handlers::health_check))
        .route("/api/stats", get(handlers::get_stats))
}

pub fn migration_routes() -> Router<AppState> {
    Router::new()
}
