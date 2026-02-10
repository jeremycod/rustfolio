1# Price Provider Configuration

This application supports multiple price data providers for fetching stock market data.

## Provider Strategies

### Multi-Provider (Recommended) ⭐⭐

**How it works:**
- Primary: Twelve Data (for US stocks)
- Fallback: Alpha Vantage (for Canadian stocks with .TO suffix)
- Automatic: Tries primary first, falls back on 404 errors

**Setup:**
```bash
# Both API keys required
export PRICE_PROVIDER=multi
export TWELVEDATA_API_KEY=your_twelvedata_key
export ALPHAVANTAGE_API_KEY=your_alphavantage_key
```

**Coverage:**
- ✅ US stocks via Twelve Data (AAPL, MSFT, CVX, etc.)
- ✅ Canadian stocks via Alpha Vantage (IFC.TO, CCO.TO, etc.)
- ✅ 800 calls/day for US stocks
- ✅ 25 calls/day for Canadian stocks

**Best for:** Portfolios with both US and Canadian holdings

## Available Providers

### 1. Twelve Data (Recommended) ⭐

**Free Tier:**
- 800 API calls per day
- 8 API credits per minute
- Real-time and historical data
- Global coverage (US, Canadian, international stocks)
- Technical indicators included

**Setup:**
```bash
# Get free API key at: https://twelvedata.com/
export PRICE_PROVIDER=twelvedata
export TWELVEDATA_API_KEY=your_key_here
```

**Coverage:**
- ✅ US stocks (AAPL, MSFT, TSLA, etc.)
- ✅ Canadian stocks (IFC, TSX listings)
- ✅ International stocks
- ✅ Forex and crypto (bonus)

**Cost per endpoint:**
- Time series data: 1 credit
- Real-time quotes: 1 credit
- Technical indicators: 1-10 credits

### 2. Alpha Vantage

**Free Tier:**
- 25 API calls per day (very limited)
- 5 API calls per minute
- Historical and real-time data
- US stocks primarily

**Setup:**
```bash
# Get free API key at: https://www.alphavantage.co/
export PRICE_PROVIDER=alphavantage
export ALPHAVANTAGE_API_KEY=your_key_here
```

**Limitations:**
- ❌ Only 25 calls per day (hits limit quickly)
- ❌ Limited international coverage
- ❌ Canadian stocks may need exchange suffix

## Switching Providers

You can switch between providers by updating the `PRICE_PROVIDER` environment variable:

```bash
# Use Twelve Data (recommended)
PRICE_PROVIDER=twelvedata

# Use Alpha Vantage
PRICE_PROVIDER=alphavantage
```

Restart the backend server after changing providers.

## Testing

Test your API key:

```bash
# Twelve Data
curl "https://api.twelvedata.com/time_series?symbol=AAPL&interval=1day&outputsize=5&apikey=$TWELVEDATA_API_KEY"

# Alpha Vantage
curl "https://www.alphavantage.co/query?function=TIME_SERIES_DAILY&symbol=AAPL&outputsize=compact&apikey=$ALPHAVANTAGE_API_KEY"
```

## Implementation

Both providers implement the `PriceProvider` trait:

```rust
pub trait PriceProvider {
    async fn fetch_daily_history(&self, ticker: &str, days: u32)
        -> Result<Vec<ExternalPricePoint>, PriceProviderError>;

    async fn search_ticker_by_keyword(&self, keyword: &str)
        -> Result<Vec<ExternalTickerMatch>, PriceProviderError>;
}
```

Files:
- `src/external/twelvedata.rs` - Twelve Data implementation
- `src/external/alphavantage.rs` - Alpha Vantage implementation
- `src/external/price_provider.rs` - Trait definition
- `src/main.rs` - Provider selection logic

## Recommendation

**Use Twelve Data as your primary provider** because:
- 32x more API calls per day (800 vs 25)
- Better international coverage
- Same features as Alpha Vantage
- More reliable for portfolio tracking applications

Keep Alpha Vantage configured as a backup option if needed.
