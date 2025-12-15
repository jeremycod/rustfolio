use crate::external::price_provider::{ExternalPricePoint, PriceProvider, PriceProviderError};
use async_trait::async_trait;
use chrono::{NaiveDate, NaiveDateTime};
use serde::Deserialize;

pub struct YahooProvider {
    client: reqwest::Client,
}

impl YahooProvider {
    pub fn new() -> Self {
        Self { client: reqwest::Client::new() }
    }
}

// Minimal response structs (only what we need)
#[derive(Debug, Deserialize)]
struct YahooChartResponse {
    chart: YahooChart,
}

#[derive(Debug, Deserialize)]
struct YahooChart {
    result: Option<Vec<YahooResult>>,
    error: Option<serde_json::Value>,
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
impl PriceProvider for YahooProvider {
    async fn fetch_daily_history(
        &self,
        ticker: &str,
        days: u32,
    ) -> Result<Vec<ExternalPricePoint>, PriceProviderError> {
        // Yahoo supports ranges like "6mo", "1y". We'll map days roughly.
        let range = if days <= 30 { "1mo" }
        else if days <= 180 { "6mo" }
        else { "1y" };

        let url = format!(
            "https://query1.finance.yahoo.com/v8/finance/chart/{ticker}?range={range}&interval=1d"
        );

        let resp = self.client
            .get(url)
            .send()
            .await
            .map_err(|e| PriceProviderError::Network(e.to_string()))?;

        if resp.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
            return Err(PriceProviderError::RateLimited);
        }

        let body = resp
            .json::<YahooChartResponse>()
            .await
            .map_err(|e| PriceProviderError::Parse(e.to_string()))?;

        let result = body.chart.result
            .and_then(|mut r| r.pop())
            .ok_or_else(|| PriceProviderError::BadResponse("missing result".into()))?;

        // timestamp aligns with close list by index
        let closes = result.indicators.quote
            .get(0)
            .ok_or_else(|| PriceProviderError::BadResponse("missing quote".into()))?
            .close
            .clone();

        let mut out = Vec::new();

        for (i, ts) in result.timestamp.iter().enumerate() {
            let close = closes.get(i).and_then(|v| *v);

            // skip missing closes
            let Some(close) = close else { continue };

            let dt = NaiveDateTime::from_timestamp_opt(*ts, 0)
                .ok_or_else(|| PriceProviderError::Parse("bad timestamp".into()))?;

            out.push(ExternalPricePoint {
                date: dt.date(),
                close,
            });
        }

        // Ensure ascending by date
        out.sort_by_key(|p| p.date);

        Ok(out)
    }
}