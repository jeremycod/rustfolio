# Development Plan for Rustfolio Enhancements

This document outlines a comprehensive plan to extend Rustfolio with risk‐assessment features, AI‑driven narrative analytics and news summarization. It draws on the architecture of both rustfolio and the stock_research_sidekick prototype. Citations refer to the existing codebase to justify design decisions.

## Project Goals

1. **Per‑position risk metrics and warnings** – Compute volatility, drawdown, beta and other risk scores for each position. Flag positions that exceed user‑defined thresholds and present the metrics in the UI.

2. **Portfolio‑level risk analytics** – Aggregate position metrics to compute portfolio volatility and risk contribution. Provide a risk score and warnings at the portfolio level.

3. **Narrative analytics** – Use a large language model (LLM) to generate natural‑language summaries explaining recent performance, highlighting risk factors and suggesting areas of further research.

4. **News aggregation and theme analysis** – Fetch recent news for each holding, cluster articles into themes using an LLM and generate concise summaries with citations, similar to the stock sidekick.

5. **User alerts and custom thresholds** – Allow users to configure thresholds for risk metrics (e.g., volatility > 20 %, drawdown > −10 %), and send notifications when positions or the portfolio cross these thresholds.

6. **Modular architecture** – Keep the Rust back end focused on data storage, basic analytics and API endpoints. Introduce a Python microservice (or integrate into the existing sidekick) to handle news fetching and LLM tasks. Communicate between services using HTTP or gRPC.

## Phase 1 – Risk Metrics Module (Rust)

### 1.1 Data requirements

- **Price history per position** – The existing price_service fetches daily history from Alpha Vantage and stores it in Postgres[1]. Extend the DB queries to fetch price series for an arbitrary ticker and time window.

- **Benchmark index data** – For beta and correlation, select a market index (e.g., S&P 500 ETF SPY) and fetch its history in the same manner.

### 1.2 Risk metrics functions

Create a new module `backend/src/services/risk_service.rs` to implement functions such as:

```rust
pub struct PositionRisk {
    pub volatility: f64,      // standard deviation of daily returns (%), annualized
    pub max_drawdown: f64,    // maximum peak‑to‑trough decline (%), negative number
    pub beta: Option<f64>,    // correlation with benchmark, scaled to variance
    pub sharpe: Option<f64>,  // risk‑adjusted return, using risk‑free rate
    pub value_at_risk: Option<f64>, // 5% VaR over 1‑day horizon
}

pub async fn compute_risk_metrics(
    pool: &PgPool, 
    ticker: &str, 
    days: usize, 
    benchmark: &str
) -> Result<PositionRisk, AppError> {
    // fetch price history for ticker and benchmark
    let series = db::price_queries::fetch_window(pool, ticker, days).await?;
    let bench  = db::price_queries::fetch_window(pool, benchmark, days).await?;
    
    // compute daily returns and statistics
    let (volatility, max_dd) = compute_vol_drawdown(&series);
    let beta = compute_beta(&series, &bench);
    let sharpe = compute_sharpe(&series);
    let var = compute_var(&series);
    
    Ok(PositionRisk { 
        volatility, 
        max_drawdown: max_dd, 
        beta, 
        sharpe, 
        value_at_risk: var 
    })
}
```

Implement helper functions:

- **compute_vol_drawdown** – replicate the volatility and drawdown logic used in the sidekick: sort the series, calculate daily returns and their variance, and compute max drawdown[2].
- **compute_beta** – compute covariance of the asset and benchmark returns divided by the benchmark variance.
- **compute_sharpe** – average daily return minus risk‑free rate divided by volatility (annualise both to 252 trading days).
- **compute_var** – estimate 5 % VaR using historical simulation: sort returns and pick the 5th percentile.

### 1.3 Risk scoring and warning logic

Add a function `score_risk(risk: &PositionRisk) -> f64` that combines the metrics into a single score (0–100). Example weighting:

- 40 % volatility (scaled to a typical range)
- 30 % drawdown severity
- 20 % beta magnitude
- 10 % VaR

Define default thresholds for warnings (e.g., score > 70 → "high risk", 40–70 → "moderate risk"). Allow users to override these thresholds through API settings.

## Phase 2 – API and Database Integration

- **New API endpoints** – Implement routes under `/api/risk`:
  - `GET /api/risk/positions/:id` → returns PositionRisk and risk score for a position.
  - `GET /api/risk/portfolios/:portfolio_id` → returns aggregated risk metrics for the portfolio and a breakdown by position.
  - `POST /api/risk/thresholds` → allows users to set custom thresholds per metric.

- **State and models** – Create new structs in `backend/src/models/risk.rs` to serialize risk metrics and scores. Update AppState if necessary to include benchmark configuration.

- **Database** – Add queries in `db::price_queries` to fetch price history for a given ticker over a rolling window (e.g., 90 days). Optionally persist computed risk metrics for caching.

- **UI integration** – In the React front end, extend the holdings table to show risk score badges (e.g., green/yellow/red). Add a detail view with metrics and graphs. Provide a settings page for thresholds.

## Phase 3 – News Aggregation and LLM Summaries (Python Microservice)

### 3.1 Microservice architecture

Create a separate Python service (could live in stock_research_sidekick repo) exposing HTTP endpoints:

- `/news/:ticker?days=30&limit=20` – fetch recent news via Serper or another API.
- `/summarize` – accept an array of news items and return clustered themes and summaries, using the existing LangGraph nodes cluster_themes[3] and summarize_themes[4].
- `/narrative` – accept price features, risk metrics and theme summaries and return a Markdown brief (similar to compose_brief.py[5]).

Implement caching to avoid repeated API calls. Use Python dependencies from the sidekick's pyproject.toml for Serper and OpenAI.

### 3.2 Integration with Rustfolio

Add asynchronous HTTP client code in Rust to call the Python microservice. This can be part of the new external module (similar to AlphaVantageProvider).

When the user views a position or portfolio detail page, call the microservice to fetch news themes and narrative summaries. Display them in a collapsible panel.

## Phase 4 – Alerts and Notifications

- **Background job** – Use tokio or a task scheduler to run periodic jobs that recompute risk metrics and fetch news for all positions.

- **Notification service** – When a metric exceeds a user's threshold or a negative news theme emerges, store an alert in the database and optionally send an e‑mail/push notification.

- **UI** – Add an alerts icon and page where the user can view active warnings and dismiss them.

## Phase 5 – QA, Testing and Deployment

- **Unit tests** – Implement tests for the new risk calculation functions, including edge cases (empty series, constant prices). Use historical fixtures to validate volatility and drawdown calculations against known results.

- **Integration tests** – Mock the Python microservice to test the API endpoints without external API calls. Ensure that risk scores are correctly computed and warnings are triggered.

- **Performance considerations** – Risk calculations are lightweight, but news summarization involves external API calls. Cache results and precompute where possible.

- **Deployment** – Containerize the Python microservice. Update docker-compose.yml (if used) to include the new service. Expose environment variables for API keys and thresholds.

## Proposed Code Additions

### Rust risk service (risk_service.rs)

```rust
// backend/src/services/risk_service.rs
use crate::db;
use crate::errors::AppError;
use sqlx::PgPool;

pub struct ReturnSeries {
    pub returns: Vec<f64>,
}

pub struct PositionRisk {
    pub volatility: f64,
    pub max_drawdown: f64,
    pub beta: Option<f64>,
    pub sharpe: Option<f64>,
    pub value_at_risk: Option<f64>,
}

pub async fn compute_risk_metrics(
    pool: &PgPool,
    ticker: &str,
    days: usize,
    benchmark: &str,
) -> Result<PositionRisk, AppError> {
    let series = db::price_queries::fetch_window(pool, ticker, days).await?;
    let bench = db::price_queries::fetch_window(pool, benchmark, days).await?;
    
    let (vol, max_dd) = compute_vol_drawdown(&series);
    let beta = compute_beta(&series, &bench);
    let sharpe = compute_sharpe(&series);
    let var = compute_var(&series);
    
    Ok(PositionRisk {
        volatility: vol,
        max_drawdown: max_dd,
        beta,
        sharpe,
        value_at_risk: var,
    })
}

fn compute_vol_drawdown(series: &[db::PricePoint]) -> (f64, f64) {
    // convert to sorted returns
    let mut sorted = series.to_vec();
    sorted.sort_by_key(|p| p.date);
    
    let mut returns = Vec::new();
    for i in 1..sorted.len() {
        let prev = sorted[i-1].close;
        let cur = sorted[i].close;
        if prev > 0.0 {
            returns.push((cur - prev) / prev);
        }
    }
    
    // variance and volatility (annualised)
    let mean = returns.iter().copied().sum::<f64>() / returns.len() as f64;
    let var: f64 = returns.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / (returns.len() as f64 - 1.0);
    let volatility = var.sqrt() * (252.0_f64).sqrt();
    
    // max drawdown
    let mut peak = sorted[0].close;
    let mut max_dd = 0.0;
    for p in &sorted {
        if p.close > peak { peak = p.close; }
        let dd = (p.close - peak) / peak;
        if dd < max_dd { max_dd = dd; }
    }
    
    (volatility * 100.0, max_dd * 100.0)
}

fn compute_beta(series: &[db::PricePoint], bench: &[db::PricePoint]) -> Option<f64> {
    if series.len() != bench.len() || series.len() < 2 { return None; }
    
    // compute daily returns
    let returns: Vec<f64> = series.windows(2).map(|w| (w[1].close - w[0].close) / w[0].close).collect();
    let bench_returns: Vec<f64> = bench.windows(2).map(|w| (w[1].close - w[0].close) / w[0].close).collect();
    
    let mean_r = returns.iter().sum::<f64>() / returns.len() as f64;
    let mean_b = bench_returns.iter().sum::<f64>() / bench_returns.len() as f64;
    
    let mut cov = 0.0;
    let mut var_b = 0.0;
    for (r, b) in returns.iter().zip(bench_returns.iter()) {
        cov += (r - mean_r) * (b - mean_b);
        var_b += (b - mean_b).powi(2);
    }
    var_b = var_b.max(f64::EPSILON);
    Some(cov / var_b)
}

fn compute_sharpe(series: &[db::PricePoint]) -> Option<f64> {
    if series.len() < 2 { return None; }
    
    let mut returns = Vec::new();
    for i in 1..series.len() {
        let prev = series[i-1].close;
        let cur = series[i].close;
        if prev > 0.0 { returns.push((cur - prev) / prev); }
    }
    
    let mean = returns.iter().sum::<f64>() / returns.len() as f64;
    let var: f64 = returns.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / (returns.len() as f64 - 1.0);
    let volatility = var.sqrt() * 252.0_f64.sqrt();
    let risk_free = 0.02 / 252.0; // 2% annual risk‑free rate
    
    Some(((mean - risk_free) * 252.0) / volatility)
}

fn compute_var(series: &[db::PricePoint]) -> Option<f64> {
    if series.len() < 2 { return None; }
    
    let mut returns = Vec::new();
    for i in 1..series.len() {
        let prev = series[i-1].close;
        let cur = series[i].close;
        if prev > 0.0 { returns.push((cur - prev) / prev); }
    }
    
    returns.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let idx = (returns.len() as f64 * 0.05).floor() as usize;
    Some(returns[idx] * 100.0)
}
```

### API route skeleton (risk.rs)

```rust
// backend/src/routes/risk.rs
use axum::{Router, Json, extract::{Path, State}};
use axum::routing::get;
use crate::{services::risk_service, models::{RiskResponse, PortfolioRiskResponse}};
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/positions/:ticker", get(get_position_risk))
        .route("/portfolios/:portfolio_id", get(get_portfolio_risk))
}

async fn get_position_risk(
    Path(ticker): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<RiskResponse>, AppError> {
    let metrics = risk_service::compute_risk_metrics(&state.pool, &ticker, 90, "SPY").await?;
    let score = risk_service::score_risk(&metrics);
    Ok(Json(RiskResponse { ticker, metrics, score }))
}

async fn get_portfolio_risk(
    Path(portfolio_id): Path<Uuid>,
    State(state): State<AppState>,
) -> Result<Json<PortfolioRiskResponse>, AppError> {
    // fetch positions for portfolio and aggregate risk
    // omitted for brevity
    todo!()
}
```

### Python summarization microservice (conceptual)

```python
# app.py
from fastapi import FastAPI
from pydantic import BaseModel
from stock_sidekick.nodes import cluster_themes, summarize_themes, plan_research, compose_brief, compute_price_features

app = FastAPI()

class NewsRequest(BaseModel):
    ticker: str
    days: int = 30
    limit: int = 20

# ... implement routes to fetch news via Serper and call LangGraph nodes

@app.post("/summarize")
async def summarize(items: list[dict]):
    # call cluster_themes and summarize_themes
    # return JSON with themes and summaries
    pass

@app.post("/narrative")
async def narrative(req: dict):
    # accept price_features and theme_summaries, call compose_brief
    pass
```

This skeleton re‑uses the existing stock_sidekick functions, providing a clear interface for the Rust back end.

## Conclusion

By following this phased plan, Rustfolio can evolve into a comprehensive portfolio assistant that not only tracks positions but also evaluates risk, fetches and summarises news, and provides personalised narrative insights to the user. The provided code skeletons serve as a starting point for implementation.

## References

[1] [prices.rs](https://github.com/jeremycod/rustfolio/blob/main/backend/src/routes/prices.rs)

[2] [compute_price_features.py](https://github.com/jeremycod/stock_research_sidekick/blob/main/src/stock_sidekick/nodes/compute_price_features.py)

[3] [cluster_themes.py](https://github.com/jeremycod/stock_research_sidekick/blob/main/src/stock_sidekick/nodes/cluster_themes.py)

[4] [summarize_themes.py](https://github.com/jeremycod/stock_research_sidekick/blob/main/src/stock_sidekick/nodes/summarize_themes.py)

[5] [compose_brief.py](https://github.com/jeremycod/stock_research_sidekick/blob/main/src/stock_sidekick/nodes/compose_brief.py)
