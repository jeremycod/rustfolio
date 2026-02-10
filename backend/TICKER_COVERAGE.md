# Ticker Coverage - Free Tier Limitations

## Overview

With Twelve Data's free tier, you get **800 API calls per day** but with some symbol restrictions.

## What Works ✅

### US Stocks (Free Tier)
All major US exchange stocks are available:

- **NASDAQ**: AAPL, MSFT, GOOGL, AMZN, TSLA, NVDA, META, etc.
- **NYSE**: JPM, CVX, BAC, WMT, JNJ, PG, etc.
- **Other US exchanges**: All major US-listed stocks

**Tested and Working:**
- ✅ AAPL (Apple)
- ✅ MSFT (Microsoft)
- ✅ CVX (Chevron)
- ✅ JPM (JP Morgan)
- ✅ NVDA (Nvidia)

## What Doesn't Work ❌

### Canadian Stocks (Requires Pro Plan)
Canadian stocks like those on the Toronto Stock Exchange (TSX) require a paid subscription:

- ❌ **IFC** (Intact Financial Corp - TSX:IFC)
- ❌ Other TSX-listed stocks

**Twelve Data Error:**
```json
{
  "code": 404,
  "message": "This symbol is available starting with Pro (Pro plan). Consider upgrading now at https://twelvedata.com/pricing",
  "status": "error"
}
```

### Mutual Funds (Not Supported)
Mutual fund tickers are not stock symbols and won't work:

- ❌ **DYN3361**, **DYN3128**, **FID5494** (Fidelity mutual funds)
- ❌ Other mutual fund identifiers

These are expected to show "N/A" risk badges since they're not stocks.

**Twelve Data Error:**
```json
{
  "code": 404,
  "message": "**symbol** or **figi** parameter is missing or invalid",
  "status": "error"
}
```

## Solutions

### For Canadian Stocks

**Option 1: Upgrade to Twelve Data Pro**
- Cost: Starting at $12/month for Pro plan
- Gives access to Canadian and other international markets
- Visit: https://twelvedata.com/pricing

**Option 2: Use Alpha Vantage for Canadian stocks**
- Alpha Vantage may support some Canadian stocks with the proper suffix (e.g., "IFC.TO")
- Limited to 25 calls/day
- Switch provider: `PRICE_PROVIDER=alphavantage`

**Option 3: Manual workaround**
- Import CSV data with Canadian stock holdings
- Use US-listed stocks for risk calculations
- Accept that some holdings won't have risk metrics

### For Mutual Funds

Mutual funds **don't have daily price volatility** the same way stocks do, so risk metrics (beta, volatility, drawdown) aren't as relevant. The "N/A" badge is the correct behavior.

## Checking Ticker Availability

Test if a ticker is available in your free tier:

```bash
# Replace YOUR_API_KEY with your actual Twelve Data key
curl "https://api.twelvedata.com/time_series?symbol=TICKER&interval=1day&outputsize=5&apikey=YOUR_API_KEY"
```

Examples:
```bash
# US stock (works)
curl "https://api.twelvedata.com/time_series?symbol=AAPL&interval=1day&outputsize=5&apikey=YOUR_KEY"

# Canadian stock (requires Pro)
curl "https://api.twelvedata.com/time_series?symbol=IFC&interval=1day&outputsize=5&apikey=YOUR_KEY"
```

## Recommendation

For a portfolio with:
- **US stocks**: Twelve Data free tier is perfect (800 calls/day)
- **Canadian stocks**: Consider Twelve Data Pro ($12/month) or accept limited coverage
- **Mutual funds**: These will always show "N/A" risk - this is expected and correct
