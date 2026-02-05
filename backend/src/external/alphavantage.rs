use crate::external::price_provider::{ExternalPricePoint, ExternalTickerMatch, PriceProvider, PriceProviderError};
use async_trait::async_trait;
use bigdecimal::BigDecimal;
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
struct TickerSearchWrapper {
    #[serde(rename = "bestMatches")]
    best_matches: Vec<ExternalTickerSearchResponse>,
}

#[derive(Debug, Clone, Deserialize)]
struct ExternalTickerSearchResponse {
    #[serde(rename = "1. symbol")]
    pub symbol: String,
    #[serde(rename = "2. name")]
    pub name: String,
    #[serde(rename = "3. type")]
    pub _type: String,
    #[serde(rename = "4. region")]
    pub region: String,
    #[serde(rename = "5. marketOpen")]
    pub marketOpen: String,
    #[serde(rename = "6. marketClose")]
    pub marketClose: String,
    #[serde(rename = "7. timezone")]
    pub timezone: String,
    #[serde(rename = "8. currency")]
    pub currency: String,
    #[serde(rename = "9. matchScore")]
    pub matchScore: String,
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

    async fn search_ticker_by_keyword(
        &self,
        keyword: &str
    ) -> Result<Vec<ExternalTickerMatch>, PriceProviderError> {
    let url = "https://www.alphavantage.co/query";
    let resp = self
    .client
    .get(url)
    .query(&[
    ("function", "SYMBOL_SEARCH"),
    ("keywords", keyword),

    ("apikey", self.api_key.as_str()),
    ])
    .send()
    .await
    .map_err(|e| PriceProviderError::Network(e.to_string()))?;

    let text = resp.text().await
        .map_err(|e| PriceProviderError::Network(e.to_string()))?;
    println!("{}", text);

    let responseWrapper: TickerSearchWrapper = serde_json::from_str(&text)
        .map_err(|e| PriceProviderError::Parse(format!("JSON parse error: {} | Response: {}", e, text)))?;

    let out: Vec<ExternalTickerMatch> = responseWrapper
        .best_matches
        .into_iter()
        .map(|ticker_match| -> Result<ExternalTickerMatch, PriceProviderError>{
            Ok(ExternalTickerMatch {
                symbol: ticker_match.symbol,
                name: ticker_match.name,
                _type: ticker_match._type,
                region: ticker_match.region,
                currency: ticker_match.currency,
                matchScore: ticker_match.matchScore.parse::<f64>()
                    .map_err(|e| PriceProviderError::Parse(e.to_string()))?,
            })
            
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(out)
    }
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
        let mut out: Vec<ExternalPricePoint> = series
            .into_iter()
            .map(|(date_str, bar)| -> Result<ExternalPricePoint, PriceProviderError> {
                let date = NaiveDate::parse_from_str(&date_str, "%Y-%m-%d")
                    .map_err(|e| PriceProviderError::Parse(e.to_string()))?;
                let close = bar.close.parse::<BigDecimal>()
                    .map_err(|e| PriceProviderError::Parse(e.to_string()))?;
                Ok(ExternalPricePoint { date, close })
            })
            .collect::<Result<Vec<_>, _>>()?;

        if days > 0 && out.len() > days as usize {
            out.drain(..out.len() - days as usize);
        }

        Ok(out)
    }
}