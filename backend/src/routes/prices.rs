
use axum::extract::{Path, State};
use axum::{Json, Router};
use axum::http::StatusCode;
use axum::routing::{get, post};
use tracing::{info, error, warn};

use crate::errors::AppError;
use crate::models::PricePoint;
use crate::services;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/:ticker", get(get_prices))
        .route("/:ticker/latest", get(get_latest_price))
        .route("/:ticker/update", post(update_prices))
        .route("/:ticker/mock", post(generate_mock_prices))
}

pub async fn get_prices(
    Path(ticker): Path<String>,
    State(state): State<AppState>
) -> Result<Json<Vec<PricePoint>>, AppError> {
    info!("GET /prices/{} - Getting price history", ticker);
    let prices = services::price_service::get_history(&state.pool, &ticker).await
        .map_err(|e| {
            error!("Failed to get price history for {}: {}", ticker, e);
            e
        })?;
    Ok(Json(prices))
}

pub async fn get_latest_price(
    Path(ticker): Path<String>,
    State(state): State<AppState>
) -> Result<Json<PricePoint>, AppError> {
    info!("GET /prices/{}/latest - Getting latest price", ticker);
    let price = services::price_service::get_latest(&state.pool, &ticker).await
        .map_err(|e| {
            error!("Failed to get latest price for {}: {}", ticker, e);
            e
        })?;
    Ok(Json(price))
}

pub async fn update_prices(
    Path(ticker): Path<String>,
    State(state): State<AppState>,
) -> Result<StatusCode, AppError> {
    info!("POST /prices/{}/update - Updating prices from API", ticker);
    services::price_service::refresh_from_api(
        &state.pool,
        state.price_provider.as_ref(),
        &ticker,
    ).await
        .map_err(|e| {
            match &e {
                AppError::RateLimited => warn!("Rate limited when updating prices for {}", ticker),
                _ => error!("Failed to update prices from API for {}: {}", ticker, e),
            }
            e
        })?;
    Ok(StatusCode::OK)
}

pub async fn generate_mock_prices(
    Path(ticker): Path<String>,
    State(state): State<AppState>
) -> Result<StatusCode, AppError> {
    info!("POST /prices/{}/mock - Generating mock prices", ticker);
    services::price_service::generate_mock(&state.pool, &ticker).await
        .map_err(|e| {
            error!("Failed to generate mock prices for {}: {}", ticker, e);
            e
        })?;
    Ok(StatusCode::CREATED)
}