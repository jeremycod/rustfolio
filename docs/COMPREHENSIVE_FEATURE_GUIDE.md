
# Rustfolio Comprehensive Feature Guide

This document provides an extensive overview of all features currently implemented in Rustfolio, based on the codebase, documentation, and UI components.

## Table of Contents
1. [Portfolio and Position Management](#1-portfolio-and-position-management)
2. [Accounts, Holdings and Transactions](#2-accounts-holdings-and-transactions)
3. [Price Data and Market Analytics](#3-price-data-and-market-analytics)
4. [Risk Analysis and Management](#4-risk-analysis-and-management)
5. [Portfolio Optimization](#5-portfolio-optimization)
6. [Sentiment and News Analysis](#6-sentiment-and-news-analysis)
7. [AI-Powered Narratives and Q&A](#7-ai-powered-narratives-and-qa)
8. [Alerts and Notifications](#8-alerts-and-notifications)
9. [Job Scheduler and Administration](#9-job-scheduler-and-administration)
10. [User Interface Features](#10-user-interface-features)

---

## 1. Portfolio and Position Management

### Core Portfolio Operations
**Create portfolios and positions** – Users can create multiple portfolios and add positions by specifying ticker symbol, number of shares, and average buy price. The system supports full CRUD operations on portfolios and positions.

**Update holdings** – Positions can be updated when shares are purchased or sold. The system recalculates cost basis, market value, and profit/loss automatically.

**Delete positions or portfolios** – Individual positions or entire portfolios can be removed. Cascade deletion ensures associated data (analytics, risk metrics) is cleaned up.

**Search tickers** – Integrated ticker search queries market data providers and returns matching symbols with company names, allowing quick discovery of securities to track.

**Portfolio selector** – UI component allows switching between portfolios across all pages, maintaining context as users navigate.

### Position Display Features
**Color-coded gains/losses** – Visual indicators show profit (green) and loss (red) at a glance in holdings tables.

**Market value tracking** – Real-time calculation of position values based on latest prices and share counts.

**Percentage allocation** – Each position shows its weight as a percentage of total portfolio value.

**Asset type classification** – Positions are categorized (stocks, ETFs, bonds, mutual funds) with visual chips and legends.

---

## 2. Accounts, Holdings and Transactions

### Account Management
**Account listing and details** – Users can create and manage multiple accounts per portfolio, representing different brokerage accounts or investment vehicles.

**Account value history** – Daily account values are stored and displayed as time-series charts showing growth over time.

**Account detail view** – Dedicated page shows account-specific holdings, transactions, cash flows, and performance metrics.

### Transaction Tracking
**Automatic transaction detection** – Background service scans account snapshots and infers buy/sell transactions, deposits, and withdrawals automatically.

**Manual transaction entry** – Users can manually add transactions with date, type, ticker, quantity, and price.

**Transaction history** – Complete audit trail of all account activity with filtering and search capabilities.

**True performance calculation** – Time-weighted and money-weighted returns that account for cash flows and transaction timing.

### Cash Flow Management
**Deposit and withdrawal tracking** – Record capital additions and removals with dates and descriptions.

**Cash flow impact analysis** – Understand how deposits/withdrawals affect performance metrics.

**CSV import** – Bulk import transactions and positions from broker statements with automatic parsing.

---

## 3. Price Data and Market Analytics

### Market Data Integration
**Latest price fetching** – Real-time (or near-real-time) price updates from multiple data providers (Alpha Vantage, Twelve Data).

**Historical price storage** – Complete daily price history stored in database for charting and analytics.

**Price update triggers** – Manual and automatic price refresh with configurable intervals.

**Multi-provider support** – Fallback logic across multiple data sources ensures reliability.

### Technical Analysis
**Moving averages** – Simple Moving Average (SMA) and Exponential Moving Average (EMA) calculated and displayed on charts.

**Trendlines** – Linear regression trendlines show price direction over selected periods.

**Bollinger Bands** – Volatility bands help identify overbought/oversold conditions.

**Price history charts** – Interactive charts with zoom, pan, and date range selection showing price evolution with 20-day moving average overlay.

### Portfolio Analytics
**Portfolio value over time** – Aggregate portfolio value charted across custom date ranges.

**Performance metrics** – Total return, annualized return, gain/loss (absolute and percentage).

**Allocation visualization** – Donut charts show portfolio composition by position and asset type.

**Forecast models** – Time-series forecasting using linear regression and exponential smoothing to project future portfolio values.

---

## 4. Risk Analysis and Management

### Individual Position Risk
**Risk metrics per ticker** – Comprehensive risk assessment including:
  - Annualized volatility (standard deviation of returns)
  - Maximum drawdown (peak-to-trough decline)
  - Beta vs benchmark (market sensitivity)
  - Value at Risk (VaR) at 95% confidence
  - Sharpe ratio (risk-adjusted returns)
  - Overall risk score (0-100 scale)
  - Risk classification (Low, Moderate, High, Very High)

**Risk analysis page** – Dedicated view for each ticker with tabbed interface showing:
  - Risk metrics panel with detailed statistics
  - Price history chart with moving averages
  - Risk trends chart (rolling volatility and drawdown)
  - Risk history timeline

**Risk score explanation** – Expandable accordion showing how risk score is calculated with weighted contributions from each metric (volatility 40%, drawdown 30%, beta 20%, VaR 10%).

### Portfolio-Level Risk
**Portfolio risk overview** – Aggregated risk metrics for entire portfolio with position-level breakdown.

**Risk badges** – Visual indicators (color-coded icons) in holdings tables showing risk level at a glance.

**Clickable risk navigation** – Risk badges link directly to detailed risk analysis pages.

**Risk threshold configuration** – Users can set custom thresholds for volatility, drawdown, beta, and VaR.

**Threshold violation alerts** – Visual warnings when positions exceed configured risk limits.

**Position warning preview** – Live preview in threshold settings showing which positions would trigger warnings at current threshold values.

### Advanced Risk Features
**Correlation matrix** – Pairwise correlation calculations between portfolio positions (up to 10 tickers) with caching for performance.

**Correlation heatmap** – Visual matrix showing correlation coefficients with color coding (green for negative, red for positive correlation).

**Correlation statistics** – Summary stats including average correlation and diversification insights.

**Rolling beta analysis** – Beta calculated over 30, 60, and 90-day windows showing how market sensitivity evolves.

**Beta forecasting** – Predictive models (linear regression, exponential smoothing, ensemble) forecast future beta values.

**Rolling beta page** – Dedicated interface for analyzing beta trends with interactive charts.

### Risk History and Tracking
**Risk snapshots** – Manual and automatic capture of risk metrics at points in time for historical comparison.

**Risk history charts** – Time-series visualization of how risk metrics evolved with selectable metrics (risk score, volatility, drawdown, Sharpe, beta).

**Risk alerts** – Automatic detection of significant risk increases with configurable thresholds and lookback periods.

**Alert markers** – Visual indicators on charts showing when risk alerts were triggered.

**Risk comparison tool** – Side-by-side comparison of risk metrics for 2-4 tickers with bar charts, best/worst indicators, and CSV export.

### Risk Reporting
**Risk exports** – Export portfolio risk metrics to CSV for offline analysis and reporting.

**PDF reports** – Generate formatted PDF reports with portfolio summary, holdings table, and risk metrics.

**Downloadable reports** – One-click export from portfolio overview with auto-generated filenames.

---

## 5. Portfolio Optimization

### Concentration Risk Analysis
**Concentration detection** – Automatic identification of positions exceeding 15% of portfolio value with severity levels (warning, high, critical).

**Risk contribution analysis** – Calculate each position's contribution to total portfolio risk (volatility, drawdown, VaR).

**Diversification scoring** – 0-10 scale based on Herfindahl index, position count, and correlation structure.

### Optimization Recommendations
**Actionable suggestions** – Specific recommendations to reduce concentration, rebalance sectors, or improve risk-adjusted returns.

**Position adjustments** – Detailed buy/sell/hold recommendations with target weights and dollar amounts.

**Rationale explanations** – Clear reasoning for each recommendation with educational context.

**Expected impact metrics** – Before/after projections for risk score, volatility, Sharpe ratio, and diversification.

**Severity classification** – Recommendations tagged as Info, Warning, High, or Critical based on urgency.

**Portfolio health assessment** – Overall classification (Excellent, Good, Fair, Poor, Critical) with key findings summary.

### What-If Analysis
**Scenario simulation** – Test potential portfolio changes before executing trades.

**Real-time impact preview** – See projected risk metrics as allocation sliders are adjusted.

**Scenario save/load** – Store and compare multiple optimization scenarios.

---

## 6. Sentiment and News Analysis

### News Integration
**News fetching** – Retrieve recent news articles for portfolio tickers from news providers (Serper API).

**Thematic clustering** – LLM-powered clustering of articles into coherent themes (earnings, regulatory, product launches, etc.).

**Theme cards** – Visual display of news themes with article counts, sentiment scores, and example headlines.

**Portfolio news page** – Aggregated news view showing all themes across portfolio holdings.

### Sentiment Analysis
**Sentiment signals** – Per-ticker sentiment scores derived from news articles with trend indicators (improving/deteriorating).

**Enhanced sentiment** – Combined analysis of news sentiment, SEC filings, and insider trading signals.

**Sentiment badges** – Visual indicators showing bullish, neutral, or bearish sentiment with confidence levels.

**Portfolio sentiment overview** – Aggregated sentiment across all positions with bullish/bearish divergence counts.

**Sentiment dashboard** – Comprehensive view of sentiment signals with filtering and sorting.

**Enhanced sentiment dashboard** – Advanced view combining multiple sentiment sources with divergence detection.

### Sentiment Features
**Divergence detection** – Identify when sentiment conflicts with price momentum (bullish/bearish divergences).

**Insider trading signals** – Track insider buying/selling activity and correlate with sentiment.

**SEC filing sentiment** – Analyze tone and content of regulatory filings.

**Sentiment caching** – Background jobs pre-calculate sentiment to ensure responsive UI.

---

## 7. AI-Powered Narratives and Q&A

### Portfolio Narratives
**AI-generated summaries** – LLM-powered narrative descriptions of portfolio health, risk profile, and areas of concern.

**Narrative caching** – Results cached for performance with refresh-on-demand capability.

**Time period selection** – Generate narratives for different lookback periods (30, 90, 180 days).

**Context-aware storytelling** – Narratives incorporate risk metrics, performance trends, concentration risk, and diversification scores.

### Question & Answer Interface
**Portfolio Q&A** – Ask natural language questions about portfolio performance, risk, and holdings.

**Contextual responses** – AI interprets questions, fetches relevant data, and provides concise answers.

**Example queries** – Pre-populated question suggestions to guide users.

**Conversation history** – Track Q&A interactions for reference.

### AI Configuration
**User preferences** – Toggle LLM consent, specify risk appetite, adjust narrative tone.

**LLM settings page** – Configure AI provider, model, temperature, and max tokens.

**Usage statistics** – Track LLM request counts and costs.

**Consent dialog** – Explicit user consent for AI features with data usage transparency.

**AI badges** – Visual indicators showing when AI-powered features are active.

**AI loading states** – Animated indicators during LLM processing.

**Experimental banner** – Clearly mark AI features as experimental/beta.

---

## 8. Alerts and Notifications

### Alert Rules
**Alert rule creation** – Define custom alert rules with conditions and thresholds.

**Alert types** – Support for price alerts, risk threshold alerts, sentiment change alerts, and portfolio value alerts.

**Alert rule management page** – View, edit, enable/disable, and delete alert rules.

**Alert rule testing** – Test alert conditions before activation with preview of triggered alerts.

**Alert severity levels** – Classify alerts as Info, Low, Medium, High, or Critical.

**Alert type chips** – Visual indicators showing alert category.

### Notification System
**Notification preferences** – Configure delivery channels (email, in-app, SMS) per alert type.

**Notification history page** – View all past notifications with filtering by type, severity, and date.

**Notification cards** – Visual display of notifications with read/unread status.

**Email notifications** – SMTP integration for email delivery with configurable templates.

**Notification preferences section** – Granular control over which alerts trigger notifications.

### Alert History
**Alert history tracking** – Complete audit trail of all triggered alerts.

**Alert history page** – Searchable, filterable view of historical alerts.

**Alert resolution tracking** – Mark alerts as acknowledged or resolved.

---

## 9. Job Scheduler and Administration

### Background Jobs
**Scheduled job execution** – Automated tasks for risk calculation, sentiment analysis, price updates, and forecasts.

**Job monitoring** – View all scheduled jobs with status, last run time, and next run time.

**Job run history** – Detailed logs of job executions with start/end times and outcomes.

**Manual job triggers** – Force immediate execution of any scheduled job for debugging or urgent updates.

**Job statistics** – Average run times, success rates, and error counts per job.

### Cache Management
**Cache health monitoring** – Real-time status of in-memory caches (fresh, stale, calculating, error).

**Cache invalidation** – Manual cache clearing for testing or troubleshooting.

**Cache status indicators** – Visual display of cache health across risk, sentiment, and news systems.

### Admin Tools
**Admin dashboard** – Centralized control panel for system monitoring and management.

**Data reset** – Clear all database tables and caches for testing or demo purposes.

**System health checks** – Monitor database connectivity, external API status, and service health.

**Health endpoint** – `/health` API for monitoring and load balancer integration.

---

## 10. User Interface Features

### Navigation and Layout
**Responsive layout** – Desktop-first design with mobile-ready responsive breakpoints.

**Sidebar navigation** – Persistent menu with icons and labels for all major sections.

**Breadcrumb navigation** – Context-aware breadcrumbs showing current location.

**Page routing** – Client-side routing with deep linking support.

### Dashboard
**Portfolio dashboard** – Overview page with key metrics, recent performance, and quick actions.

**Multi-widget layout** – Modular dashboard with portfolio value, allocation, performance, and risk widgets.

**Quick navigation** – Links to detailed views from dashboard cards.

### Data Visualization
**Interactive charts** – Recharts-powered visualizations with hover tooltips, zoom, and pan.

**Chart types** – Line charts, area charts, bar charts, donut charts, heatmaps, and combo charts.

**Date range selectors** – Filter charts by predefined ranges (1M, 3M, 6M, 1Y, All) or custom dates.

**Chart legends** – Toggle series visibility by clicking legend items.

**Export charts** – Download charts as images or underlying data as CSV.

### UI Components
**Modal dialogs** – Add position, edit position, ticker search, and settings modals.

**Loading states** – Skeleton loaders and spinners during data fetching.

**Error handling** – User-friendly error messages with retry options.

**Toast notifications** – Temporary alerts for success/error feedback.

**Tooltips** – Contextual help text on hover for metrics and controls.

**Badges and chips** – Visual tags for risk levels, sentiment, asset types, and alert severity.

**Progress bars** – Visual indicators for risk score components and loading progress.

**Accordions** – Expandable sections for detailed information without cluttering UI.

**Tabs** – Organize related content (Risk Metrics, Price History, Risk Trends, Risk History).

### Settings and Preferences
**Settings page** – Centralized configuration for user preferences, risk thresholds, and LLM settings.

**Risk threshold settings** – Configure portfolio-specific risk limits with live preview.

**User settings dialog** – Quick access to common preferences from any page.

**Auto-refresh toggle** – Enable/disable automatic data refresh every 60 seconds.

**Theme preferences** – (Planned) Light/dark mode support.

### Data Entry
**Ticker search modal** – Autocomplete search with company name display.

**Form validation** – Client-side validation with helpful error messages.

**Number formatting** – Automatic formatting of currency, percentages, and large numbers.

**Date pickers** – Calendar widgets for selecting dates in forms and filters.

### Accessibility
**Keyboard navigation** – Full keyboard support for all interactive elements.

**ARIA labels** – Screen reader support with semantic HTML and ARIA attributes.

**Color contrast** – Accessible color schemes meeting WCAG guidelines.

**Focus indicators** – Clear visual focus states for keyboard navigation.

---

## Feature Implementation Status

### Fully Implemented ✅
- Portfolio and position management
- Account and transaction tracking
- Price data integration
- Risk analysis (individual and portfolio)
- Risk history and snapshots
- Correlation analysis
- Rolling beta and forecasting
- Portfolio optimization recommendations
- News fetching and thematic clustering
- Sentiment analysis (basic and enhanced)
- AI narratives and Q&A
- Alert rules and notifications
- Job scheduler and admin tools
- Comprehensive UI components

### Partially Implemented 🔄
- CSV import (basic implementation, needs broker-specific parsers)
- Tax reporting (cost basis tracking planned)
- Real-time price updates (polling implemented, WebSocket planned)

### Planned 📋
- User authentication and multi-tenancy
- Watchlists for non-held securities
- Advanced technical indicators (MACD, RSI)
- Machine learning forecasting models
- Tax-loss harvesting suggestions
- Mobile app
- API for third-party integrations

---

## Conclusion

Rustfolio is a feature-rich portfolio management platform that goes far beyond basic tracking. With comprehensive risk analysis, AI-powered insights, sentiment monitoring, optimization recommendations, and a polished user interface, it provides investors with institutional-grade tools for managing their portfolios. The modular architecture and extensive API make it extensible for future enhancements while maintaining production-ready quality today.
