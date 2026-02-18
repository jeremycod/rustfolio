use crate::external::price_provider::{ExternalPricePoint, ExternalTickerMatch, PriceProvider, PriceProviderError};
use async_trait::async_trait;
use tracing::{info, warn};

/// MultiProvider attempts to fetch data from multiple providers with intelligent routing.
///
/// Strategy:
/// 1. Detect Canadian stocks (common TSX tickers) and route to Yahoo Finance (free, unlimited)
/// 2. US stocks go to primary provider (Twelve Data)
/// 3. If primary fails, fallback to Alpha Vantage
/// 4. If that fails for known Canadian tickers, try Yahoo Finance with .TO suffix
pub struct MultiProvider {
    primary: Box<dyn PriceProvider>,
    fallback: Box<dyn PriceProvider>,
    yahoo: Box<dyn PriceProvider>,
}

impl MultiProvider {
    pub fn new(
        primary: Box<dyn PriceProvider>,
        fallback: Box<dyn PriceProvider>,
        yahoo: Box<dyn PriceProvider>,
    ) -> Self {
        Self { primary, fallback, yahoo }
    }

    /// Detect if a ticker is likely Canadian based on common patterns
    ///
    /// Returns (is_canadian, normalized_ticker)
    fn detect_canadian_ticker(ticker: &str) -> (bool, String) {
        // Already has .TO suffix
        if ticker.ends_with(".TO") {
            return (true, ticker.to_string());
        }

        // Already has .V suffix (TSX Venture)
        if ticker.ends_with(".V") {
            return (true, ticker.to_string());
        }

        // Known Canadian blue-chip stocks that commonly appear in portfolios
        let known_canadian = [
            "RY",   // Royal Bank of Canada
            "TD",   // TD Bank
            "BNS",  // Scotiabank
            "BMO",  // Bank of Montreal
            "CM",   // CIBC
            "ENB",  // Enbridge
            "CNQ",  // Canadian Natural Resources
            "TRP",  // TC Energy
            "SU",   // Suncor Energy
            "CNR",  // Canadian National Railway
            "CP",   // Canadian Pacific Railway
            "BCE",  // BCE Inc
            "T",    // Telus (ambiguous - also AT&T, but in Canadian context usually Telus)
            "MFC",  // Manulife Financial
            "SLF",  // Sun Life Financial
            "FTS",  // Fortis
            "CCO",  // Cameco
            "TIH",  // Toromont Industries
            "WSP",  // WSP Global
            "AEM",  // Agnico Eagle Mines
            "ABX",  // Barrick Gold
            "NTR",  // Nutrien
            "POW",  // Power Corporation
            "QSR",  // Restaurant Brands International
            "ATD",  // Alimentation Couche-Tard
            "WCN",  // Waste Connections
            "BAM",  // Brookfield Asset Management
            "BIP",  // Brookfield Infrastructure
            "BEP",  // Brookfield Renewable
            "VDY",  // Vanguard FTSE Canadian High Dividend Yield Index ETF
        ];

        if known_canadian.contains(&ticker) {
            // Add .TO suffix for Yahoo Finance
            return (true, format!("{}.TO", ticker));
        }

        // Not identified as Canadian
        (false, ticker.to_string())
    }
}

#[async_trait]
impl PriceProvider for MultiProvider {
    async fn fetch_daily_history(
        &self,
        ticker: &str,
        days: u32,
    ) -> Result<Vec<ExternalPricePoint>, PriceProviderError> {
        // Detect if this is a Canadian ticker
        let (is_canadian, normalized_ticker) = Self::detect_canadian_ticker(ticker);

        if is_canadian {
            info!("🍁 Detected Canadian ticker: {} -> {}, routing to Yahoo Finance", ticker, normalized_ticker);

            // Try Yahoo Finance for Canadian stocks (free, no API key, unlimited)
            match self.yahoo.fetch_daily_history(&normalized_ticker, days).await {
                Ok(data) => {
                    info!("✓ Successfully fetched {} from Yahoo Finance", ticker);
                    return Ok(data);
                }
                Err(e) => {
                    warn!("Yahoo Finance failed for {}: {}. Will try other providers.", ticker, e);
                    // Fall through to try other providers
                }
            }
        }

        // Try primary provider (Twelve Data for US stocks)
        match self.primary.fetch_daily_history(ticker, days).await {
            Ok(data) => {
                info!("✓ Successfully fetched {} from primary provider", ticker);
                return Ok(data);
            }
            Err(PriceProviderError::BadResponse(msg)) if msg.contains("404") || msg.contains("Pro plan") => {
                // Ticker not available in primary provider's free tier
                info!("⚠️ Ticker {} not available in primary provider, trying fallback", ticker);
            }
            Err(PriceProviderError::RateLimited) => {
                info!("⚠️ Primary provider rate limited, trying fallback");
            }
            Err(e) => {
                // Other error (network, etc.) - try fallback anyway
                warn!("Primary provider error for {}: {}", ticker, e);
            }
        }

        // Try fallback provider (Alpha Vantage)
        match self.fallback.fetch_daily_history(ticker, days).await {
            Ok(data) => {
                info!("✓ Successfully fetched {} from fallback provider", ticker);
                return Ok(data);
            }
            Err(e) => {
                warn!("Fallback provider failed for {}: {}", ticker, e);
            }
        }

        // Last resort: Try Yahoo Finance (works for many tickers including Canadian)
        let yahoo_ticker = if ticker.ends_with(".TO") || ticker.ends_with(".V") {
            ticker.to_string()
        } else {
            format!("{}.TO", ticker)
        };

        info!("Last resort: Trying Yahoo Finance with ticker {}", yahoo_ticker);
        match self.yahoo.fetch_daily_history(&yahoo_ticker, days).await {
            Ok(data) => {
                info!("✓ Successfully fetched {} as {} from Yahoo Finance (last resort)", ticker, yahoo_ticker);
                return Ok(data);
            }
            Err(e) => {
                warn!("Yahoo Finance last resort failed for {}: {}", yahoo_ticker, e);
            }
        }

        // All attempts failed
        Err(PriceProviderError::BadResponse(
            format!(
                "Failed to fetch {} from all providers (Twelve Data, Alpha Vantage, Yahoo Finance). \
                The ticker may not exist, or all providers are rate limited.",
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
