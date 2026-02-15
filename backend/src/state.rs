use std::sync::Arc;
use sqlx::PgPool;
use crate::external::price_provider::PriceProvider;
use crate::services::failure_cache::FailureCache;
use crate::services::llm_service::LlmService;
use crate::services::news_service::NewsService;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub price_provider: Arc<dyn PriceProvider>,
    pub failure_cache: FailureCache,
    pub risk_free_rate: f64, // Annual risk-free rate (e.g., 0.045 for 4.5%)
    pub llm_service: Arc<LlmService>,
    pub news_service: Arc<NewsService>,
}