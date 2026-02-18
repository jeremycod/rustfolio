use std::sync::Arc;
use chrono::{DateTime, Utc, Duration};
use dashmap::DashMap;

/// Information about a failed API call for a ticker
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct FailureInfo {
    pub failed_at: DateTime<Utc>,
    pub error_type: FailureType,
    pub ttl_hours: i64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FailureType {
    NotFound,       // Ticker doesn't exist or not available in provider
    RateLimited,    // Temporary rate limit
    ApiError,       // Other API errors
}

/// Thread-safe cache to track failed ticker API calls
/// Prevents repeated expensive API calls for tickers that we know will fail
#[derive(Clone)]
pub struct FailureCache {
    cache: Arc<DashMap<String, FailureInfo>>,
}

impl FailureCache {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(DashMap::new()),
        }
    }

    /// Check if a ticker is in the failure cache and the failure is still valid
    #[allow(dead_code)]
    pub fn is_failed(&self, ticker: &str) -> Option<FailureInfo> {
        if let Some(entry) = self.cache.get(ticker) {
            let info = entry.value().clone();
            let now = Utc::now();
            let expiry = info.failed_at + Duration::hours(info.ttl_hours);

            // Check if the failure is still within TTL
            if now < expiry {
                return Some(info);
            } else {
                // TTL expired, remove from cache
                drop(entry); // Release the read lock
                self.cache.remove(ticker);
            }
        }
        None
    }

    /// Record a failed API call for a ticker
    pub fn record_failure(&self, ticker: &str, error_type: FailureType) {
        let ttl_hours = match error_type {
            FailureType::NotFound => 24,      // Don't retry not-found tickers for 24 hours
            FailureType::RateLimited => 1,    // Retry rate-limited tickers after 1 hour
            FailureType::ApiError => 6,       // Retry other errors after 6 hours
        };

        let info = FailureInfo {
            failed_at: Utc::now(),
            error_type,
            ttl_hours,
        };

        self.cache.insert(ticker.to_string(), info);
    }

    /// Clear a ticker from the failure cache (e.g., after successful fetch)
    pub fn clear(&self, ticker: &str) {
        self.cache.remove(ticker);
    }

    /// Clear all expired entries from the cache
    #[allow(dead_code)]
    pub fn cleanup_expired(&self) {
        let now = Utc::now();
        self.cache.retain(|_, info| {
            let expiry = info.failed_at + Duration::hours(info.ttl_hours);
            now < expiry
        });
    }

    /// Get the number of cached failures
    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.cache.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_records_and_retrieves_failures() {
        let cache = FailureCache::new();

        cache.record_failure("INVALID", FailureType::NotFound);

        let result = cache.is_failed("INVALID");
        assert!(result.is_some());
        assert_eq!(result.unwrap().error_type, FailureType::NotFound);
    }

    #[test]
    fn test_cache_clears_ticker() {
        let cache = FailureCache::new();

        cache.record_failure("TEST", FailureType::NotFound);
        assert!(cache.is_failed("TEST").is_some());

        cache.clear("TEST");
        assert!(cache.is_failed("TEST").is_none());
    }

    #[test]
    fn test_different_ttls_for_error_types() {
        let cache = FailureCache::new();

        cache.record_failure("NOT_FOUND", FailureType::NotFound);
        cache.record_failure("RATE_LIMITED", FailureType::RateLimited);

        let not_found = cache.is_failed("NOT_FOUND").unwrap();
        let rate_limited = cache.is_failed("RATE_LIMITED").unwrap();

        assert_eq!(not_found.ttl_hours, 24);
        assert_eq!(rate_limited.ttl_hours, 1);
    }
}
