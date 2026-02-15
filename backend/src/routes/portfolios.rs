use axum::extract::{Path, State};
use axum::{Json, Router};
use axum::routing::{delete, get, post, put};
use tracing::{info, error};
use uuid::Uuid;

use crate::services;

use crate::errors::AppError;
use crate::models::{CreatePortfolio, Portfolio, UpdatePortfolio, LatestAccountHolding};
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", post(create_portfolio).get(fetch_portfolios))
        .route("/:id", get(get_portfolio))
        .route("/:id", put(update_portfolio))
        .route("/:id", delete(delete_portfolio))
        .route("/:id/latest-holdings", get(get_portfolio_latest_holdings))
}

#[axum::debug_handler]
pub async fn create_portfolio(
    State(state): State<AppState>,
    Json(data): Json<CreatePortfolio>
) -> Result<Json<Portfolio>, AppError> {
    info!("POST /portfolios - Creating new portfolio");
    let portfolio = services::portfolio_service::create(&state.pool, data).await
        .map_err(|e| {
            error!("Failed to create portfolio: {}", e);
            e
        })?;
    Ok(Json(portfolio))

}

pub async fn fetch_portfolios(
    State(state): State<AppState>
) -> Result<Json<Vec<Portfolio>>, AppError> {
    info!("GET /portfolios - Fetching all portfolios");
    let portfolios = services::portfolio_service::fetch_all(&state.pool).await
        .map_err(|e| {
            error!("Failed to fetch portfolios: {}", e);
            e
        })?;
    Ok(Json(portfolios))
}

pub async fn get_portfolio(
    State(state): State<AppState>,
    Path(id): Path<Uuid>
) -> Result<Json<Portfolio>, AppError> {
    info!("GET /portfolios/{} - Fetching portfolio", id);
    let portfolio = services::portfolio_service::fetch_one(&state.pool, id)
        .await
        .map_err(|e| {
            error!("Failed to fetch portfolio {}: {}", id, e);
            e
        })?;
    Ok(Json(portfolio))
}

pub async fn update_portfolio(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(data): Json<UpdatePortfolio>
) -> Result<Json<Portfolio>, AppError> {
    info!("PUT /portfolios/{} - Updating portfolio", id);
    let portfolio = services::portfolio_service::update(&state.pool, id, data).await
        .map_err(|e| {
            error!("Failed to update portfolio {}: {}", id, e);
            e
        })?;
    Ok(Json(portfolio))
}

pub async fn delete_portfolio(
    State(state): State<AppState>,
    Path(id): Path<Uuid>
) -> Result<Json<()>, AppError> {
    info!("DELETE /portfolios/{} - Deleting portfolio", id);
    match services::portfolio_service::delete(&state.pool, id).await {
        Ok(0) => {
            error!("Portfolio {} not found for deletion", id);
            Err(AppError::NotFound)
        },
        Ok(_) => Ok(Json(())),
        Err(e) => {
            error!("Failed to delete portfolio {}: {}", id, e);
            Err(e)
        }
    }
}

pub async fn get_portfolio_latest_holdings(
    State(state): State<AppState>,
    Path(id): Path<Uuid>
) -> Result<Json<Vec<LatestAccountHolding>>, AppError> {
    use crate::db::holding_snapshot_queries;

    info!("GET /portfolios/{}/latest-holdings - Fetching latest holdings", id);
    let holdings = holding_snapshot_queries::fetch_portfolio_latest_holdings(&state.pool, id)
        .await
        .map_err(|e| {
            error!("Failed to fetch holdings for portfolio {}: {}", id, e);
            AppError::Db(e)
        })?;
    Ok(Json(holdings))
}

