use axum::extract::{Path, State};
use axum::{Json, Router};
use axum::routing::get;
use tracing::{info, error};
use uuid::Uuid;

use crate::db::detected_transaction_queries;
use crate::errors::AppError;
use crate::models::{AccountActivity, AccountTruePerformance, DetectedTransaction};
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/accounts/:account_id/transactions", get(list_transactions))
        .route("/accounts/:account_id/activity", get(get_activity))
        .route("/accounts/:account_id/true-performance", get(get_true_performance))
        .route("/portfolios/:portfolio_id/true-performance", get(get_portfolio_true_performance))
}

pub async fn list_transactions(
    State(state): State<AppState>,
    Path(account_id): Path<Uuid>,
) -> Result<Json<Vec<DetectedTransaction>>, AppError> {
    info!("GET /accounts/{}/transactions - Listing transactions", account_id);

    let transactions = detected_transaction_queries::fetch_by_account(&state.pool, account_id)
        .await
        .map_err(|e| {
            error!("Failed to fetch transactions: {}", e);
            AppError::Db(e)
        })?;

    Ok(Json(transactions))
}

pub async fn get_activity(
    State(state): State<AppState>,
    Path(account_id): Path<Uuid>,
) -> Result<Json<Vec<AccountActivity>>, AppError> {
    info!("GET /accounts/{}/activity - Getting account activity", account_id);

    let activity = detected_transaction_queries::fetch_account_activity(&state.pool, account_id)
        .await
        .map_err(|e| {
            error!("Failed to fetch account activity: {}", e);
            AppError::Db(e)
        })?;

    Ok(Json(activity))
}

pub async fn get_true_performance(
    State(state): State<AppState>,
    Path(account_id): Path<Uuid>,
) -> Result<Json<AccountTruePerformance>, AppError> {
    info!("GET /accounts/{}/true-performance - Getting true performance", account_id);

    let performance = detected_transaction_queries::fetch_true_performance(&state.pool, account_id)
        .await
        .map_err(|e| {
            error!("Failed to fetch true performance: {}", e);
            AppError::Db(e)
        })?
        .ok_or_else(|| {
            error!("Account {} not found", account_id);
            AppError::NotFound(format!("Account {} not found", account_id))
        })?;

    Ok(Json(performance))
}

pub async fn get_portfolio_true_performance(
    State(state): State<AppState>,
    Path(portfolio_id): Path<Uuid>,
) -> Result<Json<Vec<AccountTruePerformance>>, AppError> {
    info!("GET /portfolios/{}/true-performance - Getting portfolio true performance", portfolio_id);

    let performance = detected_transaction_queries::fetch_all_true_performance(&state.pool, portfolio_id)
        .await
        .map_err(|e| {
            error!("Failed to fetch portfolio true performance: {}", e);
            AppError::Db(e)
        })?;

    Ok(Json(performance))
}
