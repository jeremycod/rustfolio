# Price Caching and Failure Tracking System

## Overview

The Rustfolio backend implements a comprehensive caching system for ticker price data to minimize external API calls, respect rate limits, and provide a better user experience. The system consists of two layers:

1. **Price Data Cache** - Stores actual price points in the database
2. **Failure Tracking Cache** - Records and prevents repeated calls for tickers that fail

## Problem Being Solved

**Before:** The system would repeatedly try to fetch price data from external APIs (Twelve Data, Alpha Vantage) for every request, causing:
- Excessive API calls hitting rate limits quickly
- Slow response times waiting for API calls
- Poor UX with repeated failures for invalid tickers
- Waste of API quota on known-bad tickers

**After:** Intelligent caching system that:
- Checks database first for existing recent price data
- Only calls APIs when data is stale (>6 hours old)
- Remembers failed tickers and avoids retrying them
- Provides fast responses using cached data

## Architecture

### 1. Price Data Caching

**Location:** `price_points` table in PostgreSQL

**How it works:**
1. When price data is requested for a ticker:
   - Check if we have data in `price_points` table
   - If data exists and is less than 6 hours old → **Return from cache** (no API call)
   - If data is stale or missing → Fetch from API and update cache

**Database Schema:**
```sql
CREATE TABLE price_points (
    id UUID PRIMARY KEY,
    ticker TEXT NOT NULL,
    date DATE NOT NULL,
    close_price NUMERIC NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(ticker, date)
);
```

**Key Functions:**
- `db::price_queries::fetch_latest(pool, ticker)` - Get most recent price point
- `db::price_queries::upsert_external_points(pool, ticker, points)` - Store fetched prices

### 2. Failure Tracking Cache

**Location:** `ticker_fetch_failures` table in PostgreSQL

**How it works:**
1. Before making any API call, check `ticker_fetch_failures` table
2. If ticker has a recent failure record with `retry_after > now`:
   - **Skip API call** and return error immediately
   - Save API quota and time
3. If API call fails:
   - Record failure with appropriate TTL:
     - **Not Found (404):** 24 hours - ticker likely doesn't exist
     - **Rate Limited (429):** 1 hour - temporary limit
     - **API Error (500):** 6 hours - provider issue
4. If API call succeeds:
   - Clear any existing failure record

**Database Schema:**
```sql
CREATE TABLE ticker_fetch_failures (
    ticker TEXT PRIMARY KEY,
    last_attempt_at TIMESTAMP NOT NULL,
    failure_type TEXT NOT NULL, -- 'not_found', 'rate_limited', 'api_error'
    retry_after TIMESTAMP NOT NULL,
    consecutive_failures INTEGER NOT NULL DEFAULT 1,
    error_message TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
```

**Failure Types and TTLs:**
- `not_found` - 24 hours (ticker doesn't exist, long cooldown)
- `rate_limited` - 1 hour (temporary API limit)
- `api_error` - 6 hours (provider issues, medium cooldown)

**Key Functions:**
- `should_retry_ticker(pool, ticker)` - Check if we can try this ticker
- `get_active_failure(pool, ticker)` - Get failure details
- `record_fetch_failure(pool, ticker, type, error)` - Record a failure
- `clear_fetch_failure(pool, ticker)` - Clear after success

## Implementation Flow

### Fetching Price Data (`refresh_from_api`)

```rust
pub async fn refresh_from_api(
    pool: &PgPool,
    provider: &dyn PriceProvider,
    ticker: &str,
    failure_cache: &FailureCache,
) -> Result<(), AppError> {
    // 1. Check failure cache first
    if !should_retry_ticker(pool, ticker).await? {
        return Err("Ticker in failure cache");
    }

    // 2. Check if we have recent price data
    if let Some(latest) = fetch_latest(pool, ticker).await? {
        if latest.date >= today - 6_hours {
            return Ok(()); // Data is fresh, no API call needed
        }
    }

    // 3. Fetch from API with retry logic
    match provider.fetch_daily_history(ticker, 60).await {
        Ok(points) => {
            // Store in database
            upsert_external_points(pool, ticker, &points).await?;

            // Clear failure cache on success
            clear_fetch_failure(pool, ticker).await?;

            Ok(())
        }
        Err(e) => {
            // Record failure with appropriate TTL
            let failure_type = match e {
                PriceProviderError::RateLimited => "rate_limited",
                PriceProviderError::NotFound => "not_found",
                _ => "api_error",
            };

            record_fetch_failure(pool, ticker, failure_type, Some(&e.to_string())).await?;

            Err(AppError::External(e.to_string()))
        }
    }
}
```

## Benefits

### 1. Performance
- **Fast responses:** Most requests served from database cache
- **Reduced latency:** No waiting for external API calls
- **Predictable performance:** Consistent response times

### 2. Cost Efficiency
- **API quota savings:** Avoid redundant calls for same ticker
- **Rate limit respect:** Don't waste calls on known-bad tickers
- **Smart TTLs:** Different cooldowns for different failure types

### 3. User Experience
- **Faster page loads:** Portfolio/Risk pages load much quicker
- **No repeated errors:** Failed tickers don't keep showing errors
- **Clear feedback:** Users know when data was last updated

### 4. Reliability
- **Graceful degradation:** Serve stale data if API is down
- **Automatic recovery:** Retry after cooldown period expires
- **Error isolation:** One bad ticker doesn't block others

## Usage Examples

### Example 1: Fresh Data Available
```
User requests risk data for AAPL
↓
System checks price_points table
↓
Found AAPL prices updated 2 hours ago
↓
Return cached data immediately ✓
(No API call made)
```

### Example 2: Stale Data, Successful Fetch
```
User requests risk data for MSFT
↓
System checks price_points table
↓
Found MSFT prices but 8 hours old
↓
Check ticker_fetch_failures: no record
↓
Call Twelve Data API
↓
Success! Store 60 days of prices
↓
Return fresh data ✓
```

### Example 3: Known Bad Ticker
```
User requests risk data for INVALID_TICKER
↓
Check ticker_fetch_failures table
↓
Found record: not_found, retry_after: tomorrow
↓
Skip API call, return error immediately ✓
(Saved API quota and time)
```

### Example 4: First Failure
```
User requests risk data for BADTICKER
↓
Check ticker_fetch_failures: no record
↓
Check price_points: no data
↓
Call Twelve Data API
↓
404 Not Found error
↓
Record failure: not_found, retry_after: +24 hours
↓
Return error to user
```

## Monitoring and Maintenance

### Check Active Failures
```rust
let failures = get_all_active_failures(pool).await?;
for failure in failures {
    println!("{}: {} until {}",
        failure.ticker,
        failure.failure_type,
        failure.retry_after
    );
}
```

### Clean Up Expired Failures
```rust
let cleaned = cleanup_expired_failures(pool).await?;
println!("Cleaned up {} expired failure records", cleaned);
```

### Check Cache Hit Rate
```sql
-- Count tickers with recent price data
SELECT COUNT(DISTINCT ticker)
FROM price_points
WHERE created_at > NOW() - INTERVAL '6 hours';

-- Count active failure records
SELECT COUNT(*)
FROM ticker_fetch_failures
WHERE retry_after > NOW();
```

## Configuration

### TTL Values (editable in `ticker_fetch_failure_queries.rs`)

```rust
let retry_after = match failure_type {
    "not_found" => now + Duration::hours(24),    // 24 hours
    "rate_limited" => now + Duration::hours(1),   // 1 hour
    "api_error" => now + Duration::hours(6),      // 6 hours
    _ => now + Duration::hours(6),                // Default
};
```

### Price Data Freshness (editable in `price_service.rs`)

```rust
// Current: 6 hours
if latest.date >= today - ChronoDuration::hours(6) {
    return Ok(()); // Data is fresh enough
}
```

## Migration

### Applied Migration
```bash
sqlx migrate run
# Applied 20260211000000/migrate create ticker fetch failures
```

This creates the `ticker_fetch_failures` table with all necessary indexes.

## Files Modified

### New Files
- `backend/migrations/20260211000000_create_ticker_fetch_failures.sql`
- `backend/src/db/ticker_fetch_failure_queries.rs`
- `backend/PRICE_CACHING_SYSTEM.md` (this file)

### Modified Files
- `backend/src/db/mod.rs` - Added ticker_fetch_failure_queries module
- `backend/src/services/price_service.rs` - Updated refresh_from_api() to use DB failure tracking

## Future Enhancements

### Possible Improvements
1. **Automatic cleanup job:** Periodically run `cleanup_expired_failures()`
2. **Cache statistics endpoint:** `/api/admin/cache-stats` showing hit rates
3. **Manual cache invalidation:** Allow force-refresh for specific tickers
4. **Smart TTL adjustment:** Increase TTL for repeatedly failing tickers
5. **Provider-specific TTLs:** Different cooldowns for Twelve Data vs Alpha Vantage

## Troubleshooting

### Issue: Ticker not updating despite stale data
**Cause:** Ticker is in failure cache
**Solution:** Check `ticker_fetch_failures` table, manually delete record if needed

### Issue: Too many API calls
**Cause:** Price data freshness threshold too short
**Solution:** Increase TTL in `price_service.rs` from 6 hours to 12 or 24 hours

### Issue: Users seeing old data
**Cause:** TTL too long
**Solution:** Decrease TTL or add manual refresh button

## Conclusion

The price caching system dramatically improves performance and reliability by:
- ✅ Storing price data in the database for fast retrieval
- ✅ Tracking failed tickers to avoid redundant API calls
- ✅ Respecting API rate limits with smart TTLs
- ✅ Providing consistent, predictable performance

This foundation enables all Phase 3 risk analysis features to work efficiently at scale.
