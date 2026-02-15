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
mod logging;

use std::net::SocketAddr;
use std::sync::Arc;
use sqlx::postgres::PgPoolOptions;
use tokio::net::TcpListener;
use crate::external::alphavantage::AlphaVantageProvider;
use crate::external::twelvedata::TwelveDataProvider;
use crate::external::multi_provider::MultiProvider;
use crate::state::AppState;
use crate::services::failure_cache::FailureCache;
use crate::logging::{LoggingConfig, init_logging};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    // Initialize logging FIRST
    let logging_config = LoggingConfig::from_env();
    init_logging(logging_config)?;

    let database_url = std::env::var("DATABASE_URL")?;

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
    // Read risk-free rate from environment (default to 4.5% = 0.045 annual rate)
    let risk_free_rate = std::env::var("RISK_FREE_RATE")
        .ok()
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(0.045); // Default: 4.5% (US 10-year Treasury approximation)

    tracing::info!("📈 Risk-free rate set to: {:.2}%", risk_free_rate * 100.0);

    let state = AppState {
        pool,
        price_provider: provider,
        failure_cache: FailureCache::new(),
        risk_free_rate,
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
