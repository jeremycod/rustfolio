use axum::extract::{Path, State};
use axum::{Json, Router};
use axum::routing::get;
use tracing::{info, error};
use uuid::Uuid;

use crate::db::{account_queries, holding_snapshot_queries};
use crate::errors::AppError;
use crate::models::{Account, AccountValueHistory, LatestAccountHolding};
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/portfolios/:portfolio_id/accounts", get(list_accounts))
        .route("/accounts/:account_id", get(get_account))
        .route("/accounts/:account_id/holdings", get(get_latest_holdings))
        .route("/accounts/:account_id/history", get(get_account_history))
        .route("/portfolios/:portfolio_id/history", get(get_portfolio_history))
}

pub async fn list_accounts(
    State(state): State<AppState>,
    Path(portfolio_id): Path<Uuid>,
) -> Result<Json<Vec<Account>>, AppError> {
    info!("GET /portfolios/{}/accounts - Fetching all accounts", portfolio_id);
    let accounts = account_queries::fetch_all(&state.pool, portfolio_id)
        .await
        .map_err(|e| {
            error!("Failed to fetch accounts for portfolio {}: {}", portfolio_id, e);
            AppError::Db(e)
        })?;
    Ok(Json(accounts))
}

pub async fn get_account(
    State(state): State<AppState>,
    Path(account_id): Path<Uuid>,
) -> Result<Json<Account>, AppError> {
    info!("GET /accounts/{} - Fetching account", account_id);
    let account = account_queries::fetch_one(&state.pool, account_id)
        .await
        .map_err(|e| {
            error!("Failed to fetch account {}: {}", account_id, e);
            AppError::Db(e)
        })?
        .ok_or_else(|| {
            error!("Account {} not found", account_id);
            AppError::NotFound(format!("Account {} not found", account_id))
        })?;
    Ok(Json(account))
}

pub async fn get_latest_holdings(
    State(state): State<AppState>,
    Path(account_id): Path<Uuid>,
) -> Result<Json<Vec<LatestAccountHolding>>, AppError> {
    info!("GET /accounts/{}/holdings - Fetching latest holdings", account_id);
    let holdings = holding_snapshot_queries::fetch_latest_holdings(&state.pool, account_id)
        .await
        .map_err(|e| {
            error!("Failed to fetch latest holdings for account {}: {}", account_id, e);
            AppError::Db(e)
        })?;
    Ok(Json(holdings))
}

pub async fn get_account_history(
    State(state): State<AppState>,
    Path(account_id): Path<Uuid>,
) -> Result<Json<Vec<AccountValueHistory>>, AppError> {
    info!("GET /accounts/{}/history - Fetching account value history", account_id);
    let history = holding_snapshot_queries::fetch_account_value_history(&state.pool, account_id)
        .await
        .map_err(|e| {
            error!("Failed to fetch account value history for account {}: {}", account_id, e);
            AppError::Db(e)
        })?;
    Ok(Json(history))
}

pub async fn get_portfolio_history(
    State(state): State<AppState>,
    Path(portfolio_id): Path<Uuid>,
) -> Result<Json<Vec<AccountValueHistory>>, AppError> {
    info!("GET /portfolios/{}/history - Fetching portfolio value history", portfolio_id);
    let history = holding_snapshot_queries::fetch_portfolio_value_history(&state.pool, portfolio_id)
        .await
        .map_err(|e| {
            error!("Failed to fetch portfolio value history for portfolio {}: {:?}", portfolio_id, e);
            AppError::Db(e)
        })?;
    info!("Successfully fetched {} history records for portfolio {}", history.len(), portfolio_id);
    Ok(Json(history))
}
