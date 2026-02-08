use axum::Router;

use crate::routes::{portfolios, positions, prices, analytics, health, accounts, imports, cash_flows, transactions, admin};
use crate::state::AppState;
use tower_http::cors::{AllowOrigin, CorsLayer};
use http::header::{AUTHORIZATION, CONTENT_TYPE, HeaderValue};
use http::Method;



pub fn create_app(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::predicate(|origin: &HeaderValue, _| {
            origin.as_bytes().starts_with(b"http://localhost:")
                || origin.as_bytes().starts_with(b"http://127.0.0.1:")
        }))
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::OPTIONS])
        .allow_headers([CONTENT_TYPE, AUTHORIZATION]);
    Router::<AppState>::new()
        .nest("/health", health::router())
        .nest("/api/portfolios", portfolios::router())
        .nest("/api", accounts::router())
        .nest("/api", imports::router())
        .nest("/api", cash_flows::router())
        .nest("/api", transactions::router())
        .nest("/api", admin::router())
        .nest("/api/positions", positions::router())
        .nest("/api/prices", prices::router())
        .nest("/api/analytics", analytics::router())
        .with_state(state)
        .layer(cors)
}