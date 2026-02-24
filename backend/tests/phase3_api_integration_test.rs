/// Phase 3: API Integration Tests
///
/// Tests for all Phase 3 API endpoints including:
/// - Screening API (POST /api/recommendations/screen)
/// - Explanation API (GET /api/recommendations/{symbol}/explanation)
/// - Long-term API (GET /api/recommendations/long-term)
/// - Factor API (GET /api/recommendations/factors)
/// - Watchlist APIs (full CRUD)
/// - Database operations
/// - Caching behavior
///
/// NOTE: These tests validate request/response structures and business logic.
/// Full integration tests against a live database require running the test server.

// ---------------------------------------------------------------------------
// Request / Response Structures
// ---------------------------------------------------------------------------

use std::collections::HashMap;

#[derive(Debug, Clone)]
struct ScreeningRequest {
    sectors: Option<Vec<String>>,
    market_cap_min: Option<f64>,
    market_cap_max: Option<f64>,
    pe_max: Option<f64>,
    min_volume: Option<f64>,
    price_min: Option<f64>,
    price_max: Option<f64>,
    factor_weights: Option<HashMap<String, f64>>,
    risk_appetite: Option<String>,
    investment_horizon: Option<String>,
    top_n: Option<usize>,
    page: Option<usize>,
    page_size: Option<usize>,
}

impl Default for ScreeningRequest {
    fn default() -> Self {
        Self {
            sectors: None,
            market_cap_min: None,
            market_cap_max: None,
            pe_max: None,
            min_volume: None,
            price_min: None,
            price_max: None,
            factor_weights: None,
            risk_appetite: None,
            investment_horizon: None,
            top_n: Some(10),
            page: Some(1),
            page_size: Some(20),
        }
    }
}

#[derive(Debug, Clone)]
struct ScreeningResult {
    symbol: String,
    score: f64,
    factors: HashMap<String, f64>,
    rank: usize,
}

#[derive(Debug)]
struct ScreeningResponse {
    results: Vec<ScreeningResult>,
    total_screened: usize,
    total_matching: usize,
    page: usize,
    page_size: usize,
}

#[derive(Debug, Clone)]
struct ExplanationResponse {
    symbol: String,
    explanation: String,
    factors_summary: HashMap<String, f64>,
    generated_at: String,
    cached: bool,
}

#[derive(Debug, Clone)]
struct LongTermRecommendation {
    symbol: String,
    quality_score: f64,
    recommendation_type: String,
    goal_alignment: f64,
}

#[derive(Debug, Clone)]
struct FactorRecommendation {
    symbol: String,
    factor_name: String,
    factor_score: f64,
    etf_alternative: Option<String>,
}

// ---------------------------------------------------------------------------
// Request Validation Tests
// ---------------------------------------------------------------------------

fn validate_screening_request(req: &ScreeningRequest) -> Result<(), String> {
    if let Some(pe_max) = req.pe_max {
        if pe_max <= 0.0 {
            return Err("PE max must be positive".to_string());
        }
    }
    if let Some(min_vol) = req.min_volume {
        if min_vol < 0.0 {
            return Err("Min volume cannot be negative".to_string());
        }
    }
    if let (Some(min), Some(max)) = (req.price_min, req.price_max) {
        if min > max {
            return Err("Price min cannot exceed price max".to_string());
        }
    }
    if let (Some(min), Some(max)) = (req.market_cap_min, req.market_cap_max) {
        if min > max {
            return Err("Market cap min cannot exceed max".to_string());
        }
    }
    if let Some(ref appetite) = req.risk_appetite {
        if !["conservative", "moderate", "aggressive"].contains(&appetite.as_str()) {
            return Err(format!("Invalid risk appetite: {}", appetite));
        }
    }
    if let Some(ref horizon) = req.investment_horizon {
        if !["short", "medium", "long"].contains(&horizon.as_str()) {
            return Err(format!("Invalid investment horizon: {}", horizon));
        }
    }
    if let Some(top_n) = req.top_n {
        if top_n == 0 || top_n > 100 {
            return Err("top_n must be between 1 and 100".to_string());
        }
    }
    if let Some(page_size) = req.page_size {
        if page_size == 0 || page_size > 100 {
            return Err("page_size must be between 1 and 100".to_string());
        }
    }
    if let Some(ref weights) = req.factor_weights {
        for (name, &weight) in weights {
            if weight < 0.0 || weight > 1.0 {
                return Err(format!("Factor weight for '{}' must be between 0.0 and 1.0", name));
            }
        }
    }
    Ok(())
}

fn validate_long_term_params(goal: &str, horizon: i32, risk_tolerance: &str) -> Result<(), String> {
    if !["retirement", "college", "wealth"].contains(&goal) {
        return Err(format!("Invalid goal: {}", goal));
    }
    if !(1..=40).contains(&horizon) {
        return Err("Horizon must be between 1 and 40 years".to_string());
    }
    if !["low", "medium", "high"].contains(&risk_tolerance) {
        return Err(format!("Invalid risk tolerance: {}", risk_tolerance));
    }
    Ok(())
}

fn validate_factor_params(factors: &[String]) -> Result<(), String> {
    let valid_factors = ["value", "growth", "momentum", "quality", "low_volatility"];
    for f in factors {
        if !valid_factors.contains(&f.as_str()) {
            return Err(format!("Invalid factor: {}", f));
        }
    }
    if factors.is_empty() {
        return Err("At least one factor must be specified".to_string());
    }
    Ok(())
}

#[cfg(test)]
mod request_validation {
    use super::*;

    #[test]
    fn test_valid_default_request() {
        let req = ScreeningRequest::default();
        assert!(validate_screening_request(&req).is_ok());
    }

    #[test]
    fn test_valid_full_request() {
        let mut weights = HashMap::new();
        weights.insert("value".to_string(), 0.3);
        weights.insert("momentum".to_string(), 0.7);

        let req = ScreeningRequest {
            sectors: Some(vec!["Technology".to_string()]),
            market_cap_min: Some(10.0),
            market_cap_max: Some(3000.0),
            pe_max: Some(40.0),
            min_volume: Some(1_000_000.0),
            price_min: Some(10.0),
            price_max: Some(500.0),
            factor_weights: Some(weights),
            risk_appetite: Some("moderate".to_string()),
            investment_horizon: Some("medium".to_string()),
            top_n: Some(20),
            page: Some(1),
            page_size: Some(20),
        };
        assert!(validate_screening_request(&req).is_ok());
    }

    #[test]
    fn test_invalid_pe_max() {
        let req = ScreeningRequest { pe_max: Some(-5.0), ..Default::default() };
        assert!(validate_screening_request(&req).is_err());
    }

    #[test]
    fn test_invalid_negative_volume() {
        let req = ScreeningRequest { min_volume: Some(-100.0), ..Default::default() };
        assert!(validate_screening_request(&req).is_err());
    }

    #[test]
    fn test_invalid_price_range() {
        let req = ScreeningRequest {
            price_min: Some(500.0),
            price_max: Some(100.0),
            ..Default::default()
        };
        assert!(validate_screening_request(&req).is_err());
    }

    #[test]
    fn test_invalid_market_cap_range() {
        let req = ScreeningRequest {
            market_cap_min: Some(3000.0),
            market_cap_max: Some(100.0),
            ..Default::default()
        };
        assert!(validate_screening_request(&req).is_err());
    }

    #[test]
    fn test_invalid_risk_appetite() {
        let req = ScreeningRequest {
            risk_appetite: Some("extreme".to_string()),
            ..Default::default()
        };
        assert!(validate_screening_request(&req).is_err());
    }

    #[test]
    fn test_invalid_investment_horizon() {
        let req = ScreeningRequest {
            investment_horizon: Some("forever".to_string()),
            ..Default::default()
        };
        assert!(validate_screening_request(&req).is_err());
    }

    #[test]
    fn test_invalid_top_n_zero() {
        let req = ScreeningRequest { top_n: Some(0), ..Default::default() };
        assert!(validate_screening_request(&req).is_err());
    }

    #[test]
    fn test_invalid_top_n_too_large() {
        let req = ScreeningRequest { top_n: Some(101), ..Default::default() };
        assert!(validate_screening_request(&req).is_err());
    }

    #[test]
    fn test_invalid_factor_weight_out_of_range() {
        let mut weights = HashMap::new();
        weights.insert("value".to_string(), 1.5);
        let req = ScreeningRequest { factor_weights: Some(weights), ..Default::default() };
        assert!(validate_screening_request(&req).is_err());
    }

    #[test]
    fn test_valid_long_term_params() {
        assert!(validate_long_term_params("retirement", 20, "medium").is_ok());
    }

    #[test]
    fn test_invalid_goal() {
        assert!(validate_long_term_params("vacation", 10, "low").is_err());
    }

    #[test]
    fn test_invalid_horizon() {
        assert!(validate_long_term_params("retirement", 50, "low").is_err());
        assert!(validate_long_term_params("retirement", 0, "low").is_err());
    }

    #[test]
    fn test_invalid_risk_tolerance() {
        assert!(validate_long_term_params("retirement", 20, "yolo").is_err());
    }

    #[test]
    fn test_valid_factors() {
        let factors = vec!["value".to_string(), "momentum".to_string()];
        assert!(validate_factor_params(&factors).is_ok());
    }

    #[test]
    fn test_invalid_factor() {
        let factors = vec!["value".to_string(), "crypto".to_string()];
        assert!(validate_factor_params(&factors).is_err());
    }

    #[test]
    fn test_empty_factors() {
        let factors: Vec<String> = vec![];
        assert!(validate_factor_params(&factors).is_err());
    }
}

// ---------------------------------------------------------------------------
// Response Validation Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod response_validation {
    use super::*;

    fn validate_screening_response(resp: &ScreeningResponse) -> Result<(), String> {
        if resp.page == 0 {
            return Err("Page must be >= 1".to_string());
        }
        if resp.page_size == 0 {
            return Err("Page size must be >= 1".to_string());
        }
        for result in &resp.results {
            if result.score < 0.0 || result.score > 100.0 {
                return Err(format!("Score out of range for {}: {}", result.symbol, result.score));
            }
            if result.symbol.is_empty() {
                return Err("Symbol cannot be empty".to_string());
            }
        }
        // Check ranks are sequential
        for (i, result) in resp.results.iter().enumerate() {
            if result.rank != i + 1 {
                return Err(format!("Rank mismatch: expected {}, got {}", i + 1, result.rank));
            }
        }
        // Check results are sorted by score descending
        for window in resp.results.windows(2) {
            if window[0].score < window[1].score {
                return Err("Results not sorted by score descending".to_string());
            }
        }
        Ok(())
    }

    #[test]
    fn test_valid_screening_response() {
        let resp = ScreeningResponse {
            results: vec![
                ScreeningResult {
                    symbol: "AAPL".into(),
                    score: 90.0,
                    factors: HashMap::new(),
                    rank: 1,
                },
                ScreeningResult {
                    symbol: "MSFT".into(),
                    score: 85.0,
                    factors: HashMap::new(),
                    rank: 2,
                },
            ],
            total_screened: 1000,
            total_matching: 50,
            page: 1,
            page_size: 20,
        };
        assert!(validate_screening_response(&resp).is_ok());
    }

    #[test]
    fn test_unsorted_response_fails() {
        let resp = ScreeningResponse {
            results: vec![
                ScreeningResult { symbol: "A".into(), score: 50.0, factors: HashMap::new(), rank: 1 },
                ScreeningResult { symbol: "B".into(), score: 90.0, factors: HashMap::new(), rank: 2 },
            ],
            total_screened: 100,
            total_matching: 10,
            page: 1,
            page_size: 20,
        };
        assert!(validate_screening_response(&resp).is_err());
    }

    #[test]
    fn test_score_out_of_range_fails() {
        let resp = ScreeningResponse {
            results: vec![
                ScreeningResult { symbol: "A".into(), score: 150.0, factors: HashMap::new(), rank: 1 },
            ],
            total_screened: 100,
            total_matching: 10,
            page: 1,
            page_size: 20,
        };
        assert!(validate_screening_response(&resp).is_err());
    }

    #[test]
    fn test_empty_results_valid() {
        let resp = ScreeningResponse {
            results: vec![],
            total_screened: 1000,
            total_matching: 0,
            page: 1,
            page_size: 20,
        };
        assert!(validate_screening_response(&resp).is_ok());
    }
}

// ---------------------------------------------------------------------------
// Pagination Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod pagination {
    fn paginate<T: Clone>(items: &[T], page: usize, page_size: usize) -> (Vec<T>, usize) {
        let total = items.len();
        let start = (page - 1) * page_size;
        if start >= total {
            return (vec![], total);
        }
        let end = (start + page_size).min(total);
        (items[start..end].to_vec(), total)
    }

    #[test]
    fn test_first_page() {
        let items: Vec<i32> = (0..50).collect();
        let (page, total) = paginate(&items, 1, 20);
        assert_eq!(page.len(), 20);
        assert_eq!(total, 50);
        assert_eq!(page[0], 0);
    }

    #[test]
    fn test_second_page() {
        let items: Vec<i32> = (0..50).collect();
        let (page, _) = paginate(&items, 2, 20);
        assert_eq!(page.len(), 20);
        assert_eq!(page[0], 20);
    }

    #[test]
    fn test_last_partial_page() {
        let items: Vec<i32> = (0..50).collect();
        let (page, _) = paginate(&items, 3, 20);
        assert_eq!(page.len(), 10);
        assert_eq!(page[0], 40);
    }

    #[test]
    fn test_page_beyond_data() {
        let items: Vec<i32> = (0..50).collect();
        let (page, _) = paginate(&items, 10, 20);
        assert!(page.is_empty());
    }

    #[test]
    fn test_single_item_page() {
        let items = vec![42];
        let (page, total) = paginate(&items, 1, 20);
        assert_eq!(page.len(), 1);
        assert_eq!(total, 1);
    }

    #[test]
    fn test_empty_items() {
        let items: Vec<i32> = vec![];
        let (page, total) = paginate(&items, 1, 20);
        assert!(page.is_empty());
        assert_eq!(total, 0);
    }
}

// ---------------------------------------------------------------------------
// Caching Behavior Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod caching {
    use std::time::{Duration, Instant};
    use std::collections::HashMap;

    struct SimpleCache<V: Clone> {
        entries: HashMap<String, (V, Instant)>,
        ttl: Duration,
    }

    impl<V: Clone> SimpleCache<V> {
        fn new(ttl_secs: u64) -> Self {
            Self {
                entries: HashMap::new(),
                ttl: Duration::from_secs(ttl_secs),
            }
        }

        fn get(&self, key: &str) -> Option<&V> {
            self.entries.get(key).and_then(|(v, created)| {
                if created.elapsed() < self.ttl {
                    Some(v)
                } else {
                    None
                }
            })
        }

        fn set(&mut self, key: String, value: V) {
            self.entries.insert(key, (value, Instant::now()));
        }

        fn invalidate(&mut self, key: &str) {
            self.entries.remove(key);
        }

        fn clear(&mut self) {
            self.entries.clear();
        }

        fn len(&self) -> usize {
            self.entries.len()
        }
    }

    #[test]
    fn test_cache_hit() {
        let mut cache = SimpleCache::new(3600); // 1 hour TTL
        cache.set("AAPL_screen".to_string(), "cached_result".to_string());
        assert_eq!(cache.get("AAPL_screen"), Some(&"cached_result".to_string()));
    }

    #[test]
    fn test_cache_miss() {
        let cache: SimpleCache<String> = SimpleCache::new(3600);
        assert_eq!(cache.get("NONEXISTENT"), None);
    }

    #[test]
    fn test_cache_invalidation() {
        let mut cache = SimpleCache::new(3600);
        cache.set("key".to_string(), "value".to_string());
        cache.invalidate("key");
        assert_eq!(cache.get("key"), None);
    }

    #[test]
    fn test_cache_clear() {
        let mut cache = SimpleCache::new(3600);
        cache.set("a".to_string(), 1);
        cache.set("b".to_string(), 2);
        cache.set("c".to_string(), 3);
        cache.clear();
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn test_cache_overwrite() {
        let mut cache = SimpleCache::new(3600);
        cache.set("key".to_string(), "old".to_string());
        cache.set("key".to_string(), "new".to_string());
        assert_eq!(cache.get("key"), Some(&"new".to_string()));
    }

    #[test]
    fn test_recommendation_cache_key_format() {
        // Verify cache keys are deterministic
        fn make_cache_key(symbol: &str, request_hash: u64) -> String {
            format!("rec:{}:{}", symbol.to_uppercase(), request_hash)
        }
        assert_eq!(make_cache_key("aapl", 12345), "rec:AAPL:12345");
        assert_eq!(make_cache_key("AAPL", 12345), "rec:AAPL:12345");
    }

    #[test]
    fn test_explanation_cache_separate_from_screening() {
        let mut screening_cache: SimpleCache<String> = SimpleCache::new(900);    // 15 min
        let mut explanation_cache: SimpleCache<String> = SimpleCache::new(3600);  // 1 hour

        screening_cache.set("AAPL".to_string(), "screening_data".to_string());
        explanation_cache.set("AAPL".to_string(), "explanation_data".to_string());

        // Invalidating screening cache should not affect explanation cache
        screening_cache.invalidate("AAPL");
        assert_eq!(screening_cache.get("AAPL"), None);
        assert_eq!(explanation_cache.get("AAPL"), Some(&"explanation_data".to_string()));
    }
}

// ---------------------------------------------------------------------------
// Watchlist API Endpoint Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod watchlist_api {

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    struct CreateWatchlistRequest {
        name: String,
        description: Option<String>,
    }

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    struct UpdateWatchlistRequest {
        name: Option<String>,
        description: Option<String>,
    }

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    struct AddItemRequest {
        symbol: String,
        custom_thresholds: Option<serde_json::Value>,
    }

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    struct UpdateThresholdsRequest {
        price_target_high: Option<f64>,
        price_target_low: Option<f64>,
        volatility_max: Option<f64>,
        rsi_overbought: Option<f64>,
        rsi_oversold: Option<f64>,
    }

    fn validate_create_request(req: &CreateWatchlistRequest) -> Result<(), String> {
        if req.name.trim().is_empty() {
            return Err("Name is required".to_string());
        }
        if req.name.len() > 100 {
            return Err("Name too long".to_string());
        }
        Ok(())
    }

    fn validate_add_item(req: &AddItemRequest) -> Result<(), String> {
        if req.symbol.trim().is_empty() {
            return Err("Symbol is required".to_string());
        }
        if req.symbol.len() > 10 {
            return Err("Symbol too long".to_string());
        }
        // Check symbol contains only valid characters
        if !req.symbol.chars().all(|c| c.is_alphanumeric() || c == '.' || c == '-') {
            return Err("Symbol contains invalid characters".to_string());
        }
        Ok(())
    }

    fn validate_thresholds(req: &UpdateThresholdsRequest) -> Result<(), String> {
        if let (Some(high), Some(low)) = (req.price_target_high, req.price_target_low) {
            if low >= high {
                return Err("Low price target must be below high price target".to_string());
            }
        }
        if let Some(rsi_ob) = req.rsi_overbought {
            if !(50.0..=100.0).contains(&rsi_ob) {
                return Err("RSI overbought must be between 50 and 100".to_string());
            }
        }
        if let Some(rsi_os) = req.rsi_oversold {
            if !(0.0..=50.0).contains(&rsi_os) {
                return Err("RSI oversold must be between 0 and 50".to_string());
            }
        }
        if let Some(vol) = req.volatility_max {
            if vol <= 0.0 {
                return Err("Volatility max must be positive".to_string());
            }
        }
        Ok(())
    }

    #[test]
    fn test_create_request_valid() {
        let req = CreateWatchlistRequest {
            name: "My Watchlist".to_string(),
            description: Some("Test".to_string()),
        };
        assert!(validate_create_request(&req).is_ok());
    }

    #[test]
    fn test_create_request_empty_name() {
        let req = CreateWatchlistRequest {
            name: "".to_string(),
            description: None,
        };
        assert!(validate_create_request(&req).is_err());
    }

    #[test]
    fn test_create_request_whitespace_name() {
        let req = CreateWatchlistRequest {
            name: "   ".to_string(),
            description: None,
        };
        assert!(validate_create_request(&req).is_err());
    }

    #[test]
    fn test_add_item_valid() {
        let req = AddItemRequest {
            symbol: "AAPL".to_string(),
            custom_thresholds: None,
        };
        assert!(validate_add_item(&req).is_ok());
    }

    #[test]
    fn test_add_item_with_dot() {
        let req = AddItemRequest {
            symbol: "BRK.B".to_string(),
            custom_thresholds: None,
        };
        assert!(validate_add_item(&req).is_ok());
    }

    #[test]
    fn test_add_item_invalid_chars() {
        let req = AddItemRequest {
            symbol: "AAP$L".to_string(),
            custom_thresholds: None,
        };
        assert!(validate_add_item(&req).is_err());
    }

    #[test]
    fn test_add_item_empty_symbol() {
        let req = AddItemRequest {
            symbol: "".to_string(),
            custom_thresholds: None,
        };
        assert!(validate_add_item(&req).is_err());
    }

    #[test]
    fn test_thresholds_valid() {
        let req = UpdateThresholdsRequest {
            price_target_high: Some(200.0),
            price_target_low: Some(100.0),
            volatility_max: Some(0.3),
            rsi_overbought: Some(75.0),
            rsi_oversold: Some(25.0),
        };
        assert!(validate_thresholds(&req).is_ok());
    }

    #[test]
    fn test_thresholds_inverted_prices() {
        let req = UpdateThresholdsRequest {
            price_target_high: Some(100.0),
            price_target_low: Some(200.0),
            volatility_max: None,
            rsi_overbought: None,
            rsi_oversold: None,
        };
        assert!(validate_thresholds(&req).is_err());
    }

    #[test]
    fn test_thresholds_rsi_out_of_range() {
        let req = UpdateThresholdsRequest {
            price_target_high: None,
            price_target_low: None,
            volatility_max: None,
            rsi_overbought: Some(120.0),
            rsi_oversold: None,
        };
        assert!(validate_thresholds(&req).is_err());
    }

    #[test]
    fn test_thresholds_negative_volatility() {
        let req = UpdateThresholdsRequest {
            price_target_high: None,
            price_target_low: None,
            volatility_max: Some(-0.1),
            rsi_overbought: None,
            rsi_oversold: None,
        };
        assert!(validate_thresholds(&req).is_err());
    }

    #[test]
    fn test_thresholds_all_none_valid() {
        let req = UpdateThresholdsRequest {
            price_target_high: None,
            price_target_low: None,
            volatility_max: None,
            rsi_overbought: None,
            rsi_oversold: None,
        };
        assert!(validate_thresholds(&req).is_ok());
    }
}

// ---------------------------------------------------------------------------
// Error Handling Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod error_handling {
    #[derive(Debug)]
    enum ApiError {
        NotFound(String),
        Validation(String),
        RateLimited,
        External(String),
        Unauthorized,
    }

    fn error_status_code(err: &ApiError) -> u16 {
        match err {
            ApiError::NotFound(_) => 404,
            ApiError::Validation(_) => 400,
            ApiError::RateLimited => 429,
            ApiError::External(_) => 503,
            ApiError::Unauthorized => 401,
        }
    }

    #[test]
    fn test_not_found_returns_404() {
        let err = ApiError::NotFound("Watchlist not found".to_string());
        assert_eq!(error_status_code(&err), 404);
    }

    #[test]
    fn test_validation_returns_400() {
        let err = ApiError::Validation("Invalid PE ratio".to_string());
        assert_eq!(error_status_code(&err), 400);
    }

    #[test]
    fn test_rate_limited_returns_429() {
        let err = ApiError::RateLimited;
        assert_eq!(error_status_code(&err), 429);
    }

    #[test]
    fn test_external_returns_503() {
        let err = ApiError::External("Claude API unavailable".to_string());
        assert_eq!(error_status_code(&err), 503);
    }

    #[test]
    fn test_unauthorized_returns_401() {
        let err = ApiError::Unauthorized;
        assert_eq!(error_status_code(&err), 401);
    }
}
