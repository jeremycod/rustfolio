use sqlx::PgPool;
use chrono::{Utc, Duration, NaiveDateTime};

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct TickerFetchFailure {
    pub ticker: String,
    pub last_attempt_at: NaiveDateTime,
    pub failure_type: String,
    pub retry_after: NaiveDateTime,
    pub consecutive_failures: i32,
    pub error_message: Option<String>,
}

/// Check if a ticker is in the failure cache and if we should retry
pub async fn should_retry_ticker(
    pool: &PgPool,
    ticker: &str,
) -> Result<bool, sqlx::Error> {
    let now = Utc::now().naive_utc();

    let result = sqlx::query_scalar!(
        "SELECT COUNT(*) as count FROM ticker_fetch_failures
         WHERE ticker = $1 AND retry_after > $2",
        ticker,
        now
    )
    .fetch_one(pool)
    .await?;

    // If count is 0 or None, we can retry (no active failure record)
    // If count > 0, we should NOT retry (still in cooldown period)
    Ok(result.unwrap_or(0) == 0)
}

/// Get the failure record for a ticker if it exists and is still active
pub async fn get_active_failure(
    pool: &PgPool,
    ticker: &str,
) -> Result<Option<TickerFetchFailure>, sqlx::Error> {
    let now = Utc::now().naive_utc();

    let result = sqlx::query!(
        "SELECT ticker, last_attempt_at, failure_type, retry_after, consecutive_failures, error_message
         FROM ticker_fetch_failures
         WHERE ticker = $1 AND retry_after > $2",
        ticker,
        now
    )
    .fetch_optional(pool)
    .await?;

    Ok(result.map(|row| TickerFetchFailure {
        ticker: row.ticker,
        last_attempt_at: row.last_attempt_at,
        failure_type: row.failure_type,
        retry_after: row.retry_after,
        consecutive_failures: row.consecutive_failures,
        error_message: row.error_message,
    }))
}

/// Record a failed fetch attempt for a ticker
pub async fn record_fetch_failure(
    pool: &PgPool,
    ticker: &str,
    failure_type: &str, // "not_found", "rate_limited", "api_error"
    error_message: Option<&str>,
) -> Result<(), sqlx::Error> {
    let now = Utc::now();
    let now_naive = now.naive_utc();

    // Calculate retry_after based on failure type
    let retry_after = match failure_type {
        "not_found" => (now + Duration::hours(24)).naive_utc(),      // 24 hours for not found
        "rate_limited" => (now + Duration::hours(1)).naive_utc(),    // 1 hour for rate limits
        "api_error" => (now + Duration::hours(6)).naive_utc(),       // 6 hours for API errors
        _ => (now + Duration::hours(6)).naive_utc(),                 // Default 6 hours
    };

    // Use INSERT ... ON CONFLICT to upsert
    sqlx::query!(
        "INSERT INTO ticker_fetch_failures
         (ticker, last_attempt_at, failure_type, retry_after, consecutive_failures, error_message, updated_at)
         VALUES ($1, $2, $3, $4, 1, $5, $6)
         ON CONFLICT(ticker) DO UPDATE SET
           last_attempt_at = EXCLUDED.last_attempt_at,
           failure_type = EXCLUDED.failure_type,
           retry_after = EXCLUDED.retry_after,
           consecutive_failures = ticker_fetch_failures.consecutive_failures + 1,
           error_message = EXCLUDED.error_message,
           updated_at = EXCLUDED.updated_at",
        ticker,
        now_naive,
        failure_type,
        retry_after,
        error_message,
        now_naive
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// Clear the failure record for a ticker (called after successful fetch)
pub async fn clear_fetch_failure(
    pool: &PgPool,
    ticker: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "DELETE FROM ticker_fetch_failures WHERE ticker = $1",
        ticker
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// Clean up expired failure records (ones past retry_after)
#[allow(dead_code)]
pub async fn cleanup_expired_failures(
    pool: &PgPool,
) -> Result<u64, sqlx::Error> {
    let now = Utc::now().naive_utc();

    let result = sqlx::query!(
        "DELETE FROM ticker_fetch_failures WHERE retry_after <= $1",
        now
    )
    .execute(pool)
    .await?;

    Ok(result.rows_affected())
}

/// Get all active failures (for debugging/monitoring)
#[allow(dead_code)]
pub async fn get_all_active_failures(
    pool: &PgPool,
) -> Result<Vec<TickerFetchFailure>, sqlx::Error> {
    let now = Utc::now().naive_utc();

    let results = sqlx::query!(
        "SELECT ticker, last_attempt_at, failure_type, retry_after, consecutive_failures, error_message
         FROM ticker_fetch_failures
         WHERE retry_after > $1
         ORDER BY retry_after DESC",
        now
    )
    .fetch_all(pool)
    .await?;

    Ok(results.into_iter().map(|row| TickerFetchFailure {
        ticker: row.ticker,
        last_attempt_at: row.last_attempt_at,
        failure_type: row.failure_type,
        retry_after: row.retry_after,
        consecutive_failures: row.consecutive_failures,
        error_message: row.error_message,
    }).collect())
}
