use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post, put},
    Json, Router,
};
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::db::{alert_queries, watchlist_queries, price_queries};
use crate::models::watchlist::*;
use crate::state::AppState;

// ==============================================================================
// Router - 13 endpoints
// ==============================================================================

pub fn router() -> Router<AppState> {
    Router::new()
        // Watchlist CRUD
        .route("/watchlists", post(create_watchlist))
        .route("/watchlists", get(list_watchlists))
        .route("/watchlists/:id", get(get_watchlist))
        .route("/watchlists/:id", put(update_watchlist))
        .route("/watchlists/:id", delete(delete_watchlist))
        // Watchlist Items
        .route("/watchlists/:id/items", post(add_item))
        .route("/watchlists/:id/items", get(get_items))
        .route("/watchlists/:watchlist_id/items/:item_id", put(update_item))
        .route("/watchlists/:watchlist_id/items/:item_id", delete(remove_item))
        // Thresholds
        .route("/watchlists/items/:item_id/thresholds", post(set_threshold))
        .route("/watchlists/items/:item_id/thresholds/:threshold_id", delete(delete_threshold))
        // Alerts
        .route("/watchlists/alerts", get(get_alerts))
        .route("/watchlists/alerts/:alert_id/read", post(mark_alert_read))
}

// ==============================================================================
// Query Parameters
// ==============================================================================

#[derive(Debug, Deserialize)]
struct PaginationParams {
    limit: Option<i64>,
    offset: Option<i64>,
}

// ==============================================================================
// Watchlist CRUD Handlers
// ==============================================================================

async fn create_watchlist(
    State(state): State<AppState>,
    Json(req): Json<CreateWatchlistRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let pool = &state.pool;
    let user_id = get_default_user_id(pool).await?;

    let watchlist = watchlist_queries::create_watchlist(
        pool,
        user_id,
        &req.name,
        req.description.as_deref(),
        req.is_default.unwrap_or(false),
    )
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let count = watchlist_queries::count_watchlist_items(pool, watchlist.id)
        .await
        .unwrap_or(0);

    let response = WatchlistResponse {
        id: watchlist.id,
        name: watchlist.name,
        description: watchlist.description,
        is_default: watchlist.is_default,
        item_count: count,
        created_at: watchlist.created_at,
        updated_at: watchlist.updated_at,
    };

    Ok((StatusCode::CREATED, Json(response)))
}

async fn list_watchlists(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let pool = &state.pool;
    let user_id = get_default_user_id(pool).await?;

    let watchlists = watchlist_queries::get_watchlists_for_user(pool, user_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let mut responses = Vec::new();
    for w in watchlists {
        let count = watchlist_queries::count_watchlist_items(pool, w.id)
            .await
            .unwrap_or(0);

        responses.push(WatchlistResponse {
            id: w.id,
            name: w.name,
            description: w.description,
            is_default: w.is_default,
            item_count: count,
            created_at: w.created_at,
            updated_at: w.updated_at,
        });
    }

    Ok(Json(responses))
}

async fn get_watchlist(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let pool = &state.pool;

    let watchlist = watchlist_queries::get_watchlist(pool, id)
        .await
        .map_err(|e| (StatusCode::NOT_FOUND, e.to_string()))?;

    let items = watchlist_queries::get_watchlist_items(pool, id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let mut item_responses = Vec::new();
    for item in items {
        let thresholds = watchlist_queries::get_thresholds_for_item(pool, item.id)
            .await
            .unwrap_or_default();

        // Get current price for this ticker
        let (current_price, price_change_pct) = get_current_price_data(pool, &item).await;

        item_responses.push(WatchlistItemResponse {
            id: item.id,
            watchlist_id: item.watchlist_id,
            ticker: item.ticker,
            notes: item.notes,
            added_price: item.added_price.and_then(|p| p.to_string().parse().ok()),
            target_price: item.target_price.and_then(|p| p.to_string().parse().ok()),
            current_price,
            price_change_pct,
            sort_order: item.sort_order,
            thresholds: thresholds.into_iter().map(WatchlistThresholdResponse::from).collect(),
            created_at: item.created_at,
            updated_at: item.updated_at,
        });
    }

    let recent_alerts = watchlist_queries::get_alerts_for_watchlist(pool, id, Some(10))
        .await
        .unwrap_or_default()
        .into_iter()
        .map(WatchlistAlertResponse::from)
        .collect();

    let response = WatchlistDetailResponse {
        id: watchlist.id,
        name: watchlist.name,
        description: watchlist.description,
        is_default: watchlist.is_default,
        items: item_responses,
        recent_alerts,
        created_at: watchlist.created_at,
        updated_at: watchlist.updated_at,
    };

    Ok(Json(response))
}

async fn update_watchlist(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateWatchlistRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let pool = &state.pool;
    let user_id = get_default_user_id(pool).await?;

    let watchlist = watchlist_queries::update_watchlist(
        pool,
        id,
        user_id,
        req.name.as_deref(),
        req.description.as_deref(),
        req.is_default,
    )
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let count = watchlist_queries::count_watchlist_items(pool, watchlist.id)
        .await
        .unwrap_or(0);

    let response = WatchlistResponse {
        id: watchlist.id,
        name: watchlist.name,
        description: watchlist.description,
        is_default: watchlist.is_default,
        item_count: count,
        created_at: watchlist.created_at,
        updated_at: watchlist.updated_at,
    };

    Ok(Json(response))
}

async fn delete_watchlist(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let pool = &state.pool;
    let user_id = get_default_user_id(pool).await?;

    watchlist_queries::delete_watchlist(pool, id, user_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(StatusCode::NO_CONTENT)
}

// ==============================================================================
// Watchlist Item Handlers
// ==============================================================================

async fn add_item(
    State(state): State<AppState>,
    Path(watchlist_id): Path<Uuid>,
    Json(req): Json<AddWatchlistItemRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let pool = &state.pool;

    // Verify watchlist exists
    watchlist_queries::get_watchlist(pool, watchlist_id)
        .await
        .map_err(|e| (StatusCode::NOT_FOUND, format!("Watchlist not found: {}", e)))?;

    // Get current price to store as added_price
    let added_price = match price_queries::fetch_latest(pool, &req.ticker).await {
        Ok(Some(pp)) => Some(pp.close_price),
        _ => None,
    };

    let target_price_bd = req.target_price.and_then(|p| p.to_string().parse().ok()); // Convert f64 to BigDecimal

    let item = watchlist_queries::add_watchlist_item(
        pool,
        watchlist_id,
        &req.ticker.to_uppercase(),
        req.notes.as_deref(),
        added_price,
        target_price_bd,
    )
    .await
    .map_err(|e| {
        if e.to_string().contains("duplicate key") {
            (StatusCode::CONFLICT, format!("{} is already in this watchlist", req.ticker))
        } else {
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        }
    })?;

    let (current_price, price_change_pct) = get_current_price_data(pool, &item).await;

    let response = WatchlistItemResponse {
        id: item.id,
        watchlist_id: item.watchlist_id,
        ticker: item.ticker,
        notes: item.notes,
        added_price: item.added_price.and_then(|p| p.to_string().parse().ok()),
        target_price: item.target_price.and_then(|p| p.to_string().parse().ok()),
        current_price,
        price_change_pct,
        sort_order: item.sort_order,
        thresholds: Vec::new(),
        created_at: item.created_at,
        updated_at: item.updated_at,
    };

    Ok((StatusCode::CREATED, Json(response)))
}

async fn get_items(
    State(state): State<AppState>,
    Path(watchlist_id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let pool = &state.pool;

    let items = watchlist_queries::get_watchlist_items(pool, watchlist_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let mut responses = Vec::new();
    for item in items {
        let thresholds = watchlist_queries::get_thresholds_for_item(pool, item.id)
            .await
            .unwrap_or_default();

        let (current_price, price_change_pct) = get_current_price_data(pool, &item).await;

        responses.push(WatchlistItemResponse {
            id: item.id,
            watchlist_id: item.watchlist_id,
            ticker: item.ticker,
            notes: item.notes,
            added_price: item.added_price.and_then(|p| p.to_string().parse().ok()),
            target_price: item.target_price.and_then(|p| p.to_string().parse().ok()),
            current_price,
            price_change_pct,
            sort_order: item.sort_order,
            thresholds: thresholds.into_iter().map(WatchlistThresholdResponse::from).collect(),
            created_at: item.created_at,
            updated_at: item.updated_at,
        });
    }

    Ok(Json(responses))
}

async fn update_item(
    State(state): State<AppState>,
    Path((_watchlist_id, item_id)): Path<(Uuid, Uuid)>,
    Json(req): Json<UpdateWatchlistItemRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let pool = &state.pool;

    let target_price_bd = req.target_price.and_then(|p| p.to_string().parse().ok());

    let item = watchlist_queries::update_watchlist_item(
        pool,
        item_id,
        req.notes.as_deref(),
        target_price_bd,
        req.sort_order,
    )
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let thresholds = watchlist_queries::get_thresholds_for_item(pool, item.id)
        .await
        .unwrap_or_default();

    let (current_price, price_change_pct) = get_current_price_data(pool, &item).await;

    let response = WatchlistItemResponse {
        id: item.id,
        watchlist_id: item.watchlist_id,
        ticker: item.ticker,
        notes: item.notes,
        added_price: item.added_price.and_then(|p| p.to_string().parse().ok()),
        target_price: item.target_price.and_then(|p| p.to_string().parse().ok()),
        current_price,
        price_change_pct,
        sort_order: item.sort_order,
        thresholds: thresholds.into_iter().map(WatchlistThresholdResponse::from).collect(),
        created_at: item.created_at,
        updated_at: item.updated_at,
    };

    Ok(Json(response))
}

async fn remove_item(
    State(state): State<AppState>,
    Path((_watchlist_id, item_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let pool = &state.pool;

    watchlist_queries::remove_watchlist_item(pool, item_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(StatusCode::NO_CONTENT)
}

// ==============================================================================
// Threshold Handlers
// ==============================================================================

async fn set_threshold(
    State(state): State<AppState>,
    Path(item_id): Path<Uuid>,
    Json(req): Json<SetThresholdRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let pool = &state.pool;

    // Verify item exists
    watchlist_queries::get_watchlist_item(pool, item_id)
        .await
        .map_err(|e| (StatusCode::NOT_FOUND, format!("Watchlist item not found: {}", e)))?;

    let threshold = watchlist_queries::set_threshold(
        pool,
        item_id,
        req.threshold_type.as_str(),
        req.comparison.as_str(),
        req.value,
        req.enabled.unwrap_or(true),
    )
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok((StatusCode::CREATED, Json(WatchlistThresholdResponse::from(threshold))))
}

async fn delete_threshold(
    State(state): State<AppState>,
    Path((item_id, threshold_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let pool = &state.pool;

    // The item_id in the path ensures we're accessing the right resource
    let _ = item_id;

    watchlist_queries::delete_threshold(pool, threshold_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(StatusCode::NO_CONTENT)
}

// ==============================================================================
// Alert Handlers
// ==============================================================================

async fn get_alerts(
    State(state): State<AppState>,
    Query(params): Query<PaginationParams>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let pool = &state.pool;
    let user_id = get_default_user_id(pool).await?;

    let alerts = watchlist_queries::get_watchlist_alerts(pool, user_id, params.limit, params.offset)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let responses: Vec<WatchlistAlertResponse> =
        alerts.into_iter().map(WatchlistAlertResponse::from).collect();

    Ok(Json(responses))
}

async fn mark_alert_read(
    State(state): State<AppState>,
    Path(alert_id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let pool = &state.pool;

    watchlist_queries::mark_alert_read(pool, alert_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(StatusCode::OK)
}

// ==============================================================================
// Helper Functions
// ==============================================================================

async fn get_default_user_id(pool: &PgPool) -> Result<Uuid, (StatusCode, String)> {
    let default_uuid = Uuid::parse_str("00000000-0000-0000-0000-000000000001")
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    alert_queries::get_user(pool, default_uuid)
        .await
        .map_err(|e| (StatusCode::UNAUTHORIZED, e.to_string()))?;

    Ok(default_uuid)
}

async fn get_current_price_data(pool: &PgPool, item: &WatchlistItem) -> (Option<f64>, Option<f64>) {
    let current_price = match price_queries::fetch_latest(pool, &item.ticker).await {
        Ok(Some(pp)) => Some(pp.close_price.to_string().parse::<f64>().unwrap_or(0.0)),
        _ => None,
    };

    let added_price_f64 = item.added_price.as_ref().and_then(|p| p.to_string().parse::<f64>().ok());

    let price_change_pct = match (current_price, added_price_f64) {
        (Some(current), Some(added)) if added > 0.0 => {
            Some(((current - added) / added) * 100.0)
        }
        _ => None,
    };

    (current_price, price_change_pct)
}
