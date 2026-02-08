use axum::extract::State;
use axum::{Json, Router};
use axum::routing::post;
use serde::Serialize;
use tracing::{info, error};

use crate::errors::AppError;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/admin/reset-all-data", post(reset_all_data))
}

#[derive(Debug, Serialize)]
pub struct ResetResponse {
    pub message: String,
    pub tables_cleared: Vec<String>,
}

pub async fn reset_all_data(
    State(state): State<AppState>,
) -> Result<Json<ResetResponse>, AppError> {
    info!("POST /admin/reset-all-data - Resetting all data");

    let tables = vec![
        "detected_transactions",
        "cash_flows",
        "holdings_snapshots",
        "accounts",
        "transactions",
        "positions",
        "price_points",
        "portfolios",
    ];

    for table in &tables {
        let query = format!("DELETE FROM {}", table);
        match sqlx::query(&query).execute(&state.pool).await {
            Ok(result) => {
                info!("Deleted {} rows from {}", result.rows_affected(), table);
            }
            Err(e) => {
                error!("Failed to delete from {}: {}", table, e);
                return Err(AppError::Db(e));
            }
        }
    }

    info!("Successfully reset all data");

    Ok(Json(ResetResponse {
        message: "All data has been successfully deleted".to_string(),
        tables_cleared: tables.iter().map(|s| s.to_string()).collect(),
    }))
}
