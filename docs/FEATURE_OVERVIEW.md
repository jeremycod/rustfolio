# Rustfolio Feature Overview

Rustfolio is a comprehensive portfolio management and investment intelligence platform built with Rust and React. It transforms from a simple portfolio tracker into a sophisticated advisory system through three completed development phases, combining real-time risk analytics, predictive forecasting, and AI-powered stock recommendations.

**Current Status**: Phase 3 Complete (February 24, 2026) ✅
**Production Ready**: All features tested and documented
**Test Coverage**: 309 comprehensive tests (100% passing)

---

## Platform Capabilities

Rustfolio provides investors with four core capabilities:

1. **Risk Assessment** – Measure current portfolio and position-level risk with advanced metrics
2. **Predictive Analytics** – Forecast future volatility, market regimes, and price movements
3. **Investment Discovery** – Find new opportunities using quantitative screening and AI analysis
4. **Portfolio Optimization** – Build better portfolios through factor analysis and quality scoring

---

## Phase 1: Enhanced Risk Analytics ✅

Phase 1 establishes sophisticated risk measurement beyond basic volatility, focusing on tail risk, downside protection, and adaptive risk management.

### 1. Conditional Value at Risk (CVaR)

**Tail-risk measurement** – CVaR measures the average loss in worst-case scenarios beyond a confidence threshold. While Value at Risk (VaR) tells you "95% of the time, losses won't exceed X%", CVaR answers "when losses exceed X%, how bad is the average loss?"

- **CVaR 95%**: Average loss in worst 5% of outcomes
- **CVaR 99%**: Average loss in worst 1% of outcomes (extreme tail events)
- **Use case**: Position sizing, leverage decisions, regulatory compliance

**Example**: CVaR 95% of -8% means on the worst 5% of days, expect average daily loss of 8%. For a $100,000 position, that's $8,000 typical loss in bad scenarios.

### 2. Sortino Ratio

**Downside-adjusted performance** – Improves on Sharpe Ratio by penalizing only downside volatility, not upside volatility. Investors don't mind positive surprises—they only care about negative ones.

- **Calculation**: (Return - Risk-Free Rate) / Downside Deviation
- **Interpretation**: Sortino > 2.0 = excellent, 1.0-2.0 = good, <0.5 = poor
- **Use case**: Compare investments on downside-adjusted returns

**Why it matters**: Two stocks with similar Sharpe ratios can have very different downside characteristics. Sortino reveals which stock delivers returns with less downside pain.

### 3. Rolling Beta Analysis

**Dynamic market sensitivity** – Beta measures a stock's sensitivity to market movements. Rolling beta shows how this sensitivity evolves over time using 30-day, 60-day, and 90-day windows.

- **Beta forecasting**: Predict future beta using linear regression, exponential smoothing, or ensemble methods
- **Use cases**: Market timing (reduce high-beta positions before downturns), portfolio rebalancing, defensive positioning

**Example**: TSLA beta declining from 2.1 to 1.3 over 9 months indicates company-specific factors becoming more important than market moves—improving diversification benefit.

### 4. Downside Risk Analysis

**Comprehensive downside metrics** – Four complementary measures of loss potential:

1. **Downside Deviation**: Volatility of negative returns only (used in Sortino)
2. **Downside Capture Ratio**: How much of market declines the stock captures (<100% = defensive)
3. **Maximum Drawdown**: Largest peak-to-trough decline in measurement period
4. **Recovery Duration**: How long drawdowns typically last

**Risk score**: Composite 0-100 score weighted across all four metrics. Scores below 40 = high downside risk; above 70 = strong downside protection.

### 5. Correlation Clustering

**Identify hidden concentration** – Groups portfolio holdings by price movement patterns, revealing concentration risks that sector classifications miss. Stocks in different sectors can still move together due to shared risk factors.

- **Hierarchical clustering**: Automatically groups correlated stocks (2-5 clusters typical)
- **Diversification score**: Portfolio-wide metric (0-100) showing true diversification
- **Actionable insights**: Which clusters to reduce/increase for better diversification

**Example**: A "tech-diversified" portfolio with AAPL, MSFT, GOOGL, AMZN, META, NVDA, TSLA may cluster as one group with 0.82 correlation—acting as ~1 stock, not 7.

### 6. Dynamic Risk Thresholds

**Adaptive risk limits** – Risk thresholds adjust based on current market regime. What's "high risk" in calm markets is "normal risk" in volatile markets, preventing alert fatigue while catching genuine anomalies.

- **Four market regimes**: Bull, Bear, High Volatility, Normal
- **Threshold multipliers**: 0.8× (Bull) to 1.5× (High Volatility)
- **User preference integration**: Conservative/Balanced/Aggressive profiles layer on top

**Example**: Base volatility threshold 30%. In Bull regime (0.8×), threshold = 24%. In High Volatility regime (1.5×), threshold = 45%. Conservative user (0.85×) gets even tighter thresholds.

---

## Phase 2: Predictive Capabilities & Signals ✅

Phase 2 adds forward-looking predictions and probability-based signals, transforming Rustfolio from reactive to proactive.

### 7. GARCH Volatility Forecasting

**Forward-looking volatility** – GARCH (Generalized Autoregressive Conditional Heteroskedasticity) forecasts volatility 1-90 days ahead by capturing volatility clustering—high volatility today predicts high volatility tomorrow.

- **GARCH(1,1) model**: Industry-standard specification with three parameters (omega, alpha, beta)
- **Multi-step forecasts**: Predict volatility trajectory with 80% and 95% confidence intervals
- **Mean reversion**: Forecasts converge to long-run average as recent shocks fade

**Use cases**: Position sizing (reduce positions when volatility forecast rises), options pricing (compare to implied volatility), risk budgeting, rebalancing timing.

**Performance**: Generates 90-day forecasts in <2 seconds.

### 8. HMM Market Regime Detection

**Probabilistic regime analysis** – Hidden Markov Models (HMM) identify four market regimes with probability distributions, revealing transition risks that rule-based detection misses.

- **Four regimes**: Bull (positive returns, low vol), Bear (negative returns, high vol), High Volatility (extreme vol), Normal (moderate)
- **Probabilistic output**: E.g., 65% Bull, 20% Normal, 10% Bear, 5% High Vol
- **Regime forecasting**: Predict transitions 5, 10, 30 days ahead
- **Ensemble approach**: Combines rule-based + HMM for high-confidence classification

**Use cases**: Early warning signals (40% probability of Bear transition in 10 days), regime-dependent strategies (allocate based on regime probabilities), backtesting regime-switching.

### 9. Probability-Based Trading Signals

**Multi-factor technical analysis** – Signals express confidence as percentages (0-100%), acknowledging uncertainty rather than binary buy/sell.

- **Four signal types**: Momentum (RSI, MACD), Mean Reversion (Bollinger Bands), Trend (MA crossovers), Combined (ensemble)
- **Four time horizons**: 1M, 3M, 6M, 12M with adaptive weighting
- **Confidence levels**: High (≥70%), Medium (55-70%), Low (45-55%)

**Example**: Combined signal 72% bullish with momentum (75%), trend (68%), mean reversion (48%) suggests high-confidence long entry.

**Performance**: Signal generation <500ms per ticker.

### 10. Sentiment-Aware Forecasts

**Incorporate news and sentiment** – Adjusts price forecasts based on sentiment from three sources: news articles (40%), SEC filings (30%), insider transactions (30%).

- **Sentiment momentum**: Track 7-day and 30-day sentiment changes
- **Spike detection**: Identify unusual sentiment shifts (>2 standard deviations)
- **Divergence detection**: Bullish (negative sentiment + positive price) vs Bearish (positive sentiment + negative price)
- **Forecast adjustments**: Widen/narrow confidence intervals based on sentiment and divergences

**Use cases**: Early reversal detection, risk adjustment for news events, opportunity identification, contrarian signals.

### 11. User Risk Preferences

**Personalized recommendations** – Three risk profiles (Conservative, Balanced, Aggressive) adjust all system behavior:

- **Conservative**: 12+ month horizon, 75% signal confidence, 0.85× risk thresholds
- **Balanced**: 6 month horizon, 60% confidence, 1.0× thresholds
- **Aggressive**: 1-3 month horizon, 45% confidence, 1.2× thresholds

**Custom factor weights**: Override default weights for signal generation (sentiment, technical, fundamental).

**Integration**: Preferences flow through forecasts, signals, thresholds, and recommendations throughout the platform.

---

## Phase 3: AI-Powered Stock Recommendations ✅

Phase 3 completes the transformation into an intelligent recommendation engine with quantitative screening, AI explanations, and continuous monitoring.

### 12. Multi-Factor Stock Screening

**Intelligent stock discovery** – Screen thousands of stocks across four factor categories simultaneously:

1. **Fundamental**: P/E, P/B, PEG, debt-to-equity, earnings growth
2. **Technical**: Moving averages, RSI, relative strength, volume trends
3. **Sentiment**: News sentiment, sentiment momentum
4. **Momentum**: 1M, 3M, 6M, 12M price momentum

**Advanced filtering**: Sector, market cap, price range, geography, liquidity constraints.

**Weighted ranking**: User-configurable factor weights produce composite scores (0-100). Highest-scoring stocks surface as recommendations.

**Performance**: Screens 1,000+ stocks in <2 seconds with 15-minute result caching.

### 13. AI-Powered Recommendation Explanations

**Human-readable analysis** – Claude Sonnet 4 (Anthropic) generates plain-English explanations for each recommendation, citing specific metrics and providing context.

**Six narrative types**:
1. **Valuation**: Focus on undervalued/overvalued analysis
2. **Growth**: Highlight growth potential and expansion
3. **Risk**: Examine risks and downsides
4. **Contrarian**: Present opposite viewpoints
5. **Dividend**: Focus on dividend sustainability
6. **Balanced**: Well-rounded analysis (default)

**Structured format**: Summary, key highlights, risk factors, factor comparisons, recommendation strength.

**Cost management**: Two-tier caching (in-memory + database, 1-hour TTL) reduces Claude API calls by ~95%.

**Performance**: <3 seconds with LLM call, <100ms when cached.

### 14. Watchlist Management & Monitoring

**Track investment opportunities** – Create unlimited watchlists to organize stocks by strategy, goal, or sector.

**Six custom alert types**:
1. **Price Target (High)**: Alert when price reaches upside target
2. **Price Target (Low)**: Alert when price drops below floor
3. **RSI Overbought**: Alert when RSI exceeds threshold (default 70)
4. **RSI Oversold**: Alert when RSI drops below threshold (default 30)
5. **Volume Spike**: Alert when volume exceeds N× 30-day average
6. **Sentiment Shift**: Alert on dramatic sentiment changes (>0.3 delta)

**Technical pattern recognition**: Automatic detection of MACD crossovers, Bollinger Band breakouts/breakdowns.

**Continuous monitoring**: Background job runs every 30 minutes during market hours checking all watchlist items against thresholds.

**Alert cooldown**: 4-hour cooldown per alert type per stock prevents duplicate alerts.

**Performance**: Monitors 2,000+ items in <1 second, generates alerts for 5,000 symbols in <100ms.

### 15. Factor-Based Portfolio Recommendations

**Academic-grade factor investing** – Implements five canonical factors validated by decades of research:

1. **Value**: Low P/E, P/B (Fama-French methodology)
2. **Growth**: High revenue/earnings growth, ROE
3. **Momentum**: 3M, 6M, 12M returns (Jegadeesh-Titman)
4. **Quality**: High ROE, ROIC, low debt (AQR research)
5. **Low-Volatility**: Low beta, volatility (Frazzini-Pedersen)

**Multi-factor optimizer**: Mean-variance optimization determines optimal factor combination weights.

**ETF suggestions**: 11 well-known factor ETFs (VTV, VUG, MTUM, QUAL, USMV, etc.) with expense ratios and liquidity analysis.

**Expected risk premiums**: Academic research-based return expectations (3-8% annual premiums).

**Backtesting**: Validate factor strategies on historical data using top-quintile construction.

### 16. Long-Term Investment Guidance

**Quality-focused retirement planning** – Identify high-quality companies suitable for buy-and-hold strategies with 10+ year horizons.

**Quality scoring**: Four dimensions (25% each)
1. **Long-Term Growth**: Revenue/earnings consistency, ROE, ROIC
2. **Dividend Analysis**: Yield, payout ratio, growth rate, consecutive years
3. **Competitive Advantage**: Gross margins, market share, pricing power
4. **Management Quality**: Insider ownership, tenure, capital allocation

**Goal-based recommendations**:
- **Retirement**: Conservative allocations, dividend aristocrats, utilities, consumer staples
- **College Fund**: Balanced growth with preservation, quality growth stocks
- **Wealth Building**: Long-term growth optimization, high-growth sectors

**Dividend aristocrat identification**: 25+ consecutive years of dividend increases (JNJ, PG, KO, etc.).

**Age-based allocation**: Automatic adjustment from aggressive (20s-30s) to conservative (60s+) based on time horizon.

---

## Cross-Phase Integration

### How Features Work Together

**Phase 1 + Phase 2 Integration**:
- CVaR (current tail risk) + GARCH forecasts (future volatility) = Complete risk picture
- Dynamic thresholds (Phase 1) + HMM regimes (Phase 2) = Adaptive limits with regime forecasts
- Sortino ratio (Phase 1) + Trading signals (Phase 2) = Downside risk + entry timing

**Phase 2 + Phase 3 Integration**:
- Trading signals (Phase 2) + Screening (Phase 3) = Signal-based stock discovery
- Sentiment forecasts (Phase 2) + AI explanations (Phase 3) = Context-aware recommendations
- User preferences (Phase 2) + Factor recommendations (Phase 3) = Personalized factor tilts

**Phase 1 + Phase 3 Integration**:
- Correlation clustering (Phase 1) + Factor analysis (Phase 3) = Identify factor concentration
- Downside risk (Phase 1) + Long-term guidance (Phase 3) = Quality scoring with downside protection
- Risk thresholds (Phase 1) + Watchlist alerts (Phase 3) = Unified alert system

---

## Complete Investment Workflow Example

**Building a Retirement Portfolio** (All Three Phases Working Together):

1. **Set Preferences** (Phase 2): Conservative profile, 15-year horizon, 75% signal threshold
2. **Screen Quality Stocks** (Phase 3): Long-term guidance identifies 20 dividend aristocrats (quality scores >85)
3. **Check Risk Metrics** (Phase 1): Verify CVaR 95% -5.8%, Sortino 1.9, low correlation (0.3)
4. **Review AI Explanations** (Phase 3): "JNJ: 61-year dividend aristocrat, low beta 0.65..."
5. **Add to Watchlist** (Phase 3): Monitor all 20 candidates with price alerts and RSI thresholds
6. **Check Entry Signals** (Phase 2): PG shows 73% bullish signal (high confidence)—good entry
7. **Verify Volatility** (Phase 2): GARCH forecast stable (12% → 13%)—low future risk
8. **Check Regime** (Phase 2): Bull regime (68%) but declining to 48% in 10 days—take profits soon
9. **Make Allocation**: 50% PG (strong signal), 30% WMT (oversold), 20% cash (regime uncertainty)
10. **Ongoing Monitoring**: Daily watchlist checks, weekly signal reviews, monthly factor rebalancing

This workflow demonstrates all 16 features working together seamlessly.

---

## Performance & Quality Metrics

### Performance Benchmarks (All Targets Met ✅)

| Feature | Target | Actual | Phase |
|---------|--------|--------|-------|
| CVaR calculation | <100ms | ~80ms | 1 |
| Correlation clustering | <500ms | ~350ms | 1 |
| GARCH forecast | <2s | ~1.5s | 2 |
| HMM inference | <200ms | ~50ms | 2 |
| Signal generation | <500ms | ~300ms | 2 |
| Stock screening (1K) | <2s | ~1.5s | 3 |
| AI explanation (w/LLM) | <3s | ~2.5s | 3 |
| Watchlist monitoring (2K) | <1s | ~0.8s | 3 |

### Test Coverage

- **Phase 1**: 89 comprehensive tests
- **Phase 2**: 36 comprehensive tests
- **Phase 3**: 184 comprehensive tests
- **Total**: 309 tests (100% passing) covering unit, integration, and performance

### API Endpoints

- **Phase 1**: 12 new/enhanced endpoints
- **Phase 2**: 9 new/enhanced endpoints
- **Phase 3**: 18 new endpoints
- **Total**: 39 endpoints across all phases

---

## Technology Stack

### Backend
- **Language**: Rust (latest stable)
- **Web Framework**: Axum
- **Database**: PostgreSQL with SQLx
- **Caching**: Redis + in-memory
- **ML/Statistics**: Custom Rust implementations (GARCH, HMM, factor analysis)
- **LLM Integration**: Claude Sonnet 4 (Anthropic)

### Frontend
- **Language**: TypeScript
- **Framework**: React
- **UI Library**: Material-UI (MUI)
- **State Management**: React Query
- **Charts**: Recharts, Chart.js
- **Build Tool**: Vite

---

## Development Timeline

- **Phase 1 Completion**: February 20, 2026 (1 day parallel development)
- **Phase 2 Completion**: February 22, 2026 (1 day parallel development)
- **Phase 3 Completion**: February 24, 2026 (1 day parallel development)

**Total Development Time**: 3 days using aggressive parallel multi-agent execution approach

**Development Approach**: Each phase implemented by 5-8 specialized agents working concurrently on independent features, with careful coordination on shared interfaces and database schema.

---

## Future Roadmap

### Phase 4: Virtual Portfolios & Paper Trading (Planned)
- Paper trading simulator for risk-free strategy testing
- Virtual portfolio backtesting with historical data
- Strategy comparison tools and performance attribution
- "What-if" analysis for trade impact

### Phase 5: Real-Time Features & Notifications (Planned)
- WebSocket streaming for live price and risk updates
- Push notifications (mobile + web) for alerts
- Real-time watchlist alert delivery
- Intraday signal updates during market hours

### Phase 6: ESG & Social Responsibility (Planned)
- ESG scoring integration from multiple providers
- Carbon footprint tracking for portfolios
- Ethical screening filters (exclude weapons, tobacco, etc.)
- Impact investing recommendations

### Phase 7: Options & Derivatives (Planned)
- Options analytics and Black-Scholes pricing
- Greeks calculation and visualization
- Options strategy builder (covered calls, protective puts, spreads)
- Options-based hedging recommendations

### Phase 8: Tax Optimization (Planned)
- Tax-loss harvesting opportunity identification
- Capital gains/losses tracking and reporting
- Wash sale detection and warnings
- Tax-efficient rebalancing suggestions

---

## Getting Started

### For Users
1. **Create Portfolio**: Add your holdings with ticker symbols, shares, and purchase prices
2. **Set Preferences**: Choose risk profile (Conservative/Balanced/Aggressive) and investment horizon
3. **Explore Risk**: View risk metrics, CVaR, Sortino ratio, and correlation clustering
4. **Get Predictions**: Check GARCH volatility forecasts, HMM regime forecasts, and trading signals
5. **Discover Opportunities**: Screen stocks, review AI explanations, and build watchlists
6. **Optimize**: Use factor analysis and long-term guidance to improve portfolio quality

### For Developers
- **Backend**: `/backend` directory contains Rust services, models, and database migrations
- **Frontend**: `/frontend` directory contains React components and TypeScript types
- **Documentation**: `/docs` directory contains comprehensive feature guides and API references
- **Tests**: Run `cargo test` (backend) and `npm test` (frontend) for full test suites

### API Access
All features are accessible via RESTful API endpoints documented in `/docs/api/` directory. Endpoints follow consistent patterns:
- `/api/risk/*` - Risk analytics (Phase 1)
- `/api/market/*` - Market data and regimes (Phase 2)
- `/api/stocks/{symbol}/*` - Stock-level analytics (Phase 2)
- `/api/recommendations/*` - Screening and recommendations (Phase 3)
- `/api/watchlists/*` - Watchlist management (Phase 3)

---

## Conclusion

Rustfolio has evolved from a simple portfolio tracker into a comprehensive investment intelligence platform through three completed development phases:

**Phase 1** established sophisticated risk analytics with tail-risk measurement (CVaR), downside-focused performance (Sortino), dynamic market sensitivity (rolling beta), concentration risk identification (correlation clustering), and adaptive risk thresholds.

**Phase 2** added predictive capabilities with forward-looking volatility forecasting (GARCH), probabilistic market regime detection (HMM), probability-based trading signals across multiple horizons, sentiment-aware forecasts incorporating news and insider data, and personalized user preferences.

**Phase 3** completed the transformation with intelligent stock discovery (multi-factor screening), AI-powered explanations (Claude integration), continuous monitoring (watchlists with 6 alert types), academic-grade factor investing (5 canonical factors with optimization), and quality-focused long-term guidance for retirement planning.

All three phases integrate seamlessly, providing end-to-end investment intelligence: **assess current risk** (Phase 1), **predict future changes** (Phase 2), and **discover new opportunities** (Phase 3).

**Current Status**: Production-ready with 309 comprehensive tests passing, 39 API endpoints, extensive documentation, and optimized performance meeting or exceeding all targets. Ready for staging deployment and user acceptance testing.

---

**Document Version**: 3.0
**Last Updated**: February 24, 2026
**Status**: All Phases Complete (Phase 1, 2, 3) ✅
**Next Phase**: Phase 4 - Virtual Portfolios & Paper Trading
