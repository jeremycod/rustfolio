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
use crate::external::yahoofinance::YahooFinanceProvider;
use crate::external::multi_provider::MultiProvider;
use crate::state::AppState;
use crate::services::failure_cache::FailureCache;
use crate::services::rate_limiter::RateLimiter;
use crate::services::llm_service::{LlmService, LlmConfig};
use crate::services::news_service::{NewsService, NewsConfig};
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
            tracing::info!("📊 Using price provider: Multi-provider (Twelve Data + Alpha Vantage + Yahoo Finance)");
            let primary = Box::new(TwelveDataProvider::from_env()
                .expect("Failed to create TwelveDataProvider (check TWELVEDATA_API_KEY)"));
            let fallback = Box::new(AlphaVantageProvider::from_env()
                .expect("Failed to create AlphaVantageProvider (check ALPHAVANTAGE_API_KEY)"));
            let yahoo = Box::new(YahooFinanceProvider::new());
            Arc::new(MultiProvider::new(primary, fallback, yahoo))
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

    // Initialize LLM service
    let llm_config = LlmConfig {
        enabled: std::env::var("LLM_ENABLED")
            .ok()
            .and_then(|s| s.parse::<bool>().ok())
            .unwrap_or(false),
        provider: std::env::var("LLM_PROVIDER")
            .unwrap_or_else(|_| "openai".to_string()),
        api_key: std::env::var("OPENAI_API_KEY").ok(),
        max_tokens: std::env::var("LLM_MAX_TOKENS")
            .ok()
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(500),
        temperature: std::env::var("LLM_TEMPERATURE")
            .ok()
            .and_then(|s| s.parse::<f32>().ok())
            .unwrap_or(0.7),
    };

    let llm_service = Arc::new(LlmService::new(llm_config));

    if llm_service.is_enabled() {
        tracing::info!("🤖 LLM service enabled");
    } else {
        tracing::info!("🤖 LLM service disabled");
    }

    // Initialize News service
    let news_config = NewsConfig::from_env();
    let news_service = Arc::new(NewsService::new(news_config, llm_service.clone()));

    if news_service.is_enabled() {
        tracing::info!("📰 News service enabled");
    } else {
        tracing::info!("📰 News service disabled");
    }

    // Initialize rate limiter for API calls
    // Allow max 3 concurrent requests, 8 per minute (free tier limit)
    let rate_limiter = Arc::new(RateLimiter::new(3, 8));
    tracing::info!("⏱️  Rate limiter initialized: 3 concurrent, 8 requests/min");

    let state = AppState {
        pool,
        price_provider: provider,
        failure_cache: FailureCache::new(),
        rate_limiter,
        risk_free_rate,
        llm_service,
        news_service,
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
