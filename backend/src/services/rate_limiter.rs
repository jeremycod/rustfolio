use std::sync::Arc;
use tokio::sync::Semaphore;
use tokio::time::{sleep, Duration, Instant};
use parking_lot::Mutex;

/// Rate limiter to control API request frequency
///
/// This prevents exhausting the free tier quotas of price providers like Twelve Data (8 req/min)
/// and Alpha Vantage (5 req/min).
pub struct RateLimiter {
    /// Semaphore to limit concurrent requests
    semaphore: Arc<Semaphore>,
    /// Last request timestamp to enforce minimum delay between requests
    last_request: Arc<Mutex<Instant>>,
    /// Minimum delay between requests in milliseconds
    min_delay: Duration,
}

impl RateLimiter {
    /// Create a new rate limiter
    ///
    /// # Arguments
    /// * `max_concurrent` - Maximum number of concurrent API requests (recommend 3)
    /// * `requests_per_minute` - Maximum requests per minute (recommend 8 for free tier)
    ///
    /// # Example
    /// ```
    /// // Allow max 3 concurrent requests, 8 per minute
    /// let limiter = RateLimiter::new(3, 8);
    /// ```
    pub fn new(max_concurrent: usize, requests_per_minute: u32) -> Self {
        let min_delay_ms = 60_000 / requests_per_minute as u64;
        Self {
            semaphore: Arc::new(Semaphore::new(max_concurrent)),
            last_request: Arc::new(Mutex::new(Instant::now() - Duration::from_secs(60))),
            min_delay: Duration::from_millis(min_delay_ms),
        }
    }

    /// Acquire permission to make a request
    ///
    /// This will block until:
    /// 1. A semaphore permit is available (concurrent limit)
    /// 2. Enough time has passed since the last request (rate limit)
    ///
    /// Returns a guard that releases the permit when dropped.
    pub async fn acquire(&self) -> RateLimitGuard {
        // Wait for a semaphore permit
        let permit = self.semaphore.clone().acquire_owned().await.unwrap();

        // Enforce minimum delay between requests
        let wait_time = {
            let last = self.last_request.lock();
            let elapsed = last.elapsed();

            if elapsed < self.min_delay {
                Some(self.min_delay - elapsed)
            } else {
                None
            }
        }; // Lock is dropped here

        // Sleep outside the lock if needed
        if let Some(delay) = wait_time {
            sleep(delay).await;
        }

        // Update last request time
        *self.last_request.lock() = Instant::now();

        RateLimitGuard { _permit: permit }
    }

    /// Get the current utilization (for monitoring)
    pub fn available_permits(&self) -> usize {
        self.semaphore.available_permits()
    }
}

/// Guard that holds a rate limit permit
/// The permit is automatically released when this is dropped
pub struct RateLimitGuard {
    _permit: tokio::sync::OwnedSemaphorePermit,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant as StdInstant;

    #[tokio::test]
    async fn test_rate_limiter_enforces_delay() {
        // Allow 2 concurrent, 60 per minute (1 per second)
        let limiter = RateLimiter::new(2, 60);

        let start = StdInstant::now();

        // First request should be immediate
        let _guard1 = limiter.acquire().await;
        let elapsed1 = start.elapsed();
        assert!(elapsed1.as_millis() < 100, "First request should be immediate");
        drop(_guard1);

        // Second request should wait ~1 second
        let _guard2 = limiter.acquire().await;
        let elapsed2 = start.elapsed();
        assert!(elapsed2.as_millis() >= 900, "Second request should wait ~1 second");
    }

    #[tokio::test]
    async fn test_concurrent_limit() {
        // Allow max 2 concurrent
        let limiter = Arc::new(RateLimiter::new(2, 120)); // 120/min = 500ms delay

        let limiter1 = limiter.clone();
        let limiter2 = limiter.clone();
        let limiter3 = limiter.clone();

        // Start 3 concurrent requests
        let handle1 = tokio::spawn(async move {
            let _guard = limiter1.acquire().await;
            sleep(Duration::from_millis(100)).await;
        });

        let handle2 = tokio::spawn(async move {
            let _guard = limiter2.acquire().await;
            sleep(Duration::from_millis(100)).await;
        });

        let handle3 = tokio::spawn(async move {
            let _guard = limiter3.acquire().await;
            sleep(Duration::from_millis(100)).await;
        });

        // All should complete (third waits for first two)
        tokio::try_join!(handle1, handle2, handle3).unwrap();
    }
}
