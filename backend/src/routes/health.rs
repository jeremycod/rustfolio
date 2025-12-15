use axum::{
    Router,
    routing::get,
};
use tracing::info;

use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(health))
}

async fn health() -> &'static str {
    info!("GET /health - Health check");
    "OK"
}