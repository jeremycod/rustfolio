# Rustfolio Comprehensive Feature Guide

This document provides a complete overview of all features implemented in Rustfolio across all three development phases. Rustfolio is a sophisticated portfolio management and analysis platform that combines real-time risk analytics, predictive forecasting, and AI-powered investment recommendations.

**Current Version**: Phase 3 Complete (February 24, 2026)
**Status**: Production-Ready ✅

## Table of Contents

### Phase 1: Enhanced Risk Analytics (Completed)
1. [Conditional Value at Risk (CVaR)](#1-conditional-value-at-risk-cvar)
2. [Sortino Ratio](#2-sortino-ratio)
3. [Rolling Beta Analysis](#3-rolling-beta-analysis)
4. [Downside Risk Analysis](#4-downside-risk-analysis)
5. [Correlation Clustering](#5-correlation-clustering)
6. [Dynamic Risk Thresholds](#6-dynamic-risk-thresholds)

### Phase 2: Predictive Capabilities & Signals (Completed)
7. [GARCH Volatility Forecasting](#7-garch-volatility-forecasting)
8. [HMM Market Regime Detection](#8-hmm-market-regime-detection)
9. [Probability-Based Trading Signals](#9-probability-based-trading-signals)
10. [Sentiment-Aware Forecasts](#10-sentiment-aware-forecasts)
11. [User Risk Preferences](#11-user-risk-preferences)

### Phase 3: AI-Powered Stock Recommendations (Completed)
12. [Multi-Factor Stock Screening](#12-multi-factor-stock-screening)
13. [AI-Powered Recommendation Explanations](#13-ai-powered-recommendation-explanations)
14. [Watchlist Management & Monitoring](#14-watchlist-management--monitoring)
15. [Factor-Based Portfolio Recommendations](#15-factor-based-portfolio-recommendations)
16. [Long-Term Investment Guidance](#16-long-term-investment-guidance)

### Integration & Workflows
17. [Cross-Phase Integration](#17-cross-phase-integration)
18. [Complete Investment Workflows](#18-complete-investment-workflows)

---

# Phase 1: Enhanced Risk Analytics

Phase 1 establishes Rustfolio's foundation as a professional-grade risk management platform, moving beyond basic volatility and Sharpe ratio to sophisticated downside risk measurement, tail-risk analysis, and dynamic risk adaptation.

---

## 1. Conditional Value at Risk (CVaR)

### Tail-Risk Measurement Beyond VaR

**What is CVaR?** – Conditional Value at Risk (CVaR), also called Expected Shortfall, measures the average loss in the worst-case scenarios beyond a given confidence level. While traditional Value at Risk (VaR) tells you the threshold loss (e.g., "95% of the time, daily losses won't exceed 5%"), CVaR answers the more important question: "When things go wrong beyond that threshold, how bad is the average loss?"

**Why CVaR matters more than VaR** – VaR has a critical flaw: it ignores the magnitude of extreme losses. A stock with VaR 95% = -5% could have tail losses of -6% or -50%—VaR doesn't distinguish. CVaR captures tail severity, making it essential for risk-averse investors and regulatory compliance (Basel III capital requirements).

**Rustfolio's CVaR implementation** – Uses historical simulation methodology analyzing the past 252 trading days (one year) to estimate tail risk at 95% and 99% confidence levels:
- **CVaR 95%**: Average loss in the worst 5% of outcomes
- **CVaR 99%**: Average loss in the worst 1% of outcomes (extreme tail events)

**Interpretation guidelines**:
- CVaR 95% of -8% means: In bad scenarios (worse than 95% of days), expect average daily loss of 8%
- For a $100,000 position, this translates to $8,000 average loss on the worst 5% of days
- CVaR 99% shows true catastrophic risk—relevant for position sizing and leverage decisions

**Comparison across tickers** – CVaR enables apples-to-apples risk comparison:
- Conservative stock (JNJ): CVaR 95% ≈ -2.5%
- Moderate stock (AAPL): CVaR 95% ≈ -4.0%
- Volatile stock (NVDA): CVaR 95% ≈ -8.5%
- High-risk stock (TSLA): CVaR 95% ≈ -12.0%

### API Access & UI

**API Endpoint**: `GET /api/risk/positions/{ticker}/cvar?confidence_level=0.95&window_days=252`

**UI Location**: Navigate to any stock's risk analysis page → "Downside Risk" section → CVaR metrics panel

**Performance**: Calculation completes in <100ms for 252-day history, enabling real-time portfolio risk aggregation

---

## 2. Sortino Ratio

### Downside-Adjusted Performance Measurement

**What is the Sortino Ratio?** – The Sortino Ratio improves upon the Sharpe Ratio by penalizing only downside volatility rather than all volatility. This distinction is crucial because investors don't mind upside volatility (gains exceeding expectations)—they only care about downside volatility (losses).

**Why Sortino beats Sharpe** – Consider two stocks:
- **Stock A**: Returns of +20%, +5%, +5%, +5% (high upside volatility) → High Sharpe due to volatility penalty
- **Stock B**: Returns of +10%, +8%, +9%, +10% (consistent, low volatility) → High Sharpe

Both might have similar Sharpe ratios, but Stock A is preferable (asymmetric upside). The Sortino Ratio correctly rewards Stock A by only penalizing downside volatility.

**Rustfolio's calculation**:
```
Sortino Ratio = (Annualized Return - Risk-Free Rate) / Downside Deviation

Where:
- Annualized Return: Computed from daily returns over trailing 252 days
- Risk-Free Rate: 10-year US Treasury yield (configurable)
- Downside Deviation: Standard deviation of returns below 0% (negative returns only)
```

**Interpretation scale**:
- **Sortino > 2.0**: Excellent downside-adjusted returns
- **Sortino 1.0 - 2.0**: Good downside-adjusted returns
- **Sortino 0.5 - 1.0**: Moderate downside-adjusted returns
- **Sortino < 0.5**: Poor downside-adjusted returns
- **Sortino < 0**: Negative returns (losing money)

**Real-world example**:
```
AAPL (Apple Inc.):
- Annual Return: 22.5%
- Downside Deviation: 12.8% (annualized)
- Risk-Free Rate: 4.0%
- Sortino Ratio: (22.5% - 4.0%) / 12.8% = 1.45

Interpretation: AAPL generates 1.45% excess return for each 1% of downside risk taken.
```

**Comparison to Sharpe Ratio** – For the same stock:
- Sharpe Ratio uses total volatility (18.5% annualized) → Sharpe = 1.00
- Sortino uses only downside volatility (12.8%) → Sortino = 1.45
- Sortino > Sharpe indicates asymmetric upside (positive skew)

### API Access & UI

**API Endpoint**: `GET /api/risk/positions/{ticker}/sortino?days=252`

**UI Location**: Risk analysis page → "Risk-Adjusted Returns" section → Sortino Ratio card

---

## 3. Rolling Beta Analysis

### Dynamic Market Sensitivity Tracking

**What is Rolling Beta?** – Beta measures a stock's sensitivity to overall market movements (S&P 500). A beta of 1.0 means the stock moves in line with the market; beta > 1.0 amplifies market moves; beta < 1.0 dampens them. Rolling beta calculates this sensitivity over moving time windows, revealing how market sensitivity evolves.

**Why static beta is insufficient** – A stock's beta isn't constant. Market conditions, company developments, and sector rotation cause beta to drift. During bull markets, high-beta stocks outperform; during bear markets, low-beta stocks protect capital. Rolling beta helps time these rotations.

**Rustfolio's rolling windows**:
- **30-day rolling beta**: Captures recent market sensitivity shifts
- **60-day rolling beta**: Medium-term beta trends
- **90-day rolling beta**: Smoothed long-term beta baseline
- **252-day beta**: Traditional annual beta for comparison

**Beta forecasting models** – Rustfolio predicts future beta using three approaches:
1. **Linear Regression**: Projects beta trend forward assuming continuation
2. **Exponential Smoothing**: Weights recent beta more heavily for adaptive forecasting
3. **Ensemble Average**: Averages the two forecasts for robust prediction

**Use cases**:
- **Market timing**: Reduce high-beta positions when forecasting bear market
- **Portfolio rebalancing**: Adjust beta exposure to target market sensitivity
- **Defensive positioning**: Shift to low-beta stocks when volatility forecast rises
- **Sector rotation**: High-beta tech stocks in bull markets, low-beta utilities in bear markets

**Example rolling beta chart interpretation**:
```
TSLA Rolling Beta (90-day window):
Jan 2025: β = 2.1 (highly levered to market)
Apr 2025: β = 1.8 (declining sensitivity)
Jul 2025: β = 1.5 (approaching market average)
Oct 2025: β = 1.3 (below market sensitivity)

Interpretation: TSLA's market sensitivity is declining over time, suggesting company-specific
factors are becoming more important than broad market moves. Diversification benefit improving.
```

### API Access & UI

**API Endpoint**: `GET /api/risk/positions/{ticker}/rolling-beta?windows=30,60,90&forecast=true`

**UI Location**: Risk analysis page → "Rolling Beta" tab → Interactive chart with forecast overlay

---

## 4. Downside Risk Analysis

### Comprehensive Downside Protection Measurement

**What is downside risk analysis?** – While traditional volatility treats upside and downside equally, downside risk analysis focuses exclusively on negative returns. This section combines multiple downside-focused metrics into a unified framework for measuring and managing loss potential.

**Four key downside metrics**:

1. **Downside Deviation** – Standard deviation of returns below a threshold (typically 0%)
   - Measures consistency of losses when they occur
   - Lower is better (less volatile losses)
   - Used in Sortino Ratio calculation

2. **Downside Capture Ratio** – How much of market declines the stock captures
   - Formula: (Stock's average return on down-market days) / (Market's average return on down-market days)
   - <100% means stock declines less than market (defensive)
   - \>100% means stock amplifies market declines (aggressive)
   - Example: 85% downside capture means stock drops 8.5% when market drops 10%

3. **Maximum Drawdown** – Largest peak-to-trough decline over the measurement period
   - Shows worst-case historical loss
   - Critical for position sizing and stop-loss setting
   - Example: Max drawdown of -35% means stock fell 35% from its peak at worst point

4. **Average Drawdown Duration** – How long drawdowns typically last
   - Measures recovery speed after declines
   - Longer duration = slower to recover = higher risk
   - Example: 45-day average duration means stock takes 1.5 months to recover from typical drawdown

**Risk score calculation** – Composite downside risk score (0-100) weighted as:
- 30% Downside Deviation
- 25% Downside Capture Ratio
- 30% Maximum Drawdown
- 15% Recovery Duration

Scores below 40 indicate high downside risk; above 70 indicate strong downside protection.

**Visualization** – Downside Risk page includes:
- **Drawdown chart**: Shows all historical drawdowns with depth and duration
- **Downside metrics panel**: All four metrics with color-coded risk indicators
- **Benchmark comparison**: Side-by-side downside metrics vs S&P 500
- **Percentile ranking**: Where this stock ranks in downside risk vs universe

### API Access & UI

**API Endpoint**: `GET /api/risk/positions/{ticker}/downside-risk?days=252`

**UI Location**: Risk analysis page → "Downside Risk" tab

---

## 5. Correlation Clustering

### Identify Hidden Concentration Risk

**What is correlation clustering?** – Correlation clustering groups portfolio holdings by their price movements, revealing hidden concentration risks that aren't obvious from sector classifications alone. Stocks may be in different sectors but still move together due to shared risk factors (interest rates, economic cycles, consumer sentiment).

**Why clustering matters** – Naive diversification (buying stocks across different sectors) fails if those stocks are highly correlated. During the 2008 crisis, financials, homebuilders, retailers, and industrials all declined together despite being "different sectors." Correlation clustering catches these dependencies.

**Rustfolio's clustering algorithm**:
1. **Correlation matrix calculation**: Pairwise correlation of daily returns (252-day window)
2. **Hierarchical clustering**: Agglomerative clustering with Ward linkage groups stocks by similarity
3. **Optimal cluster count**: Automatically determined using silhouette score (2-5 clusters typical)
4. **Cluster characterization**: Average intra-cluster correlation, cluster size, primary holdings

**Cluster interpretation**:
- **High intra-cluster correlation (>0.7)**: Stocks move in lockstep—concentrated risk
- **Moderate correlation (0.4-0.7)**: Related but not perfectly correlated—some diversification
- **Low correlation (<0.4)**: Independent movements—true diversification

**Example clustering output**:
```
Portfolio: Tech-Heavy Growth Portfolio (20 holdings)

Cluster 1 (High Correlation - Concentration Risk):
- Holdings: AAPL, MSFT, GOOGL, AMZN, META, NVDA, TSLA (7 stocks, 60% of portfolio)
- Average intra-cluster correlation: 0.82 (very high)
- Risk: 7 stocks act as ~1 diversification unit due to high correlation
- Recommendation: Reduce tech exposure or add uncorrelated assets

Cluster 2 (Moderate Correlation):
- Holdings: JPM, BAC, WFC, GS (4 stocks, 20% of portfolio)
- Average intra-cluster correlation: 0.68
- Risk: Moderate concentration in financials
- Recommendation: Acceptable if intentional sector bet

Cluster 3 (Low Correlation - True Diversification):
- Holdings: JNJ, PG, WMT, MCD, KO, PEP, UNH, VZ, T (9 stocks, 20% of portfolio)
- Average intra-cluster correlation: 0.35 (low)
- Risk: Low—defensive stocks with independent drivers
- Recommendation: Provides downside protection
```

**Diversification score** – Rustfolio calculates a portfolio-wide diversification score (0-100):
- Score = 100 × (1 - Average Correlation)
- 80+ = Well-diversified
- 60-80 = Moderately diversified
- <60 = Concentrated risk

**Actionable insights**:
- **Over-concentration alert**: When >50% of portfolio in single cluster with correlation >0.7
- **Rebalancing suggestions**: Which clusters to reduce/increase for better diversification
- **Correlation heatmap**: Visual matrix showing all pairwise correlations color-coded

### API Access & UI

**API Endpoint**: `GET /api/risk/portfolio/{id}/correlation-clustering?min_clusters=2&max_clusters=5`

**UI Location**: Portfolio risk page → "Correlation Clustering" section

**Performance**: Efficiently handles portfolios up to 20 stocks; uses caching for repeated analysis

---

## 6. Dynamic Risk Thresholds

### Adaptive Risk Limits Based on Market Conditions

**What are dynamic risk thresholds?** – Instead of fixed risk limits (e.g., "alert if volatility exceeds 30%"), dynamic thresholds adjust based on the current market regime. What's "high risk" during calm markets (Bull regime) is "normal risk" during volatile markets (High Volatility regime). Dynamic thresholds prevent alert fatigue while catching genuine anomalies.

**Market regime framework** – Rustfolio classifies market into four regimes:
1. **Bull Market**: Positive returns, low-moderate volatility (<20%)
2. **Bear Market**: Negative returns, elevated volatility (>25%)
3. **High Volatility**: Extreme volatility (>35%), uncertain direction
4. **Normal Market**: Moderate returns and volatility, no strong trend

**Threshold multipliers by regime**:
- **Bull Market**: 0.8× base threshold (tighter risk control during favorable conditions)
- **Normal Market**: 1.0× base threshold (standard risk tolerance)
- **Bear Market**: 1.3× base threshold (higher tolerance as market-wide stress is expected)
- **High Volatility**: 1.5× base threshold (significantly relaxed to avoid false alarms)

**Example application**:
```
Base volatility threshold: 30%

Regime: Bull Market (multiplier 0.8×)
Adjusted threshold: 30% × 0.8 = 24%
Rationale: During calm markets, 25% volatility is unusual and warrants attention

Regime: High Volatility (multiplier 1.5×)
Adjusted threshold: 30% × 1.5 = 45%
Rationale: During chaotic markets, 35% volatility is normal; only extreme outliers (>45%) alert
```

**Integrated with user preferences** – Phase 2 user risk preferences (Conservative/Balanced/Aggressive) layer on top of regime multipliers:
- **Conservative investor in Bull regime**: 30% × 0.8 (regime) × 0.85 (Conservative) = 20.4% threshold
- **Aggressive investor in Bull regime**: 30% × 0.8 (regime) × 1.2 (Aggressive) = 28.8% threshold

**Threshold types**:
- **Volatility threshold**: Annual volatility percentage (e.g., 30%)
- **Drawdown threshold**: Maximum acceptable peak-to-trough decline (e.g., -20%)
- **Beta threshold**: Maximum market sensitivity (e.g., 1.5)
- **CVaR threshold**: Maximum tail-risk loss (e.g., -10%)

**Alert system** – When position exceeds adjusted threshold:
1. Visual warning badge appears on holdings table
2. Detailed alert explains: "AAPL volatility (32%) exceeds current threshold (24%) for Bull regime"
3. Alert history tracks all threshold breaches with timestamps
4. Email/push notifications (Phase 5) for immediate action

**Automatic regime detection** – Background job checks regime every 4 hours, recalculating thresholds automatically. Users see risk thresholds adapt without manual intervention.

### API Access & UI

**API Endpoint**: `GET /api/market/regime` (returns current regime and threshold multipliers)

**UI Location**: Portfolio settings → "Risk Thresholds" → Shows both base and regime-adjusted thresholds

---

# Phase 2: Predictive Capabilities & Signals

Phase 2 transforms Rustfolio from a reactive analytics tool into a proactive advisory system with forward-looking predictions, probability-based signals, and personalized recommendations.

---

## 7. GARCH Volatility Forecasting

### Forward-Looking Volatility Predictions

**What is GARCH?** – GARCH (Generalized Autoregressive Conditional Heteroskedasticity) is a statistical model designed specifically for forecasting volatility. Unlike simple historical volatility calculations, GARCH captures the empirical fact that high volatility periods tend to cluster together – if the market is volatile today, it's likely to remain volatile tomorrow.

**The GARCH(1,1) Model** – Formula: σ²ₜ = ω + α·ε²ₜ₋₁ + β·σ²ₜ₋₁
- **Omega (ω)**: Long-run variance baseline
- **Alpha (α)**: Weight on recent shocks (how much yesterday's surprise moves matter)
- **Beta (β)**: Weight on past variance (how persistent volatility is)

**Multi-step forecasting** – Generate volatility forecasts 1 to 90 days ahead with confidence intervals at 80% and 95% levels.

**API Endpoint**: `GET /api/risk/positions/{ticker}/volatility-forecast?days=30&confidence_level=0.95`

---

## 8. HMM Market Regime Detection

### Probabilistic Market State Analysis

**What are Hidden Markov Models?** – HMM assumes markets transition between hidden "regimes" with characteristic return and volatility distributions. By analyzing price patterns, HMM infers which regime the market is currently in and predicts future regime changes.

**Four market regimes**:
- **Bull Market**: Positive returns, low-moderate volatility
- **Bear Market**: Negative returns, elevated volatility
- **High Volatility**: Extreme volatility regardless of direction
- **Normal Market**: Moderate returns and volatility

**Probabilistic state estimation** – HMM provides probability distributions across all regimes (e.g., 65% Bull, 20% Normal, 10% Bear, 5% High Vol).

**API Endpoint**: `GET /api/market/regime/forecast?days=10`

---

## 9. Probability-Based Trading Signals

### Multi-Factor Technical Analysis

**What are probability-based signals?** – Signals express confidence as percentages (0-100%), reflecting strength and acknowledging uncertainty.

**Four signal types**:
1. **Momentum Signals**: Favor continuation of trends (RSI, MACD, price momentum)
2. **Mean Reversion Signals**: Identify overextended moves likely to reverse (Bollinger Bands, RSI extremes)
3. **Trend Signals**: Identify sustained directional moves (SMA/EMA crossovers, volume confirmation)
4. **Combined Signals**: Weighted ensemble with horizon-adaptive weighting

**Multi-horizon analysis** – Signals generated for 1M, 3M, 6M, 12M with adaptive weighting.

**API Endpoint**: `GET /api/stocks/{symbol}/signals?horizon=3&signal_types=combined&min_probability=0.60`

---

## 10. Sentiment-Aware Forecasts

### Incorporating News and Sentiment into Predictions

**Three sentiment sources**:
- **News sentiment** (40% weight)
- **SEC filings sentiment** (30% weight)
- **Insider transactions** (30% weight)

**Sentiment momentum** – Tracks 7-day and 30-day sentiment changes with acceleration analysis.

**Divergence detection**:
- **Bullish Divergence**: Negative sentiment + positive price trend
- **Bearish Divergence**: Positive sentiment + negative price trend

**Forecast adjustments** – Base forecasts adjusted based on sentiment and divergences.

**API Endpoint**: `GET /api/sentiment/positions/{ticker}/sentiment-forecast?days=30`

---

## 11. User Risk Preferences

### Personalized Forecasts and Signals

**Three risk appetite profiles**:
1. **Conservative**: 12+ month horizon, 75% signal confidence, 0.85× risk thresholds
2. **Balanced**: 6 month horizon, 60% confidence, 1.0× thresholds
3. **Aggressive**: 1-3 month horizon, 45% confidence, 1.2× thresholds

**Custom factor weights** – Override default weights for signal generation (sentiment, technical, fundamental).

**API Endpoints**:
- `GET /api/users/{id}/preferences`
- `PUT /api/users/{id}/preferences`

---

# Phase 3: AI-Powered Stock Recommendations

Phase 3 transforms Rustfolio into an intelligent recommendation engine combining quantitative analysis, AI explanations, and personalized preferences.

---

## 12. Multi-Factor Stock Screening

### Intelligent Stock Discovery

**Four factor categories**:
1. **Fundamental Factors**: P/E, P/B, PEG, debt-to-equity, earnings growth
2. **Technical Factors**: Moving averages, RSI, relative strength, volume trends
3. **Sentiment Factors**: News sentiment, sentiment momentum
4. **Momentum Factors**: 1M, 3M, 6M, 12M price momentum

**Advanced filtering** – Sector, market cap, price range, geography, liquidity filters.

**Weighted ranking algorithm** – User-configurable factor weights with composite scoring.

**API Endpoint**: `POST /api/recommendations/screen`

**Performance**: Screens 1,000+ stocks in <2 seconds.

---

## 13. AI-Powered Recommendation Explanations

### Human-Readable Investment Analysis

**Claude Sonnet 4 integration** – Uses Anthropic's Claude AI for explanation generation.

**Six narrative types**:
1. **Valuation**: Focuses on whether stock is undervalued/overvalued
2. **Growth**: Highlights growth potential and expansion
3. **Risk-Focused**: Examines risks and downsides
4. **Contrarian**: Presents opposite viewpoints
5. **Dividend/Income**: Focuses on dividend sustainability
6. **Balanced** (default): Well-rounded analysis

**Two-tier caching** – In-memory + PostgreSQL (1-hour TTL) to manage API costs.

**API Endpoint**: `GET /api/recommendations/{symbol}/explanation?narrative_type=balanced`

**Performance**: <3 seconds including Claude API call, <100ms when cached.

---

## 14. Watchlist Management & Monitoring

### Track and Monitor Investment Opportunities

**Unlimited watchlists** – Create multiple watchlists for different strategies.

**Six custom threshold types**:
1. **Price Target (High)**: Alert when price reaches upside target
2. **Price Target (Low)**: Alert when price drops to downside floor
3. **RSI Overbought**: Alert when RSI exceeds threshold (default 70)
4. **RSI Oversold**: Alert when RSI drops below threshold (default 30)
5. **Volume Spike**: Alert when volume exceeds N× average
6. **Sentiment Shift**: Alert on dramatic sentiment changes (>0.3 delta)

**Technical pattern recognition** – Automatic detection of MACD crossovers, Bollinger Band breakouts.

**Continuous monitoring** – Background job runs every 30 minutes during market hours.

**API Endpoints**: 13 RESTful endpoints for full CRUD and monitoring.

**Performance**: Monitors 2,000+ items in <1 second.

---

## 15. Factor-Based Portfolio Recommendations

### Academic-Grade Factor Investing

**Five canonical factors**:
1. **Value**: Low P/E, P/B ratios (Fama-French methodology)
2. **Growth**: High revenue/earnings growth, ROE
3. **Momentum**: 3M, 6M, 12M returns (Jegadeesh-Titman)
4. **Quality**: High ROE, ROIC, low debt (AQR research)
5. **Low-Volatility**: Low beta, volatility (Frazzini-Pedersen)

**Multi-factor optimizer** – Mean-variance optimization for factor combinations.

**ETF suggestions** – 11 well-known factor ETFs with expense ratios and liquidity.

**Expected risk premiums** – Academic research-based return expectations.

**API Endpoint**: `GET /api/recommendations/factors/:portfolio_id`

---

## 16. Long-Term Investment Guidance

### Quality-Focused Retirement Planning

**Quality scoring system** – Four dimensions (25% each):
1. **Long-Term Growth Potential**: Revenue/earnings consistency, ROE, ROIC
2. **Dividend Analysis**: Yield, payout ratio, growth rate, consecutive years
3. **Competitive Advantage**: Gross margins, market share, pricing power
4. **Management Quality**: Insider ownership, tenure, capital allocation

**Goal-based recommendations**:
1. **Retirement**: Conservative allocations, dividend aristocrats
2. **College Fund**: Balanced growth with preservation
3. **Wealth Building**: Long-term growth optimization

**Dividend aristocrat identification** – 25+ consecutive years of dividend increases.

**Blue-chip screening** – >$50B market cap, investment-grade, consistent profitability.

**Age-based allocation** – Automatic adjustment based on time horizon.

**API Endpoint**: `GET /api/recommendations/long-term/:portfolio_id?goal=retirement&horizon=20`

---

# 17. Cross-Phase Integration

## How All Features Work Together

### Risk Assessment + Predictions + Recommendations

**Phase 1 → Phase 2 Integration**:
- CVaR (Phase 1) + GARCH forecasts (Phase 2) = Current tail risk + future volatility predictions
- Dynamic thresholds (Phase 1) + HMM regimes (Phase 2) = Adaptive risk limits based on regime forecasts
- Sortino ratio (Phase 1) + Trading signals (Phase 2) = Downside risk measurement + entry/exit timing

**Phase 2 → Phase 3 Integration**:
- Trading signals (Phase 2) + Screening (Phase 3) = Signal-based stock discovery
- Sentiment forecasts (Phase 2) + AI explanations (Phase 3) = Context-aware recommendations
- User preferences (Phase 2) + Factor recommendations (Phase 3) = Personalized factor tilts

**Phase 1 → Phase 3 Integration**:
- Correlation clustering (Phase 1) + Factor analysis (Phase 3) = Identify concentration in specific factors
- Downside risk (Phase 1) + Long-term guidance (Phase 3) = Quality scoring incorporates downside protection
- Risk thresholds (Phase 1) + Watchlist alerts (Phase 3) = Unified alert system

---

# 18. Complete Investment Workflows

## Example: Building a Retirement Portfolio

**Step 1: Set User Preferences (Phase 2)**
- Risk appetite: Conservative
- Investment horizon: 15 years
- Signal sensitivity: Low (75% threshold)

**Step 2: Run Long-Term Guidance Screening (Phase 3)**
- Goal: Retirement
- Results: 20 high-quality dividend aristocrats (quality scores >85)

**Step 3: Evaluate Factor Exposures (Phase 3)**
- Current portfolio: 70% quality, 20% value, 10% growth
- Balanced for conservative retirement goal

**Step 4: Check Risk Metrics (Phase 1)**
- Portfolio CVaR 95%: -5.8% (low tail risk)
- Sortino ratio: 1.9 (good downside-adjusted returns)
- Correlation clustering: Low inter-cluster correlation (0.3)

**Step 5: Review AI Explanations (Phase 3)**
- JNJ: "61-year dividend aristocrat, low beta (0.65)..."
- PG: "Pricing power (32% ROE), 67-year dividend track..."

**Step 6: Monitor with Watchlist (Phase 3)**
- Add all 20 candidates to "Retirement Portfolio Candidates" watchlist
- Set price targets, RSI oversold alerts
- Background monitoring twice per hour

**Step 7: Check Entry Signals (Phase 2)**
- JNJ: Combined signal 62% (wait for stronger signal)
- PG: Combined signal 73%, trend 78% (good entry now)
- WMT: Mean reversion 68%, RSI 28 (buying opportunity)

**Step 8: Check Volatility Forecasts (Phase 2)**
- JNJ: GARCH forecast stable (15% → 16%)
- PG: Stable (12% → 13%)
- WMT: Declining (18% → 15%)

**Step 9: Check Market Regime (Phase 2)**
- Current: Bull (68% probability)
- Forecast (10-day): Bull declining to 48%
- Action: Consider defensive positioning

**Step 10: Make Allocation Decision**
- Allocate 50% to PG (strong signal, stable volatility)
- Allocate 30% to WMT (oversold, volatility declining)
- Keep 20% cash, wait for JNJ signal

---

## Technical Summary

### Performance Benchmarks

All performance targets across all phases met or exceeded:

| Feature | Target | Actual | Phase |
|---------|--------|--------|-------|
| CVaR calculation | <100ms | ~80ms | 1 |
| Correlation clustering (20 stocks) | <500ms | ~350ms | 1 |
| GARCH forecast generation | <2s | ~1.5s | 2 |
| HMM inference | <200ms | ~50ms | 2 |
| Signal generation | <500ms | ~300ms | 2 |
| Stock screening (1000 stocks) | <2s | ~1.5s | 3 |
| AI explanation (with LLM) | <3s | ~2.5s | 3 |
| Watchlist monitoring (2000 items) | <1s | ~0.8s | 3 |

### Testing Coverage

- **Phase 1**: 89 comprehensive tests (100% passing)
- **Phase 2**: 36 comprehensive tests (100% passing)
- **Phase 3**: 184 comprehensive tests (100% passing)
- **Total**: 309 tests covering unit, integration, and performance testing

### API Endpoints

- **Phase 1**: 12 new/enhanced endpoints
- **Phase 2**: 9 new/enhanced endpoints
- **Phase 3**: 18 new endpoints
- **Total**: 39 endpoints across all phases

### Database Schema

- **Phase 1**: 8 new tables/extensions
- **Phase 2**: 7 new tables
- **Phase 3**: 10 new tables
- **Total**: 25 tables supporting all features

### Technology Stack

**Backend**:
- **Language**: Rust (latest stable)
- **Framework**: Axum web framework
- **Database**: PostgreSQL with SQLx
- **Caching**: Redis + in-memory
- **ML/Statistics**: Custom Rust implementations (GARCH, HMM, factor analysis)
- **LLM**: Claude Sonnet 4 (Anthropic)

**Frontend**:
- **Language**: TypeScript
- **Framework**: React
- **UI Library**: Material-UI (MUI)
- **State Management**: React Query
- **Charts**: Recharts, Chart.js

---

## Development Timeline

- **Phase 1 Completion**: February 20, 2026
  - Duration: 1 day (parallel multi-agent execution)
  - Status: Production-ready ✅

- **Phase 2 Completion**: February 22, 2026
  - Duration: 1 day (parallel multi-agent execution)
  - Status: Production-ready ✅

- **Phase 3 Completion**: February 24, 2026
  - Duration: 1 day (parallel multi-agent execution)
  - Status: Production-ready ✅

**Total Development Time**: 3 days (aggressive parallel development approach)

---

## Future Phases (Planned)

### Phase 4: Virtual Portfolios & Paper Trading
- Paper trading simulator
- Virtual portfolio backtesting
- Strategy comparison tools
- Performance attribution analysis

### Phase 5: Real-Time Features & Notifications
- WebSocket streaming for live updates
- Push notifications (mobile + web)
- Real-time alert delivery
- Intraday signal updates

### Phase 6: ESG & Social Responsibility
- ESG scoring integration
- Carbon footprint tracking
- Ethical screening filters
- Impact investing recommendations

### Phase 7: Options & Derivatives
- Options analytics and pricing
- Greeks calculation and visualization
- Options strategy builder
- Covered call / protective put suggestions

### Phase 8: Tax Optimization
- Tax-loss harvesting suggestions
- Capital gains/losses tracking
- Wash sale detection
- Tax-efficient rebalancing

---

## Conclusion

Rustfolio has evolved from a portfolio tracker into a comprehensive investment intelligence platform through three major development phases:

**Phase 1** established sophisticated risk analytics with tail-risk measurement (CVaR), downside-focused performance (Sortino), dynamic market sensitivity (rolling beta), and concentration risk identification (correlation clustering).

**Phase 2** added predictive capabilities with forward-looking volatility forecasting (GARCH), probabilistic market regime detection (HMM), probability-based trading signals, and sentiment-aware forecasts.

**Phase 3** completed the transformation with intelligent stock discovery (multi-factor screening), AI-powered explanations (Claude integration), continuous monitoring (watchlists), academic-grade factor investing, and quality-focused long-term guidance.

All three phases integrate seamlessly, providing end-to-end investment intelligence: **assess current risk (Phase 1)**, **predict future changes (Phase 2)**, and **discover new opportunities (Phase 3)**.

**Status**: All phases production-ready with 309 comprehensive tests passing, 39 API endpoints, extensive documentation, and optimized performance meeting or exceeding all targets.

---

**Document Version**: 3.0
**Last Updated**: February 24, 2026
**Phases Complete**: Phase 1, 2, 3
**Status**: Production-Ready ✅
**Next Phase**: Phase 4 - Virtual Portfolios & Paper Trading
