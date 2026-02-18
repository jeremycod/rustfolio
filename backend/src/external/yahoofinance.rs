use crate::external::price_provider::{ExternalPricePoint, ExternalTickerMatch, PriceProvider, PriceProviderError};
use async_trait::async_trait;
use bigdecimal::BigDecimal;
use serde::Deserialize;

/// Yahoo Finance provider - Free API with excellent support for Canadian stocks
///
/// No API key required! Perfect for Canadian stocks (*.TO) that aren't available
/// in free tiers of other providers.
pub struct YahooFinanceProvider {
    client: reqwest::Client,
}

impl YahooFinanceProvider {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::builder()
                .user_agent("Mozilla/5.0 (compatible; Rustfolio/0.1)")
                .build()
                .unwrap_or_else(|_| reqwest::Client::new()),
        }
    }
}

#[derive(Debug, Deserialize)]
struct YahooChartResponse {
    chart: YahooChart,
}

#[derive(Debug, Deserialize)]
struct YahooChart {
    result: Option<Vec<YahooResult>>,
    error: Option<YahooError>,
}

#[derive(Debug, Deserialize)]
struct YahooError {
    description: String,
}

#[derive(Debug, Deserialize)]
struct YahooResult {
    timestamp: Vec<i64>,
    indicators: YahooIndicators,
}

#[derive(Debug, Deserialize)]
struct YahooIndicators {
    quote: Vec<YahooQuote>,
}

#[derive(Debug, Deserialize)]
struct YahooQuote {
    close: Vec<Option<f64>>,
}

#[async_trait]
impl PriceProvider for YahooFinanceProvider {
    async fn fetch_daily_history(
        &self,
        ticker: &str,
        days: u32,
    ) -> Result<Vec<ExternalPricePoint>, PriceProviderError> {
        // Yahoo Finance v8 API endpoint
        let url = format!("https://query1.finance.yahoo.com/v8/finance/chart/{}", ticker);

        // Calculate date range
        // Yahoo uses "1d", "5d", "1mo", "3mo", "6mo", "1y", "2y", "5y", "10y", "ytd", "max"
        let range = if days <= 5 {
            "5d"
        } else if days <= 30 {
            "1mo"
        } else if days <= 90 {
            "3mo"
        } else if days <= 180 {
            "6mo"
        } else if days <= 365 {
            "1y"
        } else if days <= 730 {
            "2y"
        } else {
            "5y"
        };

        let resp = self
            .client
            .get(&url)
            .query(&[
                ("interval", "1d"),
                ("range", range),
                ("includeAdjustedClose", "true"),
            ])
            .send()
            .await
            .map_err(|e| PriceProviderError::Network(e.to_string()))?;

        // Check HTTP status
        if !resp.status().is_success() {
            if resp.status().as_u16() == 404 {
                return Err(PriceProviderError::NotFound);
            }
            return Err(PriceProviderError::BadResponse(
                format!("HTTP {}", resp.status())
            ));
        }

        let body: YahooChartResponse = resp
            .json()
            .await
            .map_err(|e| PriceProviderError::Parse(e.to_string()))?;

        // Check for API errors
        if let Some(error) = body.chart.error {
            if error.description.contains("No data found") {
                return Err(PriceProviderError::NotFound);
            }
            return Err(PriceProviderError::BadResponse(error.description));
        }

        // Extract results
        let results = body.chart.result
            .ok_or_else(|| PriceProviderError::BadResponse("No results in response".into()))?;

        if results.is_empty() {
            return Err(PriceProviderError::NotFound);
        }

        let result = &results[0];
        let timestamps = &result.timestamp;

        if result.indicators.quote.is_empty() {
            return Err(PriceProviderError::BadResponse("No quote data in response".into()));
        }

        let closes = &result.indicators.quote[0].close;

        if timestamps.len() != closes.len() {
            return Err(PriceProviderError::Parse(
                "Timestamp and close price arrays have different lengths".into()
            ));
        }

        // Convert to our format
        let mut points: Vec<ExternalPricePoint> = timestamps
            .iter()
            .zip(closes.iter())
            .filter_map(|(timestamp, close_opt)| {
                // Skip null values (market holidays, etc.)
                let close = (*close_opt)?;

                // Convert Unix timestamp to NaiveDate
                let date = chrono::DateTime::from_timestamp(*timestamp, 0)
                    .map(|dt| dt.date_naive())?;

                // Convert f64 to BigDecimal
                let close_bd = BigDecimal::try_from(close).ok()?;

                Some(ExternalPricePoint {
                    date,
                    close: close_bd,
                })
            })
            .collect();

        // Sort by date (oldest first)
        points.sort_by(|a, b| a.date.cmp(&b.date));

        if points.is_empty() {
            return Err(PriceProviderError::NotFound);
        }

        Ok(points)
    }

    async fn search_ticker_by_keyword(
        &self,
        keyword: &str,
    ) -> Result<Vec<ExternalTickerMatch>, PriceProviderError> {
        // Yahoo Finance search API is less documented and more restricted
        // For Phase 3, we'll use a simpler approach: just validate the ticker exists
        // by trying to fetch 1 day of data

        // Try the ticker as-is first
        if self.fetch_daily_history(keyword, 5).await.is_ok() {
            return Ok(vec![ExternalTickerMatch {
                symbol: keyword.to_string(),
                name: format!("Yahoo Finance: {}", keyword),
                _type: "Stock".to_string(),
                region: "Unknown".to_string(),
                currency: "Unknown".to_string(),
                match_score: 1.0,
            }]);
        }

        // Try with .TO suffix for Canadian stocks
        let canadian_ticker = format!("{}.TO", keyword);
        if self.fetch_daily_history(&canadian_ticker, 5).await.is_ok() {
            return Ok(vec![ExternalTickerMatch {
                symbol: canadian_ticker.clone(),
                name: format!("Yahoo Finance: {}", canadian_ticker),
                _type: "Stock".to_string(),
                region: "Canada".to_string(),
                currency: "CAD".to_string(),
                match_score: 1.0,
            }]);
        }

        // No matches found
        Ok(vec![])
    }
}
