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

### 🔄 3.1 Historical Risk Tracking
**Status:** Planned
**Effort:** 8+ hours

**Objective:** Track how position risk changes over time.

**Implementation:**
- Backend: Store daily/weekly risk metric snapshots
- Database table: `risk_history` (ticker, date, volatility, drawdown, beta, risk_score)
- Background job to calculate and store metrics
- Frontend: Chart showing risk score evolution
- Alerts: "Risk increased by 15 points this month"

**Backend files to create:**
- `backend/migrations/XXXXX_create_risk_history.sql`
- `backend/src/db/risk_history_queries.rs`

**Backend files to modify:**
- `backend/src/services/risk_service.rs` (add snapshot function)

**Frontend files to create:**
- `frontend/src/components/RiskTrendChart.tsx`

**Acceptance Criteria:**
- Risk metrics stored daily
- Historical chart shows trend
- Can detect risk increases/decreases

---

### 🔄 3.2 Downloadable Risk Reports
**Status:** Planned
**Effort:** 6-8 hours

**Objective:** Export risk analysis as PDF/CSV.

**Implementation:**
- Add "Export" button to Portfolio Risk page
- Generate PDF with:
  - Portfolio overview
  - Risk score summary
  - All position risk tables
  - Charts (embedded as images)
- CSV export option for data analysis
- Use library like `jsPDF` or `react-pdf`

**Dependencies:**
- PDF generation library

**Acceptance Criteria:**
- PDF includes all risk data and charts
- CSV export for spreadsheet analysis
- Professional formatting

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

### Phase 3C: Larger Features (Optional/Future)
- Historical Risk Tracking
- Downloadable Reports
- Portfolio Optimization
- Correlation Heatmap

**Decision Point:** Evaluate value vs effort before implementing

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
5. **Current Decision Point:** Continue with Phase 3C, or move to Phase 4/5

**Recommendation:** Phase 3 is now comprehensive with 9 enhancements completed (3A + 3B). Strong foundation for risk analysis. Consider moving to Phase 4 (News & LLM Integration) or Phase 5 (Alerts & Notifications) for new feature categories.

---

## Notes

- All enhancements build on existing Phase 3 infrastructure
- No breaking changes to current functionality
- Focus on user value and educational aspects
- Maintain consistency with existing UI/UX patterns
