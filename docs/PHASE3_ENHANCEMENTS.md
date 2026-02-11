# Phase 3 Enhancements - Risk Analysis Polish & Extension

**Created:** 2026-02-11
**Status:** Phase 3A & 3B Complete ✅
**Goal:** Polish and extend Phase 3 risk management features before proceeding to Phase 4

## Overview

Phase 3 core features are complete (95%). This document outlines additional enhancements to make the risk analysis system more comprehensive, user-friendly, and valuable.

---

## Priority 1: Quick Wins (1-2 hours each)

### ✅ 1.1 Price History Chart
**Status:** Completed ✅
- Added Price History tab to Risk Analysis page
- Shows price chart with 20-day moving average
- Displays period statistics (high, low, change, drawdown)
- Implemented in `PriceHistoryChart.tsx`

### ✅ 1.2 Company Name Display
**Status:** Completed ✅
- Shows full company name below ticker symbol
- Fetches from ticker search API
- Example: "HOOD Risk Analysis" → "Robinhood Markets, Inc."

### ✅ 1.3 Risk Score Explanation
**Status:** Completed ✅
**Effort:** 1-2 hours

**Objective:** Help users understand WHY a risk score is what it is.

**Implementation:**
- ✅ Added expandable accordion "How is this calculated?" in RiskMetricsPanel
- ✅ Break down risk score formula with visual progress bars:
  - Volatility contribution: points out of 40
  - Drawdown contribution: points out of 30
  - Beta contribution: points out of 20
  - VaR contribution: points out of 10
- ✅ Show exact formula and risk level ranges
- ✅ Color-coded progress bars for each metric

**Files modified:**
- `frontend/src/components/RiskMetricsPanel.tsx`

**Acceptance Criteria:**
- ✅ User can click to see risk score breakdown
- ✅ Clear visualization of component contributions
- ✅ Formula and weighting logic clearly explained

---

### ✅ 1.4 Risk Alerts/Warnings in UI
**Status:** Completed ✅
**Effort:** 2-3 hours

**Objective:** Visual indicators when positions exceed risk thresholds.

**Implementation:**
- ✅ Enhanced RiskBadge component with showLabel and onNavigate props
- ✅ Added Risk column to Portfolio Overview holdings table
- ✅ Icon-only badges (compact for table display)
- ✅ Clickable badges navigate to risk analysis page
- ✅ Color-coded icons (green/yellow/red) for risk levels
- ✅ Tooltips show risk metrics on hover
- ✅ N/A handling for securities without risk data (mutual funds, bonds)

**Files modified:**
- `frontend/src/components/PortfolioOverview.tsx`
- `frontend/src/components/RiskBadge.tsx`

**Acceptance Criteria:**
- ✅ Warning badges visible on holdings tables
- ✅ Clicking badge navigates to risk analysis details
- ✅ Respects user-configured thresholds
- ✅ Clean, compact display in table format

---

### ✅ 1.5 Position Warning Preview in Threshold Settings
**Status:** Completed ✅
**Effort:** 2 hours

**Objective:** Show live preview of which positions would trigger warnings.

**Implementation:**
- ✅ Added "Preview Impact" section to RiskThresholdSettings
- ✅ Portfolio selector dropdown to choose which portfolio to preview
- ✅ Fetches portfolio positions and their risk metrics
- ✅ Calculates which positions exceed each threshold in real-time
- ✅ Shows summary: "X warnings would be triggered across Y positions"
- ✅ Expandable cards for each threshold showing affected tickers
- ✅ Updates immediately as user adjusts threshold values

**Files modified:**
- `frontend/src/components/RiskThresholdSettings.tsx`

**API used:**
- `listPortfolios()` - for portfolio selector
- `getPortfolioRisk()` - for position risk data

**Acceptance Criteria:**
- ✅ Live preview updates as thresholds change
- ✅ Shows count and list of affected positions
- ✅ Helps users calibrate thresholds effectively
- ✅ Expandable lists to see which specific tickers are affected

---

## Priority 2: Medium Effort (3-5 hours each)

### ✅ 2.1 RiskChart Component - Volatility/Drawdown Trends
**Status:** Completed ✅
**Effort:** 3-4 hours

**Objective:** Show how risk metrics evolved over time.

**Implementation:**
- ✅ Created `RiskChart.tsx` component
- ✅ Calculates rolling 30-day volatility windows from price data
- ✅ Chart showing:
  - Volatility trend line (annualized)
  - Drawdown underwater chart (from running peak)
- ✅ Added as third tab to Risk Analysis page: "Risk Metrics | Price History | Risk Trends"
- ✅ Summary cards showing current volatility, average volatility, current drawdown, and max drawdown
- ✅ All calculations done on frontend using existing price data

**Files created:**
- `frontend/src/components/RiskChart.tsx`

**Files modified:**
- `frontend/src/components/RiskAnalysis.tsx`

**Acceptance Criteria:**
- ✅ Chart shows volatility evolution over time
- ✅ Underwater chart clearly shows drawdown periods
- ✅ Users can identify when risk increased/decreased
- ✅ Statistics cards provide quick insight into trends

---

### ✅ 2.2 Risk Comparison Tool
**Status:** Completed ✅
**Effort:** 4-5 hours

**Objective:** Compare risk metrics for multiple tickers side-by-side.

**Implementation:**
- ✅ Created standalone Risk Comparison page
- ✅ Multi-ticker input (add 2-4 tickers)
- ✅ Side-by-side comparison table with tooltips
- ✅ Bar charts for volatility, drawdown, beta, and risk score
- ✅ Color coding: green (low risk), orange (moderate), red (high risk)
- ✅ Best/worst indicators (🏆/⚠️) for each metric
- ✅ CSV export functionality

**Files created:**
- `frontend/src/components/RiskComparison.tsx`

**Files modified:**
- `frontend/src/components/Layout.tsx` - Added menu item
- `frontend/src/App.tsx` - Added route

**Acceptance Criteria:**
- ✅ Can select 2-4 tickers
- ✅ Table shows all metrics side-by-side with tooltips
- ✅ Visual comparison with bar charts
- ✅ Export comparison as CSV
- ✅ Best/worst indicators help identify optimal choices

---

### ✅ 2.3 Enhanced Drawdown Visualization
**Status:** Completed ✅
**Effort:** 3 hours

**Objective:** Make drawdown more tangible with underwater chart.

**Implementation:**
- ✅ Added underwater chart to Price History tab
- ✅ Chart showing:
  - 0% reference line (peak level)
  - Current drawdown from running peak (shaded red area)
  - Drawdown percentage at each point in time
  - Days underwater calculation in alert
- ✅ Enhanced alert showing:
  - Max drawdown date range
  - Duration underwater
  - Risk notice for significant drawdowns

**Files modified:**
- `frontend/src/components/PriceHistoryChart.tsx`

**Acceptance Criteria:**
- ✅ Underwater chart clearly shows drawdown periods
- ✅ Max drawdown highlighted with date range
- ✅ Shows recovery time (days underwater)
- ✅ Visual area chart makes drawdown impact tangible

---

## Priority 3: Larger Features (5+ hours each)

### ✅ 3.1 Historical Risk Tracking
**Status:** Completed ✅
**Effort:** 8 hours
**Completion Date:** 2026-02-10

**Objective:** Track how position risk changes over time.

**Implementation:**
- ✅ Backend: Store daily risk metric snapshots in `risk_snapshots` table
- ✅ Database table with portfolio_id, ticker, snapshot_date, all risk metrics
- ✅ Manual snapshot creation via API endpoint
- ✅ Frontend: RiskHistoryChart component with interactive charts
- ✅ Alerts: Automatic detection of risk increases with threshold configuration
- ✅ Multi-metric display: Toggle between risk score, volatility, drawdown, sharpe, beta
- ✅ Time range selection: 30, 90, 180, 365 days
- ✅ Visual alert markers on chart for risk increases
- ✅ Portfolio and position-level tracking

**Backend files created:**
- `backend/migrations/20260210000001_create_risk_snapshots.sql`
- `backend/src/db/risk_snapshot_queries.rs`
- `backend/src/models/risk_snapshot.rs`
- `backend/src/services/risk_snapshot_service.rs`

**Backend files modified:**
- `backend/src/routes/risk.rs` - Added history, snapshot, and alerts endpoints

**Frontend files created:**
- `frontend/src/components/RiskHistoryChart.tsx` - Advanced chart with metric toggles

**Frontend files modified:**
- `frontend/src/components/RiskAnalysis.tsx` - Added Risk History tab
- `frontend/src/components/PortfolioRiskOverview.tsx` - Added create snapshot button
- `frontend/src/lib/endpoints.ts` - Added getRiskHistory, createRiskSnapshot, getRiskAlerts
- `frontend/src/types.ts` - Added RiskSnapshot and RiskAlert types

**API Endpoints:**
- `POST /api/risk/portfolios/:id/snapshot` - Create snapshots
- `GET /api/risk/portfolios/:id/history?days=90&ticker=AAPL` - Get history
- `GET /api/risk/portfolios/:id/alerts?days=30&threshold=20` - Get alerts

**Acceptance Criteria:**
- ✅ Risk metrics stored daily via manual snapshot creation
- ✅ Historical chart shows trends with multiple selectable metrics
- ✅ Detects risk increases/decreases with configurable thresholds
- ✅ Visual alerts with red dots on chart
- ✅ Summary statistics and alert notifications

---

### ✅ 3.2 Downloadable Risk Reports
**Status:** Completed ✅
**Effort:** 6 hours
**Completion Date:** 2026-02-10

**Objective:** Export risk analysis as PDF/CSV.

**Implementation:**
- ✅ Added export buttons to Portfolio Overview page
- ✅ CSV export with complete holdings and risk data
- ✅ PDF export with formatted report including:
  - Portfolio name and report date
  - Summary metrics (current value, deposits, withdrawals, gain/loss)
  - Holdings table with risk metrics
  - Professional formatting with tables
  - Pagination and footer
- ✅ Uses jsPDF + jspdf-autotable for PDF generation
- ✅ Fetches risk data for all positions in parallel
- ✅ Graceful handling of securities without risk data (mutual funds, bonds)
- ✅ Auto-generated filename with timestamp

**Files created:**
- `frontend/src/lib/exportUtils.ts` - Export utility functions for CSV and PDF

**Files modified:**
- `frontend/src/components/PortfolioOverview.tsx` - Added export functionality

**Acceptance Criteria:**
- ✅ PDF includes all risk data in formatted tables
- ✅ CSV export for spreadsheet analysis
- ✅ Professional formatting with branded footer
- ✅ Handles missing risk data gracefully

---

### 🔄 3.3 Portfolio Optimization Suggestions
**Status:** Planned
**Effort:** 10+ hours

**Objective:** AI/rule-based portfolio rebalancing suggestions.

**Implementation:**
- Analyze portfolio composition
- Identify concentration risk
- Suggest rebalancing: "Reduce TSLA by 10% to lower volatility"
- "What-if" calculator: Preview risk impact of changes
- Monte Carlo simulation for optimization

**Complexity:** High - requires optimization algorithms

**Acceptance Criteria:**
- Actionable rebalancing suggestions
- What-if calculator works
- Explanations for recommendations

---

### 🔄 3.4 Correlation Heatmap
**Status:** Planned
**Effort:** 6-8 hours

**Objective:** Visual matrix showing position correlations.

**Implementation:**
- Calculate correlation matrix for all portfolio positions
- Heatmap visualization (green = negative correlation, red = high correlation)
- Identify diversification opportunities
- Highlight correlated pairs

**Backend needs:**
- Correlation calculation in Rust
- Endpoint: `GET /api/risk/portfolios/:id/correlations`

**Acceptance Criteria:**
- Heatmap clearly shows correlations
- Identifies highly correlated pairs
- Helps with diversification decisions

---

## Implementation Plan

### ✅ Phase 3A: Quick Wins - COMPLETED
1. ✅ Price History Chart
2. ✅ Company Name Display
3. ✅ Risk Score Explanation
4. ✅ Risk Alerts/Warnings in UI
5. ✅ Position Warning Preview in Threshold Settings

**Total Time Spent:** ~6 hours
**Completion Date:** 2026-02-09

### ✅ Phase 3B: Medium Effort - COMPLETED
1. ✅ RiskChart Component (Volatility/Drawdown Trends)
2. ✅ Risk Comparison Tool
3. ✅ Enhanced Drawdown Visualization

**Total Time Spent:** ~10 hours
**Completion Date:** 2026-02-09

### Phase 3C: Larger Features (In Progress)
- ✅ Historical Risk Tracking (Completed)
- ✅ Downloadable Reports (Completed)
- 🔄 Portfolio Optimization (Planned)
- 🔄 Correlation Heatmap (Planned)

**Status:** 2 of 4 features completed

---

## Success Metrics

**Phase 3A Success:** ✅
- ✅ Users understand their risk scores
- ✅ Visual warnings highlight high-risk positions
- ✅ Threshold settings show immediate feedback

**Phase 3B Success:** ✅
- ✅ Users can see risk trends over time
- ✅ Comparison tool aids research decisions
- ✅ Drawdown visualization makes risk tangible

**Phase 3C Success:**
- Historical tracking detects risk changes early
- Reports enable stakeholder communication
- Optimization suggestions improve portfolios

---

## Next Steps

1. ✅ Document plan (this file)
2. ✅ Update IMPLEMENTATION_TRACKER.md with current status
3. ✅ Complete Phase 3A implementation (Quick Wins)
4. ✅ Complete Phase 3B implementation (Medium Effort)
5. 🔄 Complete Phase 3C implementation (Larger Features - In Progress)
   - ✅ Historical Risk Tracking
   - ✅ Downloadable Reports
   - 🔄 Portfolio Optimization Suggestions (Next)
   - 🔄 Correlation Heatmap

**Current Status:** Phase 3C: 2 of 4 features completed.
**Next Feature:** Portfolio Optimization Suggestions (10+ hours) or Correlation Heatmap (6-8 hours)

**Recommendation:** Phase 3 now has 11 enhancements completed (3A + 3B + 2 from 3C). The risk analysis system is comprehensive and production-ready. Consider moving to Phase 4 (News & LLM Integration) or Phase 5 (Alerts & Notifications) for new feature categories, or continue with remaining Phase 3C features if needed.

---

## Notes

- All enhancements build on existing Phase 3 infrastructure
- No breaking changes to current functionality
- Focus on user value and educational aspects
- Maintain consistency with existing UI/UX patterns
