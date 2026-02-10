use crate::external::price_provider::{ExternalPricePoint, ExternalTickerMatch, PriceProvider, PriceProviderError};
use async_trait::async_trait;
use bigdecimal::BigDecimal;
use chrono::NaiveDate;
use serde::Deserialize;

pub struct TwelveDataProvider {
    client: reqwest::Client,
    api_key: String,
}

impl TwelveDataProvider {
    pub fn from_env() -> Result<Self, PriceProviderError> {
        let api_key = std::env::var("TWELVEDATA_API_KEY")
            .map_err(|_| PriceProviderError::BadResponse("TWELVEDATA_API_KEY not set".into()))?;

        Ok(Self {
            client: reqwest::Client::new(),
            api_key,
        })
    }
}

#[derive(Debug, Deserialize)]
struct TwelveDataSearchResponse {
    data: Vec<TwelveDataSearchMatch>,
    status: String,
}

#[derive(Debug, Clone, Deserialize)]
struct TwelveDataSearchMatch {
    symbol: String,
    instrument_name: String,
    exchange: String,
    #[serde(default)]
    mic_code: String,
    #[serde(default)]
    exchange_timezone: String,
    instrument_type: String,
    country: String,
    currency: String,
}

#[derive(Debug, Deserialize)]
struct TwelveDataTimeSeriesResponse {
    meta: Option<TwelveDataMeta>,
    values: Option<Vec<TwelveDataValue>>,
    status: String,

    // Error handling
    message: Option<String>,
    code: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct TwelveDataMeta {
    symbol: String,
    interval: String,
    currency: String,
    exchange: String,
    #[serde(rename = "type")]
    instrument_type: String,
}

#[derive(Debug, Deserialize)]
struct TwelveDataValue {
    datetime: String,
    open: String,
    high: String,
    low: String,
    close: String,
    volume: Option<String>,
}

#[async_trait]
impl PriceProvider for TwelveDataProvider {
    async fn search_ticker_by_keyword(
        &self,
        keyword: &str
    ) -> Result<Vec<ExternalTickerMatch>, PriceProviderError> {
        let url = "https://api.twelvedata.com/symbol_search";

        let resp = self
            .client
            .get(url)
            .query(&[
                ("symbol", keyword),
                ("outputsize", "30"),
                ("apikey", self.api_key.as_str()),
            ])
            .send()
            .await
            .map_err(|e| PriceProviderError::Network(e.to_string()))?;

        let body: TwelveDataSearchResponse = resp
            .json()
            .await
            .map_err(|e| PriceProviderError::Parse(e.to_string()))?;

        if body.status != "ok" {
            return Err(PriceProviderError::BadResponse(
                format!("API returned status: {}", body.status)
            ));
        }

        let matches = body.data
            .into_iter()
            .enumerate()
            .map(|(idx, m)| ExternalTickerMatch {
                symbol: m.symbol,
                name: m.instrument_name,
                _type: m.instrument_type,
                region: m.country,
                currency: m.currency,
                // Calculate match score based on position (first result = highest score)
                matchScore: 1.0 - (idx as f64 * 0.05),
            })
            .collect();

        Ok(matches)
    }

    async fn fetch_daily_history(
        &self,
        ticker: &str,
        days: u32,
    ) -> Result<Vec<ExternalPricePoint>, PriceProviderError> {
        let url = "https://api.twelvedata.com/time_series";

        // Twelve Data uses "1day" for daily data
        // outputsize determines how many data points (default 30, max 5000)
        let outputsize = std::cmp::min(days, 5000);

        let resp = self
            .client
            .get(url)
            .query(&[
                ("symbol", ticker),
                ("interval", "1day"),
                ("outputsize", &outputsize.to_string()),
                ("apikey", self.api_key.as_str()),
            ])
            .send()
            .await
            .map_err(|e| PriceProviderError::Network(e.to_string()))?;

        let body: TwelveDataTimeSeriesResponse = resp
            .json()
            .await
            .map_err(|e| PriceProviderError::Parse(e.to_string()))?;

        // Check for rate limiting or errors
        if body.status != "ok" {
            if let Some(msg) = body.message {
                // Check for rate limit messages
                if msg.contains("API rate limit") || msg.contains("credits") {
                    return Err(PriceProviderError::RateLimited);
                }
                return Err(PriceProviderError::BadResponse(msg));
            }
            return Err(PriceProviderError::BadResponse(
                format!("API returned status: {}", body.status)
            ));
        }

        let values = body.values
            .ok_or_else(|| PriceProviderError::BadResponse("missing values in response".into()))?;

        // Convert to our format
        let mut points: Vec<ExternalPricePoint> = values
            .into_iter()
            .map(|v| -> Result<ExternalPricePoint, PriceProviderError> {
                // Parse datetime - Twelve Data returns "YYYY-MM-DD HH:MM:SS" or "YYYY-MM-DD"
                let date = if v.datetime.contains(' ') {
                    // Has time component, extract just the date part
                    let date_part = v.datetime.split(' ').next()
                        .ok_or_else(|| PriceProviderError::Parse("invalid datetime format".into()))?;
                    NaiveDate::parse_from_str(date_part, "%Y-%m-%d")
                        .map_err(|e| PriceProviderError::Parse(e.to_string()))?
                } else {
                    // Just date
                    NaiveDate::parse_from_str(&v.datetime, "%Y-%m-%d")
                        .map_err(|e| PriceProviderError::Parse(e.to_string()))?
                };

                let close = v.close.parse::<BigDecimal>()
                    .map_err(|e| PriceProviderError::Parse(e.to_string()))?;

                Ok(ExternalPricePoint { date, close })
            })
            .collect::<Result<Vec<_>, _>>()?;

        // Twelve Data returns newest first, we need oldest first
        points.reverse();

        Ok(points)
    }
}
