use axum::extract::{Path, State};
use axum::{Json, Router};
use axum::routing::{get, post};
use tracing::{info, error};
use uuid::Uuid;

use crate::db::cash_flow_queries;
use crate::errors::AppError;
use crate::models::{CashFlow, CreateCashFlow};
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/accounts/:account_id/cash-flows", post(create_cash_flow))
        .route("/accounts/:account_id/cash-flows", get(list_cash_flows))
}

pub async fn create_cash_flow(
    State(state): State<AppState>,
    Path(account_id): Path<Uuid>,
    Json(data): Json<CreateCashFlow>,
) -> Result<Json<CashFlow>, AppError> {
    info!("POST /accounts/{}/cash-flows - Creating cash flow", account_id);

    let cash_flow = cash_flow_queries::create(&state.pool, account_id, data)
        .await
        .map_err(|e| {
            error!("Failed to create cash flow: {}", e);
            AppError::Db(e)
        })?;

    // Update account totals
    if let Err(e) = cash_flow_queries::update_account_totals(&state.pool, account_id).await {
        error!("Failed to update account totals: {}", e);
    }

    Ok(Json(cash_flow))
}

pub async fn list_cash_flows(
    State(state): State<AppState>,
    Path(account_id): Path<Uuid>,
) -> Result<Json<Vec<CashFlow>>, AppError> {
    info!("GET /accounts/{}/cash-flows - Listing cash flows", account_id);

    let cash_flows = cash_flow_queries::fetch_by_account(&state.pool, account_id)
        .await
        .map_err(|e| {
            error!("Failed to fetch cash flows: {}", e);
            AppError::Db(e)
        })?;

    Ok(Json(cash_flows))
}
