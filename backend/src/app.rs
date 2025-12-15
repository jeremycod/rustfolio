use axum::Router;

use crate::routes::{portfolios, positions, prices, analytics, health};
use crate::state::AppState;

pub fn create_app(state: AppState) -> Router {
    Router::<AppState>::new()
        .nest("/health", health::router())
        .nest("/api/portfolios", portfolios::router())
        .nest("/api/positions", positions::router())
        .nest("/api/prices", prices::router())
        .nest("/api/analytics", analytics::router())
        .with_state(state)
}