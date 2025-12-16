extern crate core;

mod db;
mod routes;
mod models;
mod errors;
mod utils;
mod app;
mod services;
mod external;
mod state;

use axum::{routing::get, Router};
use std::net::SocketAddr;
use std::sync::Arc;
use sqlx::postgres::PgPoolOptions;
use tokio::net::TcpListener;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use crate::external::alphavantage::AlphaVantageProvider;
use crate::state::AppState;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL")?;

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await?;

    let provider = Arc::new(
        AlphaVantageProvider::from_env()
            .expect("Failed to create AlphaVantageProvider (check ALPHAVANTAGE_API_KEY)"));
    // Initialize logging
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new("info"))
        .with(tracing_subscriber::fmt::layer())
        .init();
    let state = AppState {
        pool,
        price_provider: provider,
    };
    let app = app::create_app(state);

/*    let app = Router::new()
        .route("/health", get(health_check))
        .route("/", get(root));*/

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    let listener = TcpListener::bind(&addr).await.unwrap();
    tracing::info!("🚀 Rustfolio backend running at http://{}/", addr);
    axum::serve(listener, app)
        .await?;
    
    Ok(())
}

async fn health_check() -> &'static str {
    "OK"
}

async fn root() -> &'static str {
    "Rustfolio backend is alive"
}
