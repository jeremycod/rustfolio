use bigdecimal::BigDecimal;
use std::str::FromStr;
use crate::models::watchlist::*;
use sqlx::{PgPool, Postgres, QueryBuilder};
use uuid::Uuid;

// ==============================================================================
// Watchlist CRUD Operations
// ==============================================================================

pub async fn create_watchlist(
    pool: &PgPool,
    user_id: Uuid,
    name: &str,
    description: Option<&str>,
    is_default: bool,
) -> Result<Watchlist, sqlx::Error> {
    // If setting as default, unset any existing default first
    if is_default {
        sqlx::query(
            "UPDATE watchlists SET is_default = FALSE WHERE user_id = $1 AND is_default = TRUE"
        )
        .bind(user_id)
        .execute(pool)
        .await?;
    }

    let watchlist = sqlx::query_as::<_, Watchlist>(
        r#"
        INSERT INTO watchlists (user_id, name, description, is_default)
        VALUES ($1, $2, $3, $4)
        RETURNING *
        "#,
    )
    .bind(user_id)
    .bind(name)
    .bind(description)
    .bind(is_default)
    .fetch_one(pool)
    .await?;

    Ok(watchlist)
}

pub async fn get_watchlist(pool: &PgPool, watchlist_id: Uuid) -> Result<Watchlist, sqlx::Error> {
    sqlx::query_as::<_, Watchlist>(
        "SELECT * FROM watchlists WHERE id = $1",
    )
    .bind(watchlist_id)
    .fetch_one(pool)
    .await
}

pub async fn get_watchlists_for_user(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Vec<Watchlist>, sqlx::Error> {
    sqlx::query_as::<_, Watchlist>(
        r#"
        SELECT * FROM watchlists
        WHERE user_id = $1
        ORDER BY is_default DESC, name ASC
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
}

pub async fn update_watchlist(
    pool: &PgPool,
    watchlist_id: Uuid,
    user_id: Uuid,
    name: Option<&str>,
    description: Option<&str>,
    is_default: Option<bool>,
) -> Result<Watchlist, sqlx::Error> {
    // If setting as default, unset others
    if is_default == Some(true) {
        sqlx::query(
            "UPDATE watchlists SET is_default = FALSE WHERE user_id = $1 AND is_default = TRUE AND id != $2"
        )
        .bind(user_id)
        .bind(watchlist_id)
        .execute(pool)
        .await?;
    }

    let mut query_builder: QueryBuilder<Postgres> =
        QueryBuilder::new("UPDATE watchlists SET ");

    let mut separated = query_builder.separated(", ");
    let mut has_updates = false;

    if let Some(name) = name {
        separated.push("name = ");
        separated.push_bind_unseparated(name);
        has_updates = true;
    }

    if let Some(description) = description {
        separated.push("description = ");
        separated.push_bind_unseparated(description);
        has_updates = true;
    }

    if let Some(is_default) = is_default {
        separated.push("is_default = ");
        separated.push_bind_unseparated(is_default);
        has_updates = true;
    }

    if !has_updates {
        return get_watchlist(pool, watchlist_id).await;
    }

    query_builder.push(", updated_at = NOW() WHERE id = ");
    query_builder.push_bind(watchlist_id);
    query_builder.push(" AND user_id = ");
    query_builder.push_bind(user_id);
    query_builder.push(" RETURNING *");

    query_builder
        .build_query_as::<Watchlist>()
        .fetch_one(pool)
        .await
}

pub async fn delete_watchlist(
    pool: &PgPool,
    watchlist_id: Uuid,
    user_id: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM watchlists WHERE id = $1 AND user_id = $2")
        .bind(watchlist_id)
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(())
}

// ==============================================================================
// Watchlist Item Count
// ==============================================================================

pub async fn count_watchlist_items(
    pool: &PgPool,
    watchlist_id: Uuid,
) -> Result<i64, sqlx::Error> {
    let (count,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM watchlist_items WHERE watchlist_id = $1",
    )
    .bind(watchlist_id)
    .fetch_one(pool)
    .await?;
    Ok(count)
}

// ==============================================================================
// Watchlist Item Operations
// ==============================================================================

pub async fn add_watchlist_item(
    pool: &PgPool,
    watchlist_id: Uuid,
    ticker: &str,
    notes: Option<&str>,
    added_price: Option<BigDecimal>,
    target_price: Option<BigDecimal>,
) -> Result<WatchlistItem, sqlx::Error> {
    // Get next sort order
    let (max_order,): (Option<i32>,) = sqlx::query_as(
        "SELECT MAX(sort_order) FROM watchlist_items WHERE watchlist_id = $1",
    )
    .bind(watchlist_id)
    .fetch_one(pool)
    .await?;

    let sort_order = max_order.unwrap_or(-1) + 1;

    sqlx::query_as::<_, WatchlistItem>(
        r#"
        INSERT INTO watchlist_items (watchlist_id, ticker, notes, added_price, target_price, sort_order)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING *
        "#,
    )
    .bind(watchlist_id)
    .bind(ticker)
    .bind(notes)
    .bind(added_price)
    .bind(target_price)
    .bind(sort_order)
    .fetch_one(pool)
    .await
}

pub async fn get_watchlist_items(
    pool: &PgPool,
    watchlist_id: Uuid,
) -> Result<Vec<WatchlistItem>, sqlx::Error> {
    sqlx::query_as::<_, WatchlistItem>(
        r#"
        SELECT * FROM watchlist_items
        WHERE watchlist_id = $1
        ORDER BY sort_order ASC, created_at ASC
        "#,
    )
    .bind(watchlist_id)
    .fetch_all(pool)
    .await
}

pub async fn get_watchlist_item(
    pool: &PgPool,
    item_id: Uuid,
) -> Result<WatchlistItem, sqlx::Error> {
    sqlx::query_as::<_, WatchlistItem>(
        "SELECT * FROM watchlist_items WHERE id = $1",
    )
    .bind(item_id)
    .fetch_one(pool)
    .await
}

pub async fn update_watchlist_item(
    pool: &PgPool,
    item_id: Uuid,
    notes: Option<&str>,
    target_price: Option<BigDecimal>,
    sort_order: Option<i32>,
) -> Result<WatchlistItem, sqlx::Error> {
    let mut query_builder: QueryBuilder<Postgres> =
        QueryBuilder::new("UPDATE watchlist_items SET ");

    let mut separated = query_builder.separated(", ");
    let mut has_updates = false;

    if let Some(notes) = notes {
        separated.push("notes = ");
        separated.push_bind_unseparated(notes);
        has_updates = true;
    }

    if let Some(target_price) = target_price {
        separated.push("target_price = ");
        separated.push_bind_unseparated(target_price);
        has_updates = true;
    }

    if let Some(sort_order) = sort_order {
        separated.push("sort_order = ");
        separated.push_bind_unseparated(sort_order);
        has_updates = true;
    }

    if !has_updates {
        return get_watchlist_item(pool, item_id).await;
    }

    query_builder.push(", updated_at = NOW() WHERE id = ");
    query_builder.push_bind(item_id);
    query_builder.push(" RETURNING *");

    query_builder
        .build_query_as::<WatchlistItem>()
        .fetch_one(pool)
        .await
}

pub async fn remove_watchlist_item(
    pool: &PgPool,
    item_id: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM watchlist_items WHERE id = $1")
        .bind(item_id)
        .execute(pool)
        .await?;
    Ok(())
}

#[allow(dead_code)]
pub async fn remove_watchlist_item_by_ticker(
    pool: &PgPool,
    watchlist_id: Uuid,
    ticker: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM watchlist_items WHERE watchlist_id = $1 AND ticker = $2")
        .bind(watchlist_id)
        .bind(ticker)
        .execute(pool)
        .await?;
    Ok(())
}

// ==============================================================================
// Threshold Operations
// ==============================================================================

pub async fn set_threshold(
    pool: &PgPool,
    watchlist_item_id: Uuid,
    threshold_type: &str,
    comparison: &str,
    value: f64,
    enabled: bool,
) -> Result<WatchlistThreshold, sqlx::Error> {
    tracing::info!("📝 DB: set_threshold - item_id={}, type={}, comparison={}, value={}, enabled={}",
        watchlist_item_id, threshold_type, comparison, value, enabled);

    // Convert f64 to BigDecimal for database storage
    let value_bd = BigDecimal::from_str(&value.to_string())
        .unwrap_or_else(|_| BigDecimal::from(0));

    let result = sqlx::query_as::<_, WatchlistThreshold>(
        r#"
        INSERT INTO watchlist_thresholds (watchlist_item_id, threshold_type, comparison, value, enabled)
        VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT (watchlist_item_id, threshold_type)
        DO UPDATE SET comparison = $3, value = $4, enabled = $5, updated_at = NOW()
        RETURNING *
        "#,
    )
    .bind(watchlist_item_id)
    .bind(threshold_type)
    .bind(comparison)
    .bind(value_bd)
    .bind(enabled)
    .fetch_one(pool)
    .await;

    match &result {
        Ok(threshold) => {
            tracing::info!("✅ DB: Threshold saved successfully with id={}", threshold.id);
        }
        Err(e) => {
            tracing::error!("❌ DB: Failed to save threshold: {:?}", e);
        }
    }

    result
}

pub async fn get_thresholds_for_item(
    pool: &PgPool,
    watchlist_item_id: Uuid,
) -> Result<Vec<WatchlistThreshold>, sqlx::Error> {
    sqlx::query_as::<_, WatchlistThreshold>(
        r#"
        SELECT * FROM watchlist_thresholds
        WHERE watchlist_item_id = $1
        ORDER BY threshold_type ASC
        "#,
    )
    .bind(watchlist_item_id)
    .fetch_all(pool)
    .await
}

pub async fn get_thresholds_for_items(
    pool: &PgPool,
    watchlist_item_ids: &[Uuid],
) -> Result<std::collections::HashMap<Uuid, Vec<WatchlistThreshold>>, sqlx::Error> {
    if watchlist_item_ids.is_empty() {
        return Ok(std::collections::HashMap::new());
    }

    let thresholds = sqlx::query_as::<_, WatchlistThreshold>(
        r#"
        SELECT * FROM watchlist_thresholds
        WHERE watchlist_item_id = ANY($1)
        ORDER BY watchlist_item_id, threshold_type ASC
        "#,
    )
    .bind(watchlist_item_ids)
    .fetch_all(pool)
    .await?;

    // Group by item_id
    let mut map: std::collections::HashMap<Uuid, Vec<WatchlistThreshold>> = std::collections::HashMap::new();
    for threshold in thresholds {
        map.entry(threshold.watchlist_item_id)
            .or_insert_with(Vec::new)
            .push(threshold);
    }

    Ok(map)
}

pub async fn delete_threshold(
    pool: &PgPool,
    threshold_id: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM watchlist_thresholds WHERE id = $1")
        .bind(threshold_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn delete_all_thresholds_for_item(
    pool: &PgPool,
    watchlist_item_id: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM watchlist_thresholds WHERE watchlist_item_id = $1")
        .bind(watchlist_item_id)
        .execute(pool)
        .await?;
    Ok(())
}

#[allow(dead_code)]
pub async fn get_all_enabled_thresholds(
    pool: &PgPool,
) -> Result<Vec<(WatchlistItem, WatchlistThreshold, Uuid)>, sqlx::Error> {
    #[derive(sqlx::FromRow)]
    struct ThresholdRow {
        // watchlist_item fields
        item_id: Uuid,
        watchlist_id: Uuid,
        ticker: String,
        notes: Option<String>,
        added_price: Option<BigDecimal>,
        target_price: Option<BigDecimal>,
        sort_order: i32,
        item_created_at: chrono::DateTime<chrono::Utc>,
        item_updated_at: chrono::DateTime<chrono::Utc>,
        // threshold fields
        threshold_id: Uuid,
        threshold_type: String,
        comparison: String,
        value: BigDecimal,
        enabled: bool,
        threshold_created_at: chrono::DateTime<chrono::Utc>,
        threshold_updated_at: chrono::DateTime<chrono::Utc>,
        // user
        user_id: Uuid,
    }

    let rows = sqlx::query_as::<_, ThresholdRow>(
        r#"
        SELECT
            wi.id AS item_id, wi.watchlist_id, wi.ticker, wi.notes,
            wi.added_price, wi.target_price, wi.sort_order,
            wi.created_at AS item_created_at, wi.updated_at AS item_updated_at,
            wt.id AS threshold_id, wt.threshold_type, wt.comparison, wt.value,
            wt.enabled, wt.created_at AS threshold_created_at,
            wt.updated_at AS threshold_updated_at,
            w.user_id
        FROM watchlist_items wi
        JOIN watchlist_thresholds wt ON wt.watchlist_item_id = wi.id
        JOIN watchlists w ON w.id = wi.watchlist_id
        WHERE wt.enabled = TRUE
        ORDER BY wi.ticker ASC
        "#,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| {
            let item = WatchlistItem {
                id: r.item_id,
                watchlist_id: r.watchlist_id,
                ticker: r.ticker,
                notes: r.notes,
                added_price: r.added_price,
                target_price: r.target_price,
                sort_order: r.sort_order,
                created_at: r.item_created_at,
                updated_at: r.item_updated_at,
            };
            let threshold = WatchlistThreshold {
                id: r.threshold_id,
                watchlist_item_id: r.item_id,
                threshold_type: r.threshold_type,
                comparison: r.comparison,
                value: r.value,
                enabled: r.enabled,
                created_at: r.threshold_created_at,
                updated_at: r.threshold_updated_at,
            };
            (item, threshold, r.user_id)
        })
        .collect())
}

// ==============================================================================
// Alert Operations
// ==============================================================================

pub async fn create_watchlist_alert(
    pool: &PgPool,
    watchlist_item_id: Uuid,
    user_id: Uuid,
    ticker: &str,
    alert_type: &str,
    severity: &str,
    message: &str,
    actual_value: Option<f64>,
    threshold_value: Option<f64>,
    metadata: serde_json::Value,
) -> Result<WatchlistAlert, sqlx::Error> {
    sqlx::query_as::<_, WatchlistAlert>(
        r#"
        INSERT INTO watchlist_alerts (
            watchlist_item_id, user_id, ticker, alert_type, severity,
            message, actual_value, threshold_value, metadata
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        RETURNING *
        "#,
    )
    .bind(watchlist_item_id)
    .bind(user_id)
    .bind(ticker)
    .bind(alert_type)
    .bind(severity)
    .bind(message)
    .bind(actual_value)
    .bind(threshold_value)
    .bind(metadata)
    .fetch_one(pool)
    .await
}

pub async fn get_watchlist_alerts(
    pool: &PgPool,
    user_id: Uuid,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<Vec<WatchlistAlert>, sqlx::Error> {
    let limit = limit.unwrap_or(50).min(200);
    let offset = offset.unwrap_or(0);

    sqlx::query_as::<_, WatchlistAlert>(
        r#"
        SELECT * FROM watchlist_alerts
        WHERE user_id = $1
        ORDER BY created_at DESC
        LIMIT $2 OFFSET $3
        "#,
    )
    .bind(user_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await
}

pub async fn get_alerts_for_watchlist(
    pool: &PgPool,
    watchlist_id: Uuid,
    limit: Option<i64>,
) -> Result<Vec<WatchlistAlert>, sqlx::Error> {
    let limit = limit.unwrap_or(20).min(100);

    sqlx::query_as::<_, WatchlistAlert>(
        r#"
        SELECT wa.* FROM watchlist_alerts wa
        JOIN watchlist_items wi ON wi.id = wa.watchlist_item_id
        WHERE wi.watchlist_id = $1
        ORDER BY wa.created_at DESC
        LIMIT $2
        "#,
    )
    .bind(watchlist_id)
    .bind(limit)
    .fetch_all(pool)
    .await
}

#[allow(dead_code)]
pub async fn get_unread_alert_count(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<i64, sqlx::Error> {
    let (count,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM watchlist_alerts WHERE user_id = $1 AND read = FALSE",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;
    Ok(count)
}

pub async fn mark_alert_read(pool: &PgPool, alert_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE watchlist_alerts SET read = TRUE, read_at = NOW() WHERE id = $1",
    )
    .bind(alert_id)
    .execute(pool)
    .await?;
    Ok(())
}

#[allow(dead_code)]
pub async fn mark_all_alerts_read(pool: &PgPool, user_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE watchlist_alerts SET read = TRUE, read_at = NOW() WHERE user_id = $1 AND read = FALSE",
    )
    .bind(user_id)
    .execute(pool)
    .await?;
    Ok(())
}

#[allow(dead_code)]
pub async fn delete_alert(pool: &PgPool, alert_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM watchlist_alerts WHERE id = $1")
        .bind(alert_id)
        .execute(pool)
        .await?;
    Ok(())
}

// ==============================================================================
// Monitoring State Operations
// ==============================================================================

pub async fn upsert_monitoring_state(
    pool: &PgPool,
    watchlist_item_id: Uuid,
    last_price: Option<f64>,
    last_rsi: Option<f64>,
    last_volume_ratio: Option<f64>,
    last_volatility: Option<f64>,
    last_sentiment_score: Option<f64>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO watchlist_monitoring_state (
            watchlist_item_id, last_price, last_rsi, last_volume_ratio,
            last_volatility, last_sentiment_score, last_checked_at, updated_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, NOW(), NOW())
        ON CONFLICT (watchlist_item_id)
        DO UPDATE SET
            last_price = COALESCE($2, watchlist_monitoring_state.last_price),
            last_rsi = COALESCE($3, watchlist_monitoring_state.last_rsi),
            last_volume_ratio = COALESCE($4, watchlist_monitoring_state.last_volume_ratio),
            last_volatility = COALESCE($5, watchlist_monitoring_state.last_volatility),
            last_sentiment_score = COALESCE($6, watchlist_monitoring_state.last_sentiment_score),
            last_checked_at = NOW(),
            updated_at = NOW()
        "#,
    )
    .bind(watchlist_item_id)
    .bind(last_price)
    .bind(last_rsi)
    .bind(last_volume_ratio)
    .bind(last_volatility)
    .bind(last_sentiment_score)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_monitoring_state(
    pool: &PgPool,
    watchlist_item_id: Uuid,
) -> Result<Option<WatchlistMonitoringState>, sqlx::Error> {
    sqlx::query_as::<_, WatchlistMonitoringState>(
        "SELECT * FROM watchlist_monitoring_state WHERE watchlist_item_id = $1",
    )
    .bind(watchlist_item_id)
    .fetch_optional(pool)
    .await
}

// ==============================================================================
// Bulk Operations (for monitoring job)
// ==============================================================================

pub async fn get_all_watchlist_tickers(pool: &PgPool) -> Result<Vec<String>, sqlx::Error> {
    let tickers: Vec<(String,)> = sqlx::query_as(
        "SELECT DISTINCT ticker FROM watchlist_items ORDER BY ticker",
    )
    .fetch_all(pool)
    .await?;

    Ok(tickers.into_iter().map(|(t,)| t).collect())
}

pub async fn get_all_items_for_ticker(
    pool: &PgPool,
    ticker: &str,
) -> Result<Vec<(WatchlistItem, Uuid)>, sqlx::Error> {
    #[derive(sqlx::FromRow)]
    struct ItemWithUser {
        id: Uuid,
        watchlist_id: Uuid,
        ticker: String,
        notes: Option<String>,
        added_price: Option<BigDecimal>,
        target_price: Option<BigDecimal>,
        sort_order: i32,
        created_at: chrono::DateTime<chrono::Utc>,
        updated_at: chrono::DateTime<chrono::Utc>,
        user_id: Uuid,
    }

    let rows = sqlx::query_as::<_, ItemWithUser>(
        r#"
        SELECT wi.*, w.user_id
        FROM watchlist_items wi
        JOIN watchlists w ON w.id = wi.watchlist_id
        WHERE wi.ticker = $1
        "#,
    )
    .bind(ticker)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| {
            let item = WatchlistItem {
                id: r.id,
                watchlist_id: r.watchlist_id,
                ticker: r.ticker,
                notes: r.notes,
                added_price: r.added_price,
                target_price: r.target_price,
                sort_order: r.sort_order,
                created_at: r.created_at,
                updated_at: r.updated_at,
            };
            (item, r.user_id)
        })
        .collect())
}

/// Check if a recent alert already exists to avoid duplicates
pub async fn has_recent_alert(
    pool: &PgPool,
    watchlist_item_id: Uuid,
    alert_type: &str,
    cooldown_hours: i32,
) -> Result<bool, sqlx::Error> {
    let (count,): (i64,) = sqlx::query_as(
        r#"
        SELECT COUNT(*) FROM watchlist_alerts
        WHERE watchlist_item_id = $1
          AND alert_type = $2
          AND created_at > NOW() - ($3 || ' hours')::INTERVAL
        "#,
    )
    .bind(watchlist_item_id)
    .bind(alert_type)
    .bind(cooldown_hours)
    .fetch_one(pool)
    .await?;

    Ok(count > 0)
}
