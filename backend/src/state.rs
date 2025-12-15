use std::sync::Arc;
use sqlx::PgPool;
use crate::external::price_provider::PriceProvider;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub price_provider: Arc<dyn PriceProvider>,
}