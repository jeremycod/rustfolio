use axum::Router;

use crate::routes::{portfolios, positions, prices, analytics, health};
use crate::state::AppState;
use tower_http::cors::{Any, CorsLayer};
use http::header::{AUTHORIZATION, CONTENT_TYPE};
use http::Method;



pub fn create_app(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin("http://localhost:5173".parse::<http::HeaderValue>().unwrap())
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::OPTIONS])
        .allow_headers([CONTENT_TYPE, AUTHORIZATION]);
    Router::<AppState>::new()
        .nest("/health", health::router())
        .nest("/api/portfolios", portfolios::router())
        .nest("/api/positions", positions::router())
        .nest("/api/prices", prices::router())
        .nest("/api/analytics", analytics::router())
        .with_state(state)
        .layer(cors)
}