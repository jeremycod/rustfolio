use crate::external::price_provider::{ExternalPricePoint, PriceProvider, PriceProviderError};
use async_trait::async_trait;
use chrono::NaiveDate;
use serde::Deserialize;
use std::collections::BTreeMap;

pub struct AlphaVantageProvider {
    client: reqwest::Client,
    api_key: String,
}

impl AlphaVantageProvider {
    pub fn from_env() -> Result<Self, PriceProviderError> {
        let api_key = std::env::var("ALPHAVANTAGE_API_KEY")
            .map_err(|_| PriceProviderError::BadResponse("ALPHAVANTAGE_API_KEY not set".into()))?;

        Ok(Self {
            client: reqwest::Client::new(),
            api_key,
        })
    }
}

#[derive(Debug, Deserialize)]
struct AvDailyResponse {
    #[serde(rename = "Time Series (Daily)")]
    time_series: Option<BTreeMap<String, AvDailyBar>>,

    // When rate-limited Alpha Vantage returns:
    // { "Note": "Thank you for using Alpha Vantage! ... 5 calls per minute ..." }
    #[serde(rename = "Note")]
    note: Option<String>,

    // When invalid:
    // { "Error Message": "Invalid API call. ..." }
    #[serde(rename = "Error Message")]
    error_message: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AvDailyBar {
    #[serde(rename = "4. close")]
    close: String,
}

#[async_trait]
impl PriceProvider for AlphaVantageProvider {
    async fn fetch_daily_history(
        &self,
        ticker: &str,
        days: u32,
    ) -> Result<Vec<ExternalPricePoint>, PriceProviderError> {
        // AlphaVantage supports:
        // outputsize=compact (latest ~100 points) or full (~20+ years)
        let outputsize = if days <= 100 { "compact" } else { "full" };

        // TIME_SERIES_DAILY is easiest to parse and reliable for closes
        let url = "https://www.alphavantage.co/query";

        let resp = self
            .client
            .get(url)
            .query(&[
                ("function", "TIME_SERIES_DAILY"),
                ("symbol", ticker),
                ("outputsize", outputsize),
                ("apikey", self.api_key.as_str()),
            ])
            .send()
            .await
            .map_err(|e| PriceProviderError::Network(e.to_string()))?;

        let body = resp
            .json::<AvDailyResponse>()
            .await
            .map_err(|e| PriceProviderError::Parse(e.to_string()))?;

        if let Some(note) = body.note {
            // This is the throttle response
            return Err(PriceProviderError::RateLimited);
        }

        if let Some(msg) = body.error_message {
            return Err(PriceProviderError::BadResponse(msg));
        }

        let series = body
            .time_series
            .ok_or_else(|| PriceProviderError::BadResponse("missing time series".into()))?;

        // series is keyed by "YYYY-MM-DD" strings; BTreeMap sorts ascending
        let mut out: Vec<ExternalPricePoint> = Vec::new();

        // Keep only the latest N days if outputsize=full or compact has more than requested.
        // We iterate ascending, then trim from the end.
        for (date_str, bar) in series {
            let date = NaiveDate::parse_from_str(&date_str, "%Y-%m-%d")
                .map_err(|e| PriceProviderError::Parse(e.to_string()))?;

            let close = bar
                .close
                .parse::<f64>()
                .map_err(|e| PriceProviderError::Parse(e.to_string()))?;

            out.push(ExternalPricePoint { date, close });
        }

        // out is already ascending because BTreeMap iterates in order
        if days > 0 && (out.len() as u32) > days {
            let keep = days as usize;
            out = out.into_iter().rev().take(keep).collect::<Vec<_>>();
            out.reverse();
        }

        Ok(out)
    }
}