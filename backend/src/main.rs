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
use crate::external::twelvedata::TwelveDataProvider;
use crate::external::multi_provider::MultiProvider;
use crate::state::AppState;
use crate::services::failure_cache::FailureCache;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL")?;

    // Initialize logging FIRST
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new("info"))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await?;

    // Select price provider based on PRICE_PROVIDER env var (defaults to multi)
    let provider_name = std::env::var("PRICE_PROVIDER")
        .unwrap_or_else(|_| "multi".to_string());

    let provider: Arc<dyn crate::external::price_provider::PriceProvider> = match provider_name.to_lowercase().as_str() {
        "alphavantage" => {
            tracing::info!("📊 Using price provider: Alpha Vantage only");
            Arc::new(AlphaVantageProvider::from_env()
                .expect("Failed to create AlphaVantageProvider (check ALPHAVANTAGE_API_KEY)"))
        },
        "twelvedata" => {
            tracing::info!("📊 Using price provider: Twelve Data only");
            Arc::new(TwelveDataProvider::from_env()
                .expect("Failed to create TwelveDataProvider (check TWELVEDATA_API_KEY)"))
        },
        "multi" => {
            tracing::info!("📊 Using price provider: Multi-provider (Twelve Data + Alpha Vantage fallback)");
            let primary = Box::new(TwelveDataProvider::from_env()
                .expect("Failed to create TwelveDataProvider (check TWELVEDATA_API_KEY)"));
            let fallback = Box::new(AlphaVantageProvider::from_env()
                .expect("Failed to create AlphaVantageProvider (check ALPHAVANTAGE_API_KEY)"));
            Arc::new(MultiProvider::new(primary, fallback))
        },
        _ => {
            panic!("Invalid PRICE_PROVIDER: {}. Must be 'alphavantage', 'twelvedata', or 'multi'", provider_name);
        }
    };
    let state = AppState {
        pool,
        price_provider: provider,
        failure_cache: FailureCache::new(),
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
