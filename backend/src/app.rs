use axum::Router;
use sqlx::PgPool;
use crate::routes::{portfolios, positions, prices, analytics};

pub fn create_app(pool: PgPool) -> Router {
    Router::new()
        .nest("/api/portfolios", portfolios::router())
        .nest("/api/positions", positions::router())
        .nest("/api/prices", prices::router())
        .nest("/api/analytics", analytics::router())
        .with_state(pool)
}