use crate::external::price_provider::{ExternalPricePoint, ExternalTickerMatch, PriceProvider, PriceProviderError};
use async_trait::async_trait;
use tracing::{info, warn};

/// MultiProvider attempts to fetch data from multiple providers with fallback logic.
///
/// Strategy:
/// 1. Try primary provider (Twelve Data for US stocks)
/// 2. If primary fails with 404/not found, try Canadian suffix variants with fallback provider
/// 3. Return error only if all attempts fail
pub struct MultiProvider {
    primary: Box<dyn PriceProvider>,
    fallback: Box<dyn PriceProvider>,
}

impl MultiProvider {
    pub fn new(primary: Box<dyn PriceProvider>, fallback: Box<dyn PriceProvider>) -> Self {
        Self { primary, fallback }
    }

    /// Try common Canadian exchange suffixes
    fn canadian_ticker_variants(ticker: &str) -> Vec<String> {
        vec![
            format!("{}.TO", ticker),  // Toronto Stock Exchange
            format!("{}.V", ticker),   // TSX Venture Exchange
            ticker.to_string(),        // Try bare ticker as last resort
        ]
    }
}

#[async_trait]
impl PriceProvider for MultiProvider {
    async fn fetch_daily_history(
        &self,
        ticker: &str,
        days: u32,
    ) -> Result<Vec<ExternalPricePoint>, PriceProviderError> {
        // Try primary provider first (Twelve Data)
        match self.primary.fetch_daily_history(ticker, days).await {
            Ok(data) => {
                info!("Successfully fetched {} from primary provider", ticker);
                return Ok(data);
            }
            Err(PriceProviderError::BadResponse(msg)) if msg.contains("404") || msg.contains("Pro plan") => {
                // Ticker not available in primary provider's free tier
                info!("Ticker {} not available in primary provider, trying fallback with Canadian suffixes", ticker);
            }
            Err(e) => {
                // Other error (network, rate limit, etc.) - propagate immediately
                return Err(e);
            }
        }

        // Try fallback provider with Canadian exchange suffixes
        for variant in Self::canadian_ticker_variants(ticker) {
            info!("Trying fallback provider with ticker variant: {}", variant);
            match self.fallback.fetch_daily_history(&variant, days).await {
                Ok(data) => {
                    info!("Successfully fetched {} as {} from fallback provider", ticker, variant);
                    return Ok(data);
                }
                Err(e) => {
                    warn!("Fallback attempt failed for {}: {}", variant, e);
                    // Continue to next variant
                }
            }
        }

        // All attempts failed
        Err(PriceProviderError::BadResponse(
            format!(
                "Failed to fetch {} from both primary and fallback providers. \
                The ticker may not be available in free tiers, may not exist, or you may have hit rate limits.",
                ticker
            )
        ))
    }

    async fn search_ticker_by_keyword(
        &self,
        keyword: &str
    ) -> Result<Vec<ExternalTickerMatch>, PriceProviderError> {
        // Try primary provider first
        match self.primary.search_ticker_by_keyword(keyword).await {
            Ok(matches) if !matches.is_empty() => {
                return Ok(matches);
            }
            Ok(_) => {
                // Empty results, try fallback
                info!("No results from primary provider for '{}', trying fallback", keyword);
            }
            Err(e) => {
                warn!("Primary provider search failed for '{}': {}", keyword, e);
            }
        }

        // Try fallback provider
        self.fallback.search_ticker_by_keyword(keyword).await
    }
}
