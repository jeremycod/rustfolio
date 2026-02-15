use axum::extract::{Path, Query, State};
use axum::{Json, Router};
use axum::routing::{get, post};
use axum::response::Response;
use axum::http::{header, StatusCode};
use serde::Deserialize;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::{RiskAssessment, CorrelationMatrix, CorrelationPair, RiskSnapshot, RiskAlert, RiskHistoryParams, AlertQueryParams};
use crate::models::risk::{RiskThresholdSettings, UpdateRiskThresholds, PortfolioRiskWithViolations, ThresholdViolation, ViolationSeverity};
use crate::services::{risk_service, risk_snapshot_service};
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/positions/:ticker", get(get_position_risk))
        .route("/portfolios/:portfolio_id", get(get_portfolio_risk))
        .route("/portfolios/:portfolio_id/correlations", get(get_portfolio_correlations))
        .route("/portfolios/:portfolio_id/snapshot", post(create_portfolio_snapshot))
        .route("/portfolios/:portfolio_id/history", get(get_risk_history))
        .route("/portfolios/:portfolio_id/alerts", get(get_risk_alerts))
        .route("/portfolios/:portfolio_id/thresholds", get(get_thresholds))
        .route("/portfolios/:portfolio_id/thresholds", post(set_thresholds))
        .route("/portfolios/:portfolio_id/export/csv", get(export_portfolio_risk_csv))
}

/// Query parameters for risk calculation
#[derive(Debug, Deserialize)]
pub struct RiskQueryParams {
    /// Number of days for the rolling window (default: 90)
    #[serde(default = "default_days")]
    pub days: i64,

    /// Benchmark ticker for beta calculation (default: "SPY")
    #[serde(default = "default_benchmark")]
    pub benchmark: String,
}

fn default_days() -> i64 {
    90
}

fn default_benchmark() -> String {
    "SPY".to_string()
}

/// GET /api/risk/positions/:ticker
///
/// Calculate and return risk metrics for a specific ticker.
///
/// Query parameters:
/// - `days`: Rolling window in days (default: 90)
/// - `benchmark`: Benchmark ticker for beta (default: "SPY")
///
/// Example: GET /api/risk/positions/AAPL?days=60&benchmark=SPY
#[axum::debug_handler]
pub async fn get_position_risk(
    Path(ticker): Path<String>,
    Query(params): Query<RiskQueryParams>,
    State(state): State<AppState>,
) -> Result<Json<RiskAssessment>, AppError> {
    info!(
        "GET /api/risk/positions/{} - Computing risk metrics (days={}, benchmark={})",
        ticker, params.days, params.benchmark
    );

    let risk_assessment = risk_service::compute_risk_metrics(
        &state.pool,
        &ticker,
        params.days,
        &params.benchmark,
        state.price_provider.as_ref(),
        &state.failure_cache,
        state.risk_free_rate,
    )
    .await
    .map_err(|e| {
        // Log with detailed context for debugging
        match &e {
            AppError::External(msg) if msg.contains("failure cache") => {
                info!("⚠️  Ticker {} in failure cache, skipping API call: {}", ticker, msg);
            }
            AppError::External(msg) if msg.contains("No price data") => {
                warn!("📊 No price data available for {}: {}", ticker, msg);
            }
            AppError::NotFound => {
                warn!("🔍 Ticker {} not found in database or API", ticker);
            }
            AppError::RateLimited => {
                warn!("⏳ Rate limited when fetching {}", ticker);
            }
            _ => {
                error!("❌ Failed to compute risk metrics for {}: {:?}", ticker, e);
            }
        }
        e
    })?;

    Ok(Json(risk_assessment))
}

/// GET /api/risk/portfolios/:portfolio_id
///
/// Calculate aggregated risk metrics for a portfolio.
///
/// Query parameters:
/// - `days`: Rolling window in days (default: 90)
/// - `benchmark`: Benchmark ticker for beta (default: "SPY")
///
/// Example: GET /api/risk/portfolios/{uuid}?days=60
pub async fn get_portfolio_risk(
    Path(portfolio_id): Path<Uuid>,
    Query(params): Query<RiskQueryParams>,
    State(state): State<AppState>,
) -> Result<Json<PortfolioRiskWithViolations>, AppError> {
    use crate::db::holding_snapshot_queries;
    use crate::models::PositionRiskContribution;
    use std::collections::HashMap;

    info!(
        "GET /api/risk/portfolios/{} - Computing portfolio risk (days={}, benchmark={})",
        portfolio_id, params.days, params.benchmark
    );

    // 1. Fetch all latest holdings for the portfolio
    let holdings = holding_snapshot_queries::fetch_portfolio_latest_holdings(
        &state.pool,
        portfolio_id
    ).await.map_err(|e| {
        error!("Failed to fetch portfolio holdings: {}", e);
        AppError::Db(e)
    })?;

    // 2. Aggregate holdings by ticker (same ticker across multiple accounts)
    let mut ticker_aggregates: HashMap<String, (f64, f64)> = HashMap::new(); // (quantity, market_value)

    for holding in &holdings {
        let market_value = holding.market_value.to_string().parse::<f64>().unwrap_or(0.0);
        let quantity = holding.quantity.to_string().parse::<f64>().unwrap_or(0.0);

        ticker_aggregates
            .entry(holding.ticker.clone())
            .and_modify(|(q, mv)| {
                *q += quantity;
                *mv += market_value;
            })
            .or_insert((quantity, market_value));
    }

    // Calculate total portfolio value
    let total_value: f64 = ticker_aggregates.values().map(|(_, mv)| mv).sum();

    if total_value == 0.0 {
        return Err(AppError::External(
            "Portfolio has no holdings with market value".to_string()
        ));
    }

    // 3. Compute risk metrics for each ticker and collect contributions
    let mut position_risks = Vec::new();
    let mut weighted_volatility = 0.0;
    let mut weighted_max_drawdown = 0.0;
    let mut weighted_beta = 0.0;
    let mut weighted_sharpe = 0.0;
    let mut beta_count = 0;
    let mut sharpe_count = 0;

    for (ticker, (_quantity, market_value)) in ticker_aggregates {
        // Skip positions with negligible value (< 0.1% of portfolio)
        let weight = market_value / total_value;
        if weight < 0.001 {
            continue;
        }

        // Compute risk metrics for this ticker
        match risk_service::compute_risk_metrics(
            &state.pool,
            &ticker,
            params.days,
            &params.benchmark,
            state.price_provider.as_ref(),
            &state.failure_cache,
            state.risk_free_rate,
        ).await {
            Ok(assessment) => {
                // Weight metrics by position size
                weighted_volatility += assessment.metrics.volatility * weight;
                weighted_max_drawdown += assessment.metrics.max_drawdown * weight;

                if let Some(beta) = assessment.metrics.beta {
                    weighted_beta += beta * weight;
                    beta_count += 1;
                }

                if let Some(sharpe) = assessment.metrics.sharpe {
                    weighted_sharpe += sharpe * weight;
                    sharpe_count += 1;
                }

                position_risks.push(PositionRiskContribution {
                    ticker: ticker.clone(),
                    market_value,
                    weight,
                    risk_assessment: assessment,
                });
            },
            Err(e) => {
                // Log but don't fail - some positions might not have risk data
                warn!("Could not compute risk for {} in portfolio: {}", ticker, e);
            }
        }
    }

    if position_risks.is_empty() {
        return Err(AppError::External(
            "No positions in portfolio have available risk data".to_string()
        ));
    }

    // 4. Calculate portfolio-level risk score
    let portfolio_risk_score = risk_service::score_risk(&crate::models::PositionRisk {
        volatility: weighted_volatility,
        max_drawdown: weighted_max_drawdown,
        beta: if beta_count > 0 { Some(weighted_beta) } else { None },
        beta_spy: if beta_count > 0 { Some(weighted_beta) } else { None },
        beta_qqq: None,
        beta_iwm: None,
        risk_decomposition: None,
        sharpe: if sharpe_count > 0 { Some(weighted_sharpe) } else { None },
        sortino: None,
        annualized_return: None,
        value_at_risk: None, // VaR not meaningful at portfolio level without correlations
        var_95: None,
        var_99: None,
        expected_shortfall_95: None,
        expected_shortfall_99: None,
    });

    let risk_level = crate::models::RiskLevel::from_score(portfolio_risk_score);

    // 5. Sort positions by risk contribution (highest to lowest)
    position_risks.sort_by(|a, b| {
        b.risk_assessment.risk_score.partial_cmp(&a.risk_assessment.risk_score).unwrap()
    });

    let portfolio_risk = crate::models::PortfolioRisk {
        portfolio_id: portfolio_id.to_string(),
        total_value,
        portfolio_volatility: weighted_volatility,
        portfolio_max_drawdown: weighted_max_drawdown,
        portfolio_beta: if beta_count > 0 { Some(weighted_beta) } else { None },
        portfolio_sharpe: if sharpe_count > 0 { Some(weighted_sharpe) } else { None },
        portfolio_risk_score,
        risk_level,
        position_risks: position_risks.clone(),
    };

    // Fetch risk thresholds
    let thresholds = crate::db::risk_threshold_queries::get_thresholds(&state.pool, portfolio_id)
        .await
        .map_err(|e| {
            error!("Failed to fetch risk thresholds: {}", e);
            AppError::Db(e)
        })?;

    // Detect threshold violations
    let violations = detect_violations(&portfolio_risk, &thresholds);

    info!(
        "Portfolio {} has {} threshold violations",
        portfolio_id,
        violations.len()
    );

    Ok(Json(PortfolioRiskWithViolations {
        portfolio_risk,
        thresholds,
        violations,
    }))
}

/// Detect threshold violations in portfolio risk data
fn detect_violations(
    portfolio_risk: &crate::models::PortfolioRisk,
    thresholds: &RiskThresholdSettings,
) -> Vec<ThresholdViolation> {
    let mut violations = Vec::new();

    // Check each position for violations
    for position in &portfolio_risk.position_risks {
        let metrics = &position.risk_assessment.metrics;

        // Check volatility
        if metrics.volatility >= thresholds.volatility_critical_threshold {
            violations.push(ThresholdViolation {
                ticker: position.ticker.clone(),
                holding_name: None,
                metric_name: "Volatility".to_string(),
                metric_value: metrics.volatility,
                threshold_value: thresholds.volatility_critical_threshold,
                threshold_type: ViolationSeverity::Critical,
            });
        } else if metrics.volatility >= thresholds.volatility_warning_threshold {
            violations.push(ThresholdViolation {
                ticker: position.ticker.clone(),
                holding_name: None,
                metric_name: "Volatility".to_string(),
                metric_value: metrics.volatility,
                threshold_value: thresholds.volatility_warning_threshold,
                threshold_type: ViolationSeverity::Warning,
            });
        }

        // Check max drawdown (more negative is worse)
        if metrics.max_drawdown <= thresholds.drawdown_critical_threshold {
            violations.push(ThresholdViolation {
                ticker: position.ticker.clone(),
                holding_name: None,
                metric_name: "Max Drawdown".to_string(),
                metric_value: metrics.max_drawdown,
                threshold_value: thresholds.drawdown_critical_threshold,
                threshold_type: ViolationSeverity::Critical,
            });
        } else if metrics.max_drawdown <= thresholds.drawdown_warning_threshold {
            violations.push(ThresholdViolation {
                ticker: position.ticker.clone(),
                holding_name: None,
                metric_name: "Max Drawdown".to_string(),
                metric_value: metrics.max_drawdown,
                threshold_value: thresholds.drawdown_warning_threshold,
                threshold_type: ViolationSeverity::Warning,
            });
        }

        // Check beta
        if let Some(beta) = metrics.beta {
            if beta >= thresholds.beta_critical_threshold {
                violations.push(ThresholdViolation {
                    ticker: position.ticker.clone(),
                    holding_name: None,
                    metric_name: "Beta".to_string(),
                    metric_value: beta,
                    threshold_value: thresholds.beta_critical_threshold,
                    threshold_type: ViolationSeverity::Critical,
                });
            } else if beta >= thresholds.beta_warning_threshold {
                violations.push(ThresholdViolation {
                    ticker: position.ticker.clone(),
                    holding_name: None,
                    metric_name: "Beta".to_string(),
                    metric_value: beta,
                    threshold_value: thresholds.beta_warning_threshold,
                    threshold_type: ViolationSeverity::Warning,
                });
            }
        }

        // Check risk score
        let risk_score = position.risk_assessment.risk_score;
        if risk_score >= thresholds.risk_score_critical_threshold {
            violations.push(ThresholdViolation {
                ticker: position.ticker.clone(),
                holding_name: None,
                metric_name: "Risk Score".to_string(),
                metric_value: risk_score,
                threshold_value: thresholds.risk_score_critical_threshold,
                threshold_type: ViolationSeverity::Critical,
            });
        } else if risk_score >= thresholds.risk_score_warning_threshold {
            violations.push(ThresholdViolation {
                ticker: position.ticker.clone(),
                holding_name: None,
                metric_name: "Risk Score".to_string(),
                metric_value: risk_score,
                threshold_value: thresholds.risk_score_warning_threshold,
                threshold_type: ViolationSeverity::Warning,
            });
        }

        // Check VaR (more negative is worse)
        if let Some(var) = metrics.value_at_risk {
            if var <= thresholds.var_critical_threshold {
                violations.push(ThresholdViolation {
                    ticker: position.ticker.clone(),
                    holding_name: None,
                    metric_name: "Value at Risk".to_string(),
                    metric_value: var,
                    threshold_value: thresholds.var_critical_threshold,
                    threshold_type: ViolationSeverity::Critical,
                });
            } else if var <= thresholds.var_warning_threshold {
                violations.push(ThresholdViolation {
                    ticker: position.ticker.clone(),
                    holding_name: None,
                    metric_name: "Value at Risk".to_string(),
                    metric_value: var,
                    threshold_value: thresholds.var_warning_threshold,
                    threshold_type: ViolationSeverity::Warning,
                });
            }
        }
    }

    violations
}

/// GET /api/risk/portfolios/:portfolio_id/thresholds
///
/// Retrieve risk warning thresholds for a portfolio.
///
/// Returns default thresholds if none are configured yet.
pub async fn get_thresholds(
    Path(portfolio_id): Path<Uuid>,
    State(state): State<AppState>,
) -> Result<Json<RiskThresholdSettings>, AppError> {
    info!("GET /api/risk/portfolios/{}/thresholds - Retrieving risk thresholds", portfolio_id);

    let settings = crate::db::risk_threshold_queries::get_thresholds(&state.pool, portfolio_id)
        .await
        .map_err(|e| {
            error!("Failed to fetch risk thresholds: {}", e);
            AppError::Db(e)
        })?;

    Ok(Json(settings))
}

/// POST /api/risk/portfolios/:portfolio_id/thresholds
///
/// Update risk warning thresholds for a portfolio.
///
/// Request body: UpdateRiskThresholds with optional fields
pub async fn set_thresholds(
    Path(portfolio_id): Path<Uuid>,
    State(state): State<AppState>,
    Json(request): Json<UpdateRiskThresholds>,
) -> Result<Json<RiskThresholdSettings>, AppError> {
    info!("POST /api/risk/portfolios/{}/thresholds - Updating risk thresholds", portfolio_id);

    let settings = crate::db::risk_threshold_queries::upsert_thresholds(&state.pool, portfolio_id, &request)
        .await
        .map_err(|e| {
            error!("Failed to update risk thresholds: {}", e);
            AppError::Db(e)
        })?;

    Ok(Json(settings))
}

/// GET /api/risk/portfolios/:portfolio_id/correlations
///
/// Calculate correlation matrix for all positions in a portfolio.
///
/// Query parameters:
/// - `days`: Rolling window in days (default: 90)
///
/// Example: GET /api/risk/portfolios/{uuid}/correlations?days=90
pub async fn get_portfolio_correlations(
    Path(portfolio_id): Path<Uuid>,
    Query(params): Query<RiskQueryParams>,
    State(state): State<AppState>,
) -> Result<Json<CorrelationMatrix>, AppError> {
    use crate::db::{holding_snapshot_queries, price_queries};
    use std::collections::HashMap;
    use std::time::Instant;

    let start = Instant::now();
    info!(
        "GET /api/risk/portfolios/{}/correlations - Computing correlation matrix (days={})",
        portfolio_id, params.days
    );

    // 1. Fetch all latest holdings for the portfolio
    info!("Step 1: Fetching portfolio holdings...");
    let holdings = match holding_snapshot_queries::fetch_portfolio_latest_holdings(
        &state.pool,
        portfolio_id
    ).await {
        Ok(h) => {
            info!("Fetched {} holdings in {:?}", h.len(), start.elapsed());
            if h.is_empty() {
                error!("No holdings found for portfolio {}", portfolio_id);
                return Err(AppError::External(
                    "No holdings data found for this portfolio. Please import holdings data first or check that accounts are properly linked to this portfolio.".to_string()
                ));
            }
            h
        }
        Err(e) => {
            error!("Failed to fetch portfolio holdings: {}", e);
            return Err(AppError::Db(e));
        }
    };

    // 2. Aggregate holdings by ticker and filter out mutual funds and negligible positions
    info!("Step 2: Aggregating holdings by ticker...");
    let mut ticker_aggregates: HashMap<String, f64> = HashMap::new(); // ticker -> market_value
    let mut total_value = 0.0;
    let mut filtered_mutual_funds = Vec::new();

    for holding in &holdings {
        let market_value = holding.market_value.to_string().parse::<f64>().unwrap_or(0.0);
        total_value += market_value;

        // Skip mutual funds and other securities that won't have price data
        let is_mutual_fund = holding.industry.as_ref()
            .map(|i| i.to_lowercase().contains("mutual fund"))
            .unwrap_or(false);

        let is_proprietary_ticker = holding.ticker.starts_with("FID")
            || holding.ticker.starts_with("RBF")
            || holding.ticker.starts_with("LYZ")
            || holding.ticker.starts_with("BIP")
            || holding.ticker.starts_with("DYN")
            || holding.ticker.starts_with("EDG")
            || holding.ticker.len() > 5; // Most proprietary tickers are longer

        if is_mutual_fund || is_proprietary_ticker {
            filtered_mutual_funds.push(holding.ticker.clone());
            continue;
        }

        ticker_aggregates
            .entry(holding.ticker.clone())
            .and_modify(|mv| *mv += market_value)
            .or_insert(market_value);
    }

    if !filtered_mutual_funds.is_empty() {
        info!("Filtered out {} mutual funds/proprietary tickers: {:?}",
              filtered_mutual_funds.len(),
              &filtered_mutual_funds[..filtered_mutual_funds.len().min(5)]);
    }

    if total_value == 0.0 {
        error!("Portfolio has no holdings with market value");
        return Err(AppError::External(
            "Portfolio has no holdings with market value".to_string()
        ));
    }

    info!("Total portfolio value: ${:.2}, {} unique tickers", total_value, ticker_aggregates.len());

    // Filter to only include positions >= 1% of portfolio value (increased from 0.5% for performance)
    let min_value = total_value * 0.01;
    let mut tickers: Vec<String> = ticker_aggregates
        .iter()
        .filter(|(_, &market_value)| market_value >= min_value)
        .map(|(ticker, _)| ticker.clone())
        .collect();

    tickers.sort();
    info!("Tickers after 1% filter: {} (min value: ${:.2})", tickers.len(), min_value);

    // Limit to top 10 positions by value to prevent timeout
    // (10 tickers = 45 correlation pairs, vs 20 tickers = 190 pairs)
    if tickers.len() > 10 {
        let mut ticker_values: Vec<(String, f64)> = ticker_aggregates
            .iter()
            .map(|(t, &v)| (t.clone(), v))
            .collect();
        ticker_values.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        tickers = ticker_values.iter().take(10).map(|(t, _)| t.clone()).collect();
        tickers.sort();
        warn!("Limited correlation analysis to top 10 positions (out of {} total)", ticker_values.len());
    }

    if tickers.len() < 2 {
        error!("Need at least 2 positions but only found {}. {} mutual funds were filtered out.",
               tickers.len(), filtered_mutual_funds.len());
        let msg = if filtered_mutual_funds.is_empty() {
            "Need at least 2 equity/ETF positions with price data for correlation analysis.".to_string()
        } else {
            format!(
                "Insufficient data for correlation analysis. Portfolio contains mostly mutual funds ({} filtered out). \
                 Correlation analysis requires at least 2 publicly traded stocks or ETFs with price history. \
                 Consider adding more equity or ETF positions to your portfolio.",
                filtered_mutual_funds.len()
            )
        };
        return Err(AppError::External(msg));
    }

    info!("Computing correlations for {} tickers: {:?}", tickers.len(), tickers);

    // 3. Fetch price data for all tickers in one batch query (much faster!)
    info!("Step 3: Fetching price data for {} tickers (last {} days)...", tickers.len(), params.days);
    let fetch_start = Instant::now();
    let price_data = match price_queries::fetch_window_batch(&state.pool, &tickers, params.days).await {
        Ok(data) => {
            info!("Fetched price data for {} tickers in {:?}, got {} tickers with data",
                  tickers.len(), fetch_start.elapsed(), data.len());
            data
        }
        Err(e) => {
            error!("Failed to batch fetch price data: {}", e);
            return Err(AppError::Db(e));
        }
    };

    // Filter tickers to only those with sufficient price data (at least 2 points)
    info!("Step 4: Filtering tickers with sufficient price data...");
    tickers.retain(|t| {
        if let Some(prices) = price_data.get(t) {
            if prices.len() < 2 {
                warn!("Insufficient price data for ticker {} (only {} points)", t, prices.len());
                false
            } else {
                true
            }
        } else {
            warn!("No price data found for ticker {}", t);
            false
        }
    });

    if tickers.len() < 2 {
        error!("Not enough tickers with price data: {} (need at least 2)", tickers.len());
        return Err(AppError::External(
            format!(
                "Insufficient price data for correlation analysis. Only {} position(s) have price history. \
                 Correlation requires at least 2 stocks/ETFs with historical price data. \
                 Please ensure you have imported price history for your equity positions.",
                tickers.len()
            )
        ));
    }

    info!("Tickers with valid data: {} - {:?}", tickers.len(), tickers);

    // 5. Calculate correlation for each pair (upper triangle only)
    info!("Step 5: Calculating correlations for {} pairs...", (tickers.len() * (tickers.len() - 1)) / 2);
    let mut correlations = Vec::new();
    let corr_start = Instant::now();

    for i in 0..tickers.len() {
        for j in (i + 1)..tickers.len() {
            let ticker1 = &tickers[i];
            let ticker2 = &tickers[j];

            // Get price data - these should exist since we filtered above
            let series1 = match price_data.get(ticker1) {
                Some(s) => s,
                None => {
                    warn!("Missing price data for {} during correlation", ticker1);
                    continue;
                }
            };
            let series2 = match price_data.get(ticker2) {
                Some(s) => s,
                None => {
                    warn!("Missing price data for {} during correlation", ticker2);
                    continue;
                }
            };

            if let Some(corr) = risk_service::compute_correlation(series1, series2) {
                correlations.push(CorrelationPair {
                    ticker1: ticker1.clone(),
                    ticker2: ticker2.clone(),
                    correlation: corr,
                });
            } else {
                warn!("Could not compute correlation between {} and {}", ticker1, ticker2);
            }
        }
    }

    info!("Computed {} correlations in {:?}", correlations.len(), corr_start.elapsed());

    if correlations.is_empty() {
        return Err(AppError::External(
            "Failed to compute any correlations".to_string()
        ));
    }

    info!("Successfully computed correlation matrix in {:?}", start.elapsed());

    // Build 2D matrix for heatmap visualization
    let n = tickers.len();
    let mut matrix_2d = vec![vec![0.0; n]; n];

    // Set diagonal to 1.0 (perfect self-correlation)
    for i in 0..n {
        matrix_2d[i][i] = 1.0;
    }

    // Fill in correlations from pairs
    for pair in &correlations {
        if let (Some(i), Some(j)) = (
            tickers.iter().position(|t| t == &pair.ticker1),
            tickers.iter().position(|t| t == &pair.ticker2),
        ) {
            matrix_2d[i][j] = pair.correlation;
            matrix_2d[j][i] = pair.correlation; // Symmetric
        }
    }

    let matrix = CorrelationMatrix {
        portfolio_id: portfolio_id.to_string(),
        tickers,
        correlations,
        matrix_2d,
    };

    Ok(Json(matrix))
}

/// POST /api/risk/portfolios/:portfolio_id/snapshot
///
/// Manually trigger snapshot creation for a portfolio
///
/// Example: POST /api/risk/portfolios/{uuid}/snapshot
pub async fn create_portfolio_snapshot(
    Path(portfolio_id): Path<Uuid>,
    State(state): State<AppState>,
) -> Result<Json<Vec<RiskSnapshot>>, AppError> {
    info!(
        "POST /api/risk/portfolios/{}/snapshot - Creating risk snapshots",
        portfolio_id
    );

    let today = chrono::Utc::now().date_naive();

    let snapshots = risk_snapshot_service::create_daily_snapshots(
        &state.pool,
        portfolio_id,
        today,
        state.price_provider.as_ref(),
        &state.failure_cache,
        state.risk_free_rate,
    )
    .await?;

    info!(
        "Successfully created {} snapshots for portfolio {}",
        snapshots.len(),
        portfolio_id
    );

    Ok(Json(snapshots))
}

/// GET /api/risk/portfolios/:portfolio_id/history
///
/// Retrieve historical risk data for a portfolio or specific position
///
/// Query parameters:
/// - `days`: Number of days of history to retrieve (default: 90)
/// - `ticker`: Optional ticker symbol for position-specific history
///
/// Example: GET /api/risk/portfolios/{uuid}/history?days=180&ticker=AAPL
pub async fn get_risk_history(
    Path(portfolio_id): Path<Uuid>,
    Query(params): Query<RiskHistoryParams>,
    State(state): State<AppState>,
) -> Result<Json<Vec<RiskSnapshot>>, AppError> {
    info!(
        "GET /api/risk/portfolios/{}/history - Fetching risk history (days={}, ticker={:?})",
        portfolio_id, params.days, params.ticker
    );

    let history = risk_snapshot_service::get_risk_trend(
        &state.pool,
        portfolio_id,
        params.ticker.as_deref(),
        params.days,
        crate::models::risk_snapshot::Aggregation::Daily,
    )
    .await?;

    info!(
        "Successfully fetched {} historical snapshots for portfolio {}",
        history.len(),
        portfolio_id
    );

    Ok(Json(history))
}

/// GET /api/risk/portfolios/:portfolio_id/alerts
///
/// Get risk increase alerts for a portfolio
///
/// Query parameters:
/// - `days`: Lookback period in days (default: 30)
/// - `threshold`: Percentage change threshold for alerts (default: 20.0)
///
/// Example: GET /api/risk/portfolios/{uuid}/alerts?days=30&threshold=15.0
pub async fn get_risk_alerts(
    Path(portfolio_id): Path<Uuid>,
    Query(params): Query<AlertQueryParams>,
    State(state): State<AppState>,
) -> Result<Json<Vec<RiskAlert>>, AppError> {
    info!(
        "GET /api/risk/portfolios/{}/alerts - Detecting risk increases (days={}, threshold={}%)",
        portfolio_id, params.days, params.threshold
    );

    let alerts = risk_snapshot_service::detect_risk_increases(
        &state.pool,
        portfolio_id,
        params.days,
        params.threshold,
    )
    .await?;

    info!(
        "Found {} risk alerts for portfolio {}",
        alerts.len(),
        portfolio_id
    );

    Ok(Json(alerts))
}

/// GET /api/risk/portfolios/:portfolio_id/export/csv
///
/// Export portfolio risk analysis to CSV format
///
/// Query parameters:
/// - `days`: Rolling window in days (default: 90)
/// - `benchmark`: Benchmark ticker for beta (default: "SPY")
///
/// Returns CSV file with portfolio summary and position-level risk metrics
pub async fn export_portfolio_risk_csv(
    Path(portfolio_id): Path<Uuid>,
    Query(params): Query<RiskQueryParams>,
    State(state): State<AppState>,
) -> Result<Response, AppError> {
    info!(
        "GET /api/risk/portfolios/{}/export/csv - Exporting risk data to CSV",
        portfolio_id
    );

    // Get portfolio risk data (same as get_portfolio_risk)
    use crate::db::{holding_snapshot_queries, portfolio_queries};
    use std::collections::HashMap;

    // Fetch portfolio name
    let portfolio = portfolio_queries::fetch_one(&state.pool, portfolio_id)
        .await
        .map_err(|e| {
            error!("Failed to fetch portfolio: {}", e);
            AppError::Db(e)
        })?
        .ok_or_else(|| AppError::External("Portfolio not found".to_string()))?;

    // Fetch holdings
    let holdings = holding_snapshot_queries::fetch_portfolio_latest_holdings(
        &state.pool,
        portfolio_id
    ).await.map_err(|e| {
        error!("Failed to fetch portfolio holdings: {}", e);
        AppError::Db(e)
    })?;

    if holdings.is_empty() {
        return Err(AppError::External(
            "Portfolio has no holdings to export".to_string()
        ));
    }

    // Aggregate holdings by ticker
    let mut ticker_aggregates: HashMap<String, (f64, Option<String>)> = HashMap::new();
    let mut total_value = 0.0;

    for holding in &holdings {
        let market_value = holding.market_value.to_string().parse::<f64>().unwrap_or(0.0);
        total_value += market_value;

        ticker_aggregates
            .entry(holding.ticker.clone())
            .and_modify(|(mv, _)| *mv += market_value)
            .or_insert((market_value, holding.holding_name.clone()));
    }

    // Build CSV
    let mut csv_writer = csv::Writer::from_writer(vec![]);

    // Write header
    csv_writer.write_record(&[
        "Ticker",
        "Holding Name",
        "Market Value",
        "Portfolio Weight %",
        "Volatility %",
        "Max Drawdown %",
        "Beta",
        "Sharpe Ratio",
        "Value at Risk %",
        "Risk Score",
        "Risk Level",
    ]).map_err(|e| {
        error!("Failed to write CSV header: {}", e);
        AppError::External(format!("CSV generation error: {}", e))
    })?;

    // Process each ticker
    let mut rows_written = 0;
    for (ticker, (market_value, holding_name)) in ticker_aggregates {
        let weight = (market_value / total_value) * 100.0;

        // Compute risk metrics
        match risk_service::compute_risk_metrics(
            &state.pool,
            &ticker,
            params.days,
            &params.benchmark,
            state.price_provider.as_ref(),
            &state.failure_cache,
            state.risk_free_rate,
        ).await {
            Ok(assessment) => {
                csv_writer.write_record(&[
                    ticker,
                    holding_name.unwrap_or_else(|| "—".to_string()),
                    format!("{:.2}", market_value),
                    format!("{:.2}", weight),
                    format!("{:.2}", assessment.metrics.volatility),
                    format!("{:.2}", assessment.metrics.max_drawdown),
                    assessment.metrics.beta.map(|b| format!("{:.2}", b)).unwrap_or_else(|| "—".to_string()),
                    assessment.metrics.sharpe.map(|s| format!("{:.2}", s)).unwrap_or_else(|| "—".to_string()),
                    assessment.metrics.value_at_risk.map(|v| format!("{:.2}", v)).unwrap_or_else(|| "—".to_string()),
                    format!("{:.2}", assessment.risk_score),
                    assessment.risk_level.to_string().to_uppercase(),
                ]).map_err(|e| {
                    error!("Failed to write CSV row: {}", e);
                    AppError::External(format!("CSV generation error: {}", e))
                })?;
                rows_written += 1;
            },
            Err(e) => {
                warn!("Skipping {} due to error: {}", ticker, e);
                // Write row with error indication
                csv_writer.write_record(&[
                    ticker,
                    holding_name.unwrap_or_else(|| "—".to_string()),
                    format!("{:.2}", market_value),
                    format!("{:.2}", weight),
                    "N/A".to_string(),
                    "N/A".to_string(),
                    "N/A".to_string(),
                    "N/A".to_string(),
                    "N/A".to_string(),
                    "N/A".to_string(),
                    "ERROR".to_string(),
                ]).map_err(|e| {
                    error!("Failed to write CSV row: {}", e);
                    AppError::External(format!("CSV generation error: {}", e))
                })?;
            }
        }
    }

    let csv_data = csv_writer.into_inner().map_err(|e| {
        error!("Failed to finalize CSV: {}", e);
        AppError::External(format!("CSV generation error: {}", e))
    })?;

    info!("Successfully exported {} positions to CSV", rows_written);

    // Generate filename with date
    let filename = format!(
        "portfolio_risk_{}_{}_{}.csv",
        portfolio.name.replace(' ', "_"),
        portfolio_id,
        chrono::Utc::now().format("%Y%m%d")
    );

    // Build response with proper headers
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/csv; charset=utf-8")
        .header(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{}\"", filename)
        )
        .body(csv_data.into())
        .unwrap())
}
