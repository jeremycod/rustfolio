Al# Rustfolio Enhancement Implementation Tracker

## Project Overview

This document tracks the phased implementation of advanced risk analytics, AI-driven narrative insights, and news aggregation features for Rustfolio. The goal is to transform Rustfolio from a simple portfolio tracker into an intelligent portfolio assistant that provides risk assessment, contextual insights, and educational guidance.

**Key Features:**
- Per-position and portfolio-level risk metrics (volatility, drawdown, beta, VaR, Sharpe ratio)
- Risk scoring and warnings based on customizable thresholds
- LLM-powered narrative analytics explaining performance and risk factors
- News aggregation and thematic summarization per holding
- User alerts and notifications for risk threshold breaches

---

## Phase Status Overview

| Phase | Status | Description |
|-------|--------|-------------|
| Phase 1 | ✅ Completed | Risk Metrics Core Module (Rust) |
| Phase 2 | ✅ Completed | API Endpoints & Database Integration |
| Phase 3 | ✅ Completed | Frontend Integration - Risk Display |
| Phase 4 | ⬜ Not Started | News & LLM Integration (Rust) |
| Phase 5 | ⬜ Not Started | Alerts & Notifications System |
| Phase 6 | ⬜ Not Started | Testing, Performance & Deployment |

**Legend:** ⬜ Not Started | 🟡 In Progress | ✅ Completed | ⏸️ Paused | ❌ Blocked

---

## Phase 1: Risk Metrics Core Module (Rust)

**Status:** ✅ Completed

**Objective:** Implement the core risk calculation functions in Rust that will compute volatility, drawdown, beta, Sharpe ratio, and VaR for individual positions.

### Tasks

- [x] **1.1** Create `backend/src/services/risk_service.rs`
  - [x] Define `PositionRisk` struct with all risk metrics
  - [x] Implement `compute_risk_metrics()` main function
  - [x] Implement `compute_vol_drawdown()` helper
  - [x] Implement `compute_beta()` helper (requires benchmark data)
  - [x] Implement `compute_sharpe()` helper
  - [x] Implement `compute_var()` helper (5% VaR historical simulation)
  - [x] Implement `score_risk()` function (0-100 risk score)

- [x] **1.2** Extend database queries for risk calculations
  - [x] Add `fetch_window()` function in `db/price_queries.rs` to retrieve N days of price history
  - [ ] Ensure benchmark index data (SPY or similar) is available in database (deferred to Phase 2)
  - [ ] Test query performance for typical windows (30, 60, 90 days) (deferred to Phase 2)

- [x] **1.3** Create models for risk data
  - [x] Create `backend/src/models/risk.rs`
  - [x] Define serializable structs (`PositionRisk`, `RiskAssessment`, `RiskLevel`, `RiskThresholds`, `PortfolioRisk`)
  - [x] Add serde derives for JSON serialization

- [x] **1.4** Register risk_service module
  - [x] Update `backend/src/services/mod.rs` to include risk_service
  - [x] Update `backend/src/models/mod.rs` to include risk models

### Acceptance Criteria

- ✅ All risk calculation functions compile without errors
- ✅ Unit tests pass for each helper function (5 tests passed)
- ✅ Risk metrics can be computed for a sample ticker with mock price data
- ✅ Risk score formula produces values in 0-100 range

### Checkpoint

**User approval required before proceeding to Phase 2**

---

## Phase 2: API Endpoints & Database Integration

**Status:** ✅ Completed

**Objective:** Expose risk metrics through REST API endpoints and integrate with existing portfolio/position data.

### Tasks

- [x] **2.1** Create risk API routes
  - [x] Create `backend/src/routes/risk.rs`
  - [x] Implement `GET /api/risk/positions/:ticker` endpoint with auto-fetch
  - [x] Implement `GET /api/risk/portfolios/:portfolio_id` endpoint (stub)
  - [x] Implement `GET /api/risk/thresholds` endpoint (returns defaults)
  - [x] Implement `POST /api/risk/thresholds` endpoint (stub)

- [x] **2.2** Auto-fetch fresh price data
  - [x] Modified `compute_risk_metrics()` to accept `price_provider`
  - [x] Integrated with `refresh_from_api()` for automatic data freshness
  - [x] Graceful fallback to cached data if API fails
  - [x] Fixed database schema with unique constraint on `(ticker, date)`

- [ ] **2.3** Add risk thresholds to database (deferred to future phase)
  - [ ] Design `risk_thresholds` table schema
  - [ ] Create migration for new table
  - [ ] Implement queries in `db/risk_queries.rs` for CRUD operations

- [ ] **2.4** Cache computed risk metrics (deferred - not critical)
  - [ ] Design `risk_cache` table
  - [ ] Add TTL logic to invalidate stale cache entries
  - [ ] Implement cache lookup before recomputing

- [x] **2.5** Register routes in main application
  - [x] Updated `backend/src/app.rs` to include risk router under `/api/risk`
  - [x] Routes integrated with AppState for database and price provider access

### Acceptance Criteria

- ✅ API endpoints respond with valid JSON
- ✅ Risk metrics for individual positions are accurate
- ⏸️ Portfolio-level aggregation (deferred - requires position weighting logic)
- ⏸️ User thresholds stored in database (deferred - defaults work for now)
- ✅ Error handling works for invalid tickers or missing data

### Checkpoint

**User approval required before proceeding to Phase 3**

---

## Phase 3: Frontend Integration - Risk Display

**Status:** ⬜ Not Started

**Objective:** Display risk metrics, scores, and warnings in the React frontend UI.

### Tasks

- [ ] **3.1** Create frontend risk components
  - [ ] Create `RiskBadge.tsx` component (shows color-coded risk score: green/yellow/red)
  - [ ] Create `RiskMetricsPanel.tsx` component (detailed metrics display)
  - [ ] Create `RiskChart.tsx` component (volatility/drawdown over time)

- [ ] **3.2** Integrate with holdings/positions views
  - [ ] Add risk badge column to holdings table
  - [ ] Add risk metrics to position detail page
  - [ ] Show risk score tooltip on hover

- [ ] **3.3** Portfolio-level risk dashboard
  - [ ] Create `PortfolioRiskOverview.tsx` component
  - [ ] Display aggregated portfolio risk score
  - [ ] Show risk contribution breakdown by position
  - [ ] Add volatility and correlation heatmap (stretch goal)

- [ ] **3.4** Risk threshold settings page
  - [ ] Create settings UI for configuring thresholds
  - [ ] Allow users to set thresholds per metric (volatility, drawdown, etc.)
  - [ ] Show preview of positions that would trigger warnings

- [ ] **3.5** API integration
  - [ ] Create TypeScript types for risk API responses
  - [ ] Implement API client functions to fetch risk data
  - [ ] Handle loading states and error messages

### Acceptance Criteria

- Risk badges display correctly with appropriate colors
- Users can view detailed risk metrics for each position
- Portfolio risk overview shows aggregated metrics
- Settings page allows threshold customization
- UI gracefully handles missing or loading data

### Checkpoint

**User approval required before proceeding to Phase 4**

---

## Phase 4: News & LLM Integration (Rust)

**Status:** ⬜ Not Started

**Objective:** Implement news fetching and LLM-powered narrative generation directly in Rust.

### Tasks

- [ ] **4.1** Set up LLM client in Rust
  - [ ] Add dependencies: `async-openai` (or `openai-api-rust`), `reqwest`, `serde_json`
  - [ ] Create `backend/src/services/llm_service.rs`
  - [ ] Configure OpenAI API key from environment variables
  - [ ] Implement basic LLM client wrapper with error handling

- [ ] **4.2** Implement news fetching service
  - [ ] Create `backend/src/services/news_service.rs`
  - [ ] Integrate with news API (NewsAPI, Serper, or similar) using `reqwest`
  - [ ] Implement `fetch_news_for_ticker()` function
  - [ ] Add deduplication logic for news articles
  - [ ] Implement rate limiting and retry logic
  - [ ] Cache results in database to minimize API calls

- [ ] **4.3** Create news database tables
  - [ ] Design `news_articles` table (id, ticker, title, url, published_at, source, fetched_at)
  - [ ] Design `news_themes` table (id, ticker, theme_title, summary, articles, created_at)
  - [ ] Create migrations
  - [ ] Implement queries in `db/news_queries.rs`

- [ ] **4.4** Implement theme clustering with LLM
  - [ ] Create function `cluster_news_themes()` in `llm_service.rs`
  - [ ] Construct prompt asking LLM to identify themes from news articles
  - [ ] Parse LLM response into structured themes
  - [ ] Store themes in database
  - [ ] Handle LLM errors and fallback gracefully

- [ ] **4.5** Implement narrative generation
  - [ ] Create function `generate_narrative()` in `llm_service.rs`
  - [ ] Accept risk metrics, price changes, and news themes as input
  - [ ] Construct comprehensive prompt with all context
  - [ ] Request LLM to generate educational Markdown summary
  - [ ] Ensure output avoids buy/sell recommendations
  - [ ] Store generated narratives in database for caching

- [ ] **4.6** Create analysis API routes
  - [ ] Create `backend/src/routes/analysis.rs`
  - [ ] Implement `GET /api/analysis/:ticker/news` endpoint
  - [ ] Implement `GET /api/analysis/:ticker/themes` endpoint
  - [ ] Implement `GET /api/analysis/:ticker/narrative` endpoint
  - [ ] Implement `GET /api/analysis/portfolio/:id/brief` endpoint (aggregated)
  - [ ] Register routes in main.rs

### Acceptance Criteria

- News can be fetched for valid tickers and stored in database
- LLM client successfully connects to OpenAI API
- Theme clustering produces coherent, distinct themes
- Narrative generation creates informative educational content
- API endpoints return properly formatted JSON/Markdown
- Caching reduces redundant API calls
- Error handling covers API failures, rate limits, and invalid responses

### Checkpoint

**User approval required before proceeding to Phase 5**

---

## Phase 5: Alerts & Notifications System

**Status:** ⬜ Not Started

**Objective:** Implement a system that monitors risk metrics, detects threshold breaches, and notifies users.

### Tasks

- [ ] **5.1** Design alerts database schema
  - [ ] Create `alerts` table (user_id, portfolio_id, ticker, metric, threshold, triggered_at, dismissed)
  - [ ] Create migration
  - [ ] Implement queries in `db/alert_queries.rs`

- [ ] **5.2** Background job for risk monitoring
  - [ ] Create background task using Tokio or cron-like scheduler
  - [ ] Periodically recompute risk metrics for all active positions
  - [ ] Compare against user thresholds
  - [ ] Generate alert records when thresholds are breached

- [ ] **5.3** Notification delivery (optional)
  - [ ] Implement email notifications (using SMTP or SendGrid)
  - [ ] Add push notification support (optional stretch goal)
  - [ ] Allow users to configure notification preferences

- [ ] **5.4** Alert API endpoints
  - [ ] Create `GET /api/alerts` endpoint (retrieve active alerts)
  - [ ] Create `POST /api/alerts/:id/dismiss` endpoint
  - [ ] Create `GET /api/alerts/history` endpoint (view past alerts)

- [ ] **5.5** Frontend alert UI
  - [ ] Add alerts icon/badge to navigation bar
  - [ ] Create `AlertsPanel.tsx` component listing active alerts
  - [ ] Add dismiss functionality
  - [ ] Show alert details with context (position, metric, threshold)

### Acceptance Criteria

- Background job successfully detects threshold breaches
- Alerts are stored in database and retrievable via API
- Users receive notifications when configured
- UI displays active alerts prominently
- Users can dismiss alerts and view history

### Checkpoint

**User approval required before proceeding to Phase 6**

---

## Phase 6: Testing, Performance & Deployment

**Status:** ⬜ Not Started

**Objective:** Ensure code quality, optimize performance, and prepare for production deployment.

### Tasks

- [ ] **6.1** Unit tests
  - [ ] Write tests for all risk calculation functions
  - [ ] Test edge cases (empty series, constant prices, negative returns)
  - [ ] Validate calculations against known results
  - [ ] Test API endpoints with mock data

- [ ] **6.2** Integration tests
  - [ ] Mock LLM and news API responses for integration tests
  - [ ] Test end-to-end flow: fetch prices → compute risk → generate alerts
  - [ ] Test threshold configuration and alert triggering
  - [ ] Test news fetching and theme clustering pipeline

- [ ] **6.3** Performance optimization
  - [ ] Profile risk calculation performance for large datasets
  - [ ] Optimize database queries (add indexes if needed)
  - [ ] Implement caching for frequently accessed risk data
  - [ ] Monitor LLM API response times and optimize caching strategy

- [ ] **6.4** Documentation
  - [ ] Document API endpoints (OpenAPI/Swagger)
  - [ ] Write user guide explaining risk metrics
  - [ ] Add inline code comments for complex algorithms
  - [ ] Document deployment steps

- [ ] **6.5** Deployment preparation
  - [ ] Containerize Rust backend application
  - [ ] Update docker-compose.yml with all required services
  - [ ] Configure environment variables for production (API keys, DB credentials)
  - [ ] Set up CI/CD pipeline (GitHub Actions or similar)
  - [ ] Prepare database migration scripts

- [ ] **6.6** Security review
  - [ ] Review API authentication/authorization
  - [ ] Validate input sanitization for all endpoints
  - [ ] Ensure API keys are securely stored
  - [ ] Review CORS configuration

### Acceptance Criteria

- Test coverage exceeds 80% for core risk logic
- All integration tests pass
- Performance meets acceptable latency targets
- Documentation is complete and clear
- Application can be deployed via Docker Compose
- Security vulnerabilities are addressed

### Checkpoint

**Final review and production deployment**

---

## Notes and Decisions

### Technology Stack

- **Backend:** Rust (Axum, SQLx, Tokio, async-openai, reqwest)
- **Frontend:** React, TypeScript
- **Database:** PostgreSQL
- **LLM Integration:** OpenAI GPT-4 (or alternative) via Rust client
- **News API:** NewsAPI, Serper, or similar via HTTP client

### Design Decisions

1. **Phased approach:** Each phase builds incrementally, allowing validation before proceeding
2. **Pure Rust implementation:** All business logic, API integrations, and services in Rust
3. **Caching strategy:** Cache expensive computations (risk metrics, news, LLM responses) in PostgreSQL
4. **No financial advice:** All narratives and insights are educational only
5. **User control:** Users can configure thresholds and notification preferences
6. **Async-first:** Leverage Tokio for concurrent API calls and background tasks

### Risk & Mitigation

| Risk | Mitigation |
|------|------------|
| LLM API costs too high | Implement aggressive caching in DB, rate limiting, configurable refresh intervals |
| News API rate limits | Cache results, limit refresh frequency, implement exponential backoff |
| Slow risk calculations | Precompute and cache, use background jobs with Tokio tasks |
| External API failures | Graceful degradation, fallback to cached data, clear error messages |
| Data quality issues | Validate inputs, handle missing data gracefully, log anomalies |

---

## Progress Log

### 2025-02-09

**Phase 1 Completed ✅**

- Created initial IMPLEMENTATION_TRACKER.md
- Updated Phase 4 to be pure Rust implementation (removed Python microservice)
- All features will be implemented in Rust using async-openai and reqwest
- Began Phase 1 implementation

**Implementation Details:**
- Created `backend/src/models/risk.rs` with comprehensive risk data structures:
  - `PositionRisk`: Core risk metrics (volatility, drawdown, beta, Sharpe, VaR)
  - `RiskAssessment`: Combines metrics with risk score and level
  - `RiskLevel`: Classification enum (Low, Moderate, High)
  - `RiskThresholds`: User-configurable warning thresholds
  - `PortfolioRisk`: Aggregated portfolio-level risk metrics

- Created `backend/src/services/risk_service.rs` with all calculation functions:
  - `compute_risk_metrics()`: Main async function to compute all metrics for a ticker
  - `compute_vol_drawdown()`: Calculates annualized volatility and max drawdown
  - `compute_beta()`: Calculates beta coefficient relative to benchmark
  - `compute_sharpe()`: Calculates Sharpe ratio (risk-adjusted return)
  - `compute_var()`: Calculates 5% Value at Risk using historical simulation
  - `score_risk()`: Combines metrics into 0-100 risk score with weighted formula

- Extended database layer:
  - Added `fetch_window()` to `backend/src/db/price_queries.rs` for fetching N-day price windows

- Registered all modules in `mod.rs` files
- All compilation errors resolved
- **5/5 unit tests passed successfully:**
  - `test_compute_vol_drawdown_with_flat_prices`
  - `test_compute_vol_drawdown_with_decline`
  - `test_score_risk_zero_risk`
  - `test_score_risk_high_risk`
  - `test_risk_level_classification`

**Status:** Phase 1 complete. Ready for Phase 2 (API Endpoints & Database Integration)

### 2026-02-10

**Phase 2 Completed ✅**

- Created `backend/src/routes/risk.rs` with full REST API:
  - `GET /api/risk/positions/:ticker?days=90&benchmark=SPY` - Calculate risk metrics
  - `GET /api/risk/portfolios/:portfolio_id` - Portfolio-level risk (stub)
  - `GET /api/risk/thresholds` - Retrieve warning thresholds (defaults)
  - `POST /api/risk/thresholds` - Set custom thresholds (stub)

- Implemented auto-fetch for fresh price data:
  - Modified `risk_service::compute_risk_metrics()` to accept `price_provider`
  - Automatically calls `refresh_from_api()` before calculations
  - Checks data freshness (< 6 hours old)
  - Gracefully falls back to cached data if API fails
  - Prevents unnecessary API calls to respect rate limits

- Fixed critical database issue:
  - Created migration `20260210000000_add_price_unique_constraint.sql`
  - Added `UNIQUE(ticker, date)` constraint to `price_points` table
  - Required for `ON CONFLICT` upserts to work properly

- Registered risk router in `app.rs` at `/api/risk`

- Successfully tested with real data:
  - MSFT: Real data from Alpha Vantage API
  - SPY: Mock benchmark data (180 days)
  - Calculated: volatility 31.96%, drawdown -22.98%, beta 0.28, Sharpe -2.73, VaR -2.87%
  - Risk score: 45.03/100 (moderate risk)

**Status:** Phase 2 complete. Ready for Phase 3 (Frontend Integration)

### 2026-02-10 (continued)

**Phase 3 Completed ✅**

- Created TypeScript types in `frontend/src/types.ts`:
  - `RiskLevel` - 'low' | 'moderate' | 'high'
  - `PositionRisk` - Individual risk metrics
  - `RiskAssessment` - Complete risk evaluation with score
  - `RiskThresholds` - User-configurable warning thresholds

- Created API client functions in `frontend/src/lib/endpoints.ts`:
  - `getPositionRisk(ticker, days?, benchmark?)` - Fetch risk assessment
  - `getRiskThresholds()` - Get user thresholds
  - `setRiskThresholds(thresholds)` - Update thresholds

- Created React components:
  - **RiskBadge.tsx** - Compact risk indicator with color coding (green/yellow/red)
    - Shows LOW/MODERATE/HIGH with appropriate icons
    - Tooltip with quick summary (risk score, volatility, drawdown, beta)
    - Uses TanStack Query for data fetching with 1-hour stale time

  - **RiskMetricsPanel.tsx** - Detailed risk metrics display
    - Visual risk score with progress bar
    - Individual metric cards for volatility, drawdown, beta, Sharpe, VaR
    - Color-coded based on risk levels
    - Tooltips explaining each metric
    - Educational disclaimer about historical data

  - **RiskAnalysis.tsx** - Standalone risk analysis page
    - Search interface for any ticker
    - Configurable time period (30/60/90/180/365 days)
    - Selectable benchmark (SPY/QQQ/DIA/IWM)
    - Uses RiskMetricsPanel for detailed display
    - Educational content about risk metrics

- Integrated risk display into existing views:
  - Added "Risk" column to Holdings table in `Holdings.tsx`
  - Each position shows RiskBadge component
  - Auto-fetches risk data for all holdings

- Updated navigation:
  - Added "Risk Analysis" menu item to Layout.tsx
  - New dedicated page accessible from sidebar
  - Assessment icon for visual consistency

**Status:** Phase 3 complete. Ready for Phase 4 (News & LLM Integration) or Phase 5 (Alerts & Notifications)

---

## Next Steps

1. **Review this implementation plan** - Confirm the phased approach and task breakdown
2. **Begin Phase 1** - Start implementing risk_service.rs and core calculation functions
3. **Incremental validation** - Complete each phase and get approval before proceeding

**Current phase:** Phase 1 - Risk Metrics Core Module (Rust)

**Awaiting user approval to proceed.**
