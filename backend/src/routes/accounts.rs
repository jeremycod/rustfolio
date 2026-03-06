use axum::extract::{Path, State};
use axum::{Json, Router};
use axum::routing::{get, post};
use bigdecimal::BigDecimal;
use serde::Deserialize;
use std::str::FromStr;
use tracing::{info, error};
use uuid::Uuid;

use crate::db::{account_queries, holding_snapshot_queries, portfolio_queries};
use crate::errors::AppError;
use crate::middleware::auth::AuthUser;
use crate::models::{Account, AccountValueHistory, CreateAccount, CreateHoldingSnapshot, HoldingSnapshot, LatestAccountHolding};
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/portfolios/:portfolio_id/accounts", get(list_accounts).post(create_account))
        .route("/accounts/:account_id", get(get_account))
        .route("/accounts/:account_id/holdings", get(get_latest_holdings).post(add_holding))
        .route("/accounts/:account_id/history", get(get_account_history))
        .route("/portfolios/:portfolio_id/history", get(get_portfolio_history))
}

#[derive(Deserialize)]
pub struct CreateAccountRequest {
    pub account_number: String,
    pub account_nickname: String,
    pub client_id: Option<String>,
    pub client_name: Option<String>,
}

#[derive(Deserialize)]
pub struct AddHoldingRequest {
    pub ticker: String,
    pub holding_name: Option<String>,
    pub asset_category: Option<String>,
    pub industry: Option<String>,
    pub quantity: f64,
    pub price: f64,
    pub average_cost: f64,
    pub snapshot_date: Option<String>,
}

fn to_decimal(v: f64) -> BigDecimal {
    BigDecimal::from_str(&format!("{:.8}", v)).unwrap_or_else(|_| BigDecimal::from(0))
}

pub async fn list_accounts(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Path(portfolio_id): Path<Uuid>,
) -> Result<Json<Vec<Account>>, AppError> {
    info!("GET /portfolios/{}/accounts - Fetching all accounts", portfolio_id);
    portfolio_queries::fetch_one(&state.pool, portfolio_id, user_id)
        .await
        .map_err(AppError::Db)?
        .ok_or_else(|| AppError::NotFound(format!("Portfolio {} not found", portfolio_id)))?;
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
    AuthUser(user_id): AuthUser,
    Path(account_id): Path<Uuid>,
) -> Result<Json<Account>, AppError> {
    info!("GET /accounts/{} - Fetching account", account_id);
    if !account_queries::belongs_to_user(&state.pool, account_id, user_id)
        .await
        .map_err(AppError::Db)?
    {
        return Err(AppError::NotFound(format!("Account {} not found", account_id)));
    }
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
    AuthUser(user_id): AuthUser,
    Path(account_id): Path<Uuid>,
) -> Result<Json<Vec<LatestAccountHolding>>, AppError> {
    info!("GET /accounts/{}/holdings - Fetching latest holdings", account_id);
    if !account_queries::belongs_to_user(&state.pool, account_id, user_id)
        .await
        .map_err(AppError::Db)?
    {
        return Err(AppError::NotFound(format!("Account {} not found", account_id)));
    }
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
    AuthUser(user_id): AuthUser,
    Path(account_id): Path<Uuid>,
) -> Result<Json<Vec<AccountValueHistory>>, AppError> {
    info!("GET /accounts/{}/history - Fetching account value history", account_id);
    if !account_queries::belongs_to_user(&state.pool, account_id, user_id)
        .await
        .map_err(AppError::Db)?
    {
        return Err(AppError::NotFound(format!("Account {} not found", account_id)));
    }
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
    AuthUser(user_id): AuthUser,
    Path(portfolio_id): Path<Uuid>,
) -> Result<Json<Vec<AccountValueHistory>>, AppError> {
    info!("GET /portfolios/{}/history - Fetching portfolio value history", portfolio_id);
    portfolio_queries::fetch_one(&state.pool, portfolio_id, user_id)
        .await
        .map_err(AppError::Db)?
        .ok_or_else(|| AppError::NotFound(format!("Portfolio {} not found", portfolio_id)))?;
    let history = holding_snapshot_queries::fetch_portfolio_value_history(&state.pool, portfolio_id)
        .await
        .map_err(|e| {
            error!("Failed to fetch portfolio value history for portfolio {}: {:?}", portfolio_id, e);
            AppError::Db(e)
        })?;
    info!("Successfully fetched {} history records for portfolio {}", history.len(), portfolio_id);
    Ok(Json(history))
}

pub async fn create_account(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Path(portfolio_id): Path<Uuid>,
    Json(body): Json<CreateAccountRequest>,
) -> Result<Json<Account>, AppError> {
    info!("POST /portfolios/{}/accounts - Creating account manually", portfolio_id);
    portfolio_queries::fetch_one(&state.pool, portfolio_id, user_id)
        .await
        .map_err(AppError::Db)?
        .ok_or_else(|| AppError::NotFound(format!("Portfolio {} not found", portfolio_id)))?;
    let account = account_queries::create(&state.pool, portfolio_id, CreateAccount {
        account_number: body.account_number,
        account_nickname: body.account_nickname,
        client_id: body.client_id,
        client_name: body.client_name,
    })
    .await
    .map_err(|e| {
        error!("Failed to create account for portfolio {}: {}", portfolio_id, e);
        AppError::Db(e)
    })?;
    Ok(Json(account))
}

pub async fn add_holding(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Path(account_id): Path<Uuid>,
    Json(body): Json<AddHoldingRequest>,
) -> Result<Json<HoldingSnapshot>, AppError> {
    info!("POST /accounts/{}/holdings - Adding holding manually", account_id);
    if !account_queries::belongs_to_user(&state.pool, account_id, user_id)
        .await
        .map_err(AppError::Db)?
    {
        return Err(AppError::NotFound(format!("Account {} not found", account_id)));
    }
    let snapshot_date = if let Some(date_str) = &body.snapshot_date {
        chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
            .map_err(|_| AppError::Validation("Invalid snapshot_date format, expected YYYY-MM-DD".to_string()))?
    } else {
        chrono::Local::now().date_naive()
    };
    let quantity = to_decimal(body.quantity);
    let price = to_decimal(body.price);
    let average_cost = to_decimal(body.average_cost);
    let market_value = &quantity * &price;
    let book_value = &quantity * &average_cost;
    let gain_loss = &market_value - &book_value;
    let gain_loss_pct = if book_value != BigDecimal::from(0) {
        Some((&gain_loss / &book_value) * BigDecimal::from(100))
    } else {
        Some(BigDecimal::from(0))
    };
    let holding = holding_snapshot_queries::upsert(&state.pool, account_id, snapshot_date, CreateHoldingSnapshot {
        ticker: body.ticker,
        holding_name: body.holding_name,
        asset_category: body.asset_category,
        industry: body.industry,
        quantity,
        price,
        average_cost,
        book_value,
        market_value,
        fund: None,
        accrued_interest: None,
        gain_loss: Some(gain_loss),
        gain_loss_pct,
        percentage_of_assets: None,
    })
    .await
    .map_err(|e| {
        error!("Failed to add holding for account {}: {}", account_id, e);
        AppError::Db(e)
    })?;
    Ok(Json(holding))
}
