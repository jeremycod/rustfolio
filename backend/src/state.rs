use std::sync::Arc;
use sqlx::PgPool;
use crate::external::price_provider::PriceProvider;
use crate::services::failure_cache::FailureCache;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub price_provider: Arc<dyn PriceProvider>,
    pub failure_cache: FailureCache,
}