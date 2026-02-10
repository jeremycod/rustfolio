# Multi-Provider Setup Guide

## Overview

The multi-provider strategy automatically handles both US and Canadian stocks by using:
- **Twelve Data** (primary) - for US stocks (800 calls/day free tier)
- **Alpha Vantage** (fallback) - for Canadian stocks (25 calls/day free tier)

## How It Works

1. **For each ticker request:**
   - First try: Twelve Data with bare ticker (e.g., "IFC")
   - If 404/"Pro plan" error: Try Alpha Vantage with Canadian suffixes
     - IFC.TO (Toronto Stock Exchange)
     - IFC.V (TSX Venture Exchange)
     - IFC (bare ticker as last resort)
   - Return first successful result

2. **Smart fallback logic:**
   - US stocks succeed on first try via Twelve Data
   - Canadian stocks fail on Twelve Data, then succeed via Alpha Vantage
   - Mutual funds fail on both (expected - show N/A badge)

## Your Holdings Analysis

Based on your database query, here's what will happen with multi-provider:

### ✅ US Stocks (via Twelve Data - Primary)
- AAPL (Apple)
- AMZN (Amazon)
- CVX (Chevron)
- GOOGL (Alphabet)
- HOOD (Robinhood)
- JPM (JPMorgan)
- MSFT (Microsoft)
- NI (NiSource)
- NVDA (Nvidia)
- CRCL (Circle)

**Expected behavior:** Fast success on first attempt

### ✅ Canadian Stocks (via Alpha Vantage - Fallback)
- IFC (Intact Financial) → tries IFC.TO
- CCO (Cameco) → tries CCO.TO
- CGL (iShares Gold ETF) → tries CGL.TO
- META CDR (Canadian Depository Receipt) → tries META.TO

**Expected behavior:** Fails on Twelve Data, succeeds on Alpha Vantage with .TO suffix

### ❌ Mutual Funds (Expected to fail - Not stocks)
- AGF9110, BIP502, DYN3128, DYN3361, FID5494, FID5982, etc.
- All mutual fund tickers

**Expected behavior:** Fails on both providers, shows "N/A" risk badge (correct)

## Configuration

### .env File
```bash
# Use multi-provider strategy (recommended)
PRICE_PROVIDER=multi

# Both API keys required
TWELVEDATA_API_KEY=620897b2e9a44cde80271d3d3519ad2c
ALPHAVANTAGE_API_KEY=D26TMX8YM1ODY1HR
```

### Alternative Configurations

**Twelve Data only** (US stocks only):
```bash
PRICE_PROVIDER=twelvedata
```

**Alpha Vantage only** (US + Canadian, but only 25 calls/day):
```bash
PRICE_PROVIDER=alphavantage
```

## Testing

### 1. Restart the backend
```bash
cargo run
```

Look for this log message:
```
📊 Using price provider: Multi-provider (Twelve Data + Alpha Vantage fallback)
```

### 2. Test US Stock (should use Twelve Data)
Watch logs when viewing AAPL risk:
```
INFO rustfolio_backend::services::risk_service: Ensuring fresh price data for ticker: AAPL
INFO rustfolio_backend::external::multi_provider: Successfully fetched AAPL from primary provider
```

### 3. Test Canadian Stock (should fallback to Alpha Vantage)
Watch logs when viewing IFC risk:
```
INFO rustfolio_backend::services::risk_service: Ensuring fresh price data for ticker: IFC
INFO rustfolio_backend::external::multi_provider: Ticker IFC not available in primary provider, trying fallback with Canadian suffixes
INFO rustfolio_backend::external::multi_provider: Trying fallback provider with ticker variant: IFC.TO
INFO rustfolio_backend::external::multi_provider: Successfully fetched IFC as IFC.TO from fallback provider
```

### 4. Test Mutual Fund (should fail gracefully)
Watch logs when viewing DYN3361:
```
WARN rustfolio_backend::routes::risk: No price data available for DYN3361: Failed to fetch...
```
UI should show "N/A" risk badge with tooltip "Risk metrics not available for mutual funds"

## Benefits

### Rate Limit Optimization
- 800 Twelve Data calls/day for US stocks (most of your portfolio)
- 25 Alpha Vantage calls/day reserved for Canadian stocks only
- Total effective limit: much higher than using either alone

### Coverage
- ✅ All US stocks covered (free tier)
- ✅ Canadian stocks covered (free tier)
- ✅ Automatic routing - no manual configuration needed

### Fallback Safety
- If Twelve Data is down, Alpha Vantage provides backup
- If one provider hits rate limit, the other can still work
- Graceful degradation

## Troubleshooting

### "Multi-provider" not showing in logs
- Check `.env` has `PRICE_PROVIDER=multi`
- Restart the backend server

### Canadian stocks still failing
- Check Alpha Vantage API key is valid
- Check Alpha Vantage hasn't hit 25 calls/day limit
- Look for specific error in logs

### US stocks failing
- Check Twelve Data API key is valid
- Check Twelve Data hasn't hit 800 calls/day limit
- Try switching to `PRICE_PROVIDER=alphavantage` as temporary workaround

## Log Interpretation

**Success path (US stock):**
```
[ticker] from primary provider  ← Used Twelve Data (fast)
```

**Fallback path (Canadian stock):**
```
[ticker] not available in primary provider, trying fallback
Trying fallback provider with ticker variant: [ticker].TO
Successfully fetched [ticker] as [ticker].TO from fallback provider  ← Used Alpha Vantage
```

**Failure path (mutual fund):**
```
Fallback attempt failed for [ticker]: ...
Failed to fetch [ticker] from both primary and fallback providers  ← Expected for mutual funds
```
