use axum::extract::{Path, Query, State};
use axum::{Json, Router};
use axum::routing::{get, post};
use tracing::{error, info};
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::factor::{FactorAnalysisResponse, FactorQueryParams};
use crate::models::long_term_guidance::{
    LongTermGuidanceResponse, LongTermGuidanceQuery,
    InvestmentGoal, RiskTolerance,
};
use crate::models::{ExplanationQuery, NarrativeType, RecommendationExplanation};
use crate::models::screening::{ScreeningRequest, ScreeningResponse};
use crate::services::factor_service;
use crate::services::explanation_service;
use crate::services::long_term_guidance_service::LongTermGuidanceService;
use crate::services::screening_service::ScreeningService;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/screen", post(screen_stocks))
        .route("/factors/:portfolio_id", get(get_factor_recommendations))
        .route("/long-term/:portfolio_id", get(get_long_term_guidance))
        .route("/:symbol/explanation", get(get_recommendation_explanation))
}

/// POST /api/recommendations/screen
///
/// Multi-factor stock screening endpoint. Screens a universe of stocks using
/// weighted fundamental, technical, sentiment, and momentum factors. Supports
/// filtering by sector, market cap, price range, geography, and liquidity.
/// Results are ranked by composite score and paginated.
///
/// Screening results are cached for 15 minutes unless `refresh: true`.
///
/// # Request Body
/// ```json
/// {
///   "symbols": ["AAPL", "GOOG", "MSFT"],
///   "weights": { "fundamental": 0.3, "technical": 0.2, "sentiment": 0.2, "momentum": 0.3 },
///   "filters": { "min_price": 50.0, "sectors": ["Technology"] },
///   "limit": 10,
///   "risk_appetite": "moderate",
///   "horizon_months": 6
/// }
/// ```
#[axum::debug_handler]
pub async fn screen_stocks(
    State(state): State<AppState>,
    Json(req): Json<ScreeningRequest>,
) -> Result<Json<ScreeningResponse>, AppError> {
    info!(
        "POST /recommendations/screen - symbols={}, limit={}, offset={}, risk={:?}, horizon={:?}",
        req.symbols.len(),
        req.limit,
        req.offset,
        req.risk_appetite,
        req.horizon_months,
    );

    // Validate limit
    if req.limit > 100 {
        return Err(AppError::Validation(
            "limit must be at most 100".to_string(),
        ));
    }

    // Validate weights are non-negative if provided
    for (name, val) in [
        ("fundamental", req.weights.fundamental),
        ("technical", req.weights.technical),
        ("sentiment", req.weights.sentiment),
        ("momentum", req.weights.momentum),
    ] {
        if let Some(w) = val {
            if !(0.0..=1.0).contains(&w) {
                return Err(AppError::Validation(
                    format!("{} weight must be between 0.0 and 1.0", name),
                ));
            }
        }
    }

    // Validate horizon
    if let Some(h) = req.horizon_months {
        if !(1..=60).contains(&h) {
            return Err(AppError::Validation(
                "horizon_months must be between 1 and 60".to_string(),
            ));
        }
    }

    let service = ScreeningService::new(state.pool.clone());

    let response = service.screen(&req).await.map_err(|e| {
        error!("Screening failed: {}", e);
        AppError::External(format!("Screening failed: {}", e))
    })?;

    info!(
        "Screening complete: {} results from {} screened ({} passed filters)",
        response.results.len(),
        response.total_screened,
        response.total_passed_filters,
    );

    Ok(Json(response))
}

/// GET /api/recommendations/factors/:portfolio_id
///
/// Perform factor-based portfolio analysis including:
/// - Per-holding factor scores (value, growth, momentum, quality, low-volatility)
/// - Portfolio-level factor exposures and expected risk premiums
/// - Multi-factor combination optimization
/// - ETF suggestions for rebalancing
/// - Historical back-test results
///
/// # Query Parameters
/// - `days`: Price history window in trading days (default: 252)
/// - `include_backtest`: Include back-test results (default: true)
/// - `include_etfs`: Include ETF suggestions (default: true)
///
/// # Example
/// ```
/// GET /api/recommendations/factors/{portfolio_id}?days=252&include_backtest=true
/// ```
#[axum::debug_handler]
pub async fn get_factor_recommendations(
    Path(portfolio_id): Path<Uuid>,
    Query(params): Query<FactorQueryParams>,
    State(state): State<AppState>,
) -> Result<Json<FactorAnalysisResponse>, AppError> {
    let days = params.days.unwrap_or(252);
    let include_backtest = params.include_backtest.unwrap_or(true);
    let include_etfs = params.include_etfs.unwrap_or(true);

    info!(
        "GET /api/recommendations/factors/{} - days={}, backtest={}, etfs={}",
        portfolio_id, days, include_backtest, include_etfs
    );

    // Validate portfolio exists
    sqlx::query!("SELECT id FROM portfolios WHERE id = $1", portfolio_id)
        .fetch_one(&state.pool)
        .await
        .map_err(|_| AppError::NotFound(format!("Portfolio {} not found", portfolio_id)))?;

    let analysis = factor_service::analyze_portfolio_factors(
        &state.pool,
        portfolio_id,
        state.price_provider.as_ref(),
        &state.failure_cache,
        &state.rate_limiter,
        state.risk_free_rate,
        days,
        include_backtest,
        include_etfs,
    )
    .await
    .map_err(|e| {
        error!(
            "Factor analysis failed for portfolio {}: {:?}",
            portfolio_id, e
        );
        e
    })?;

    info!(
        "Factor analysis complete for portfolio {}: {} holdings scored, {} exposures, {} ETF suggestions, {} backtests",
        portfolio_id,
        analysis.holdings_scores.len(),
        analysis.factor_exposures.len(),
        analysis.etf_suggestions.len(),
        analysis.backtest_results.len(),
    );

    Ok(Json(analysis))
}

/// GET /api/recommendations/long-term/:portfolio_id
///
/// Generate long-term investment guidance for a portfolio, including:
/// - Quality scoring for each holding (growth, dividend, moat, management)
/// - Risk classification per holding (low/medium/high)
/// - Dividend aristocrat and blue-chip candidate identification
/// - Goal-based allocation recommendations (retirement, college, wealth)
/// - Portfolio-level summary with improvement suggestions
///
/// # Query Parameters
/// - `goal` - Investment goal: retirement, college, wealth (default: retirement)
/// - `horizon` - Investment horizon in years: 1-40 (default: 20)
/// - `risk_tolerance` - Risk tolerance: conservative, moderate, aggressive (default: moderate)
/// - `min_quality` - Minimum quality score filter: 0-100 (optional)
/// - `refresh` - Force refresh, ignore cache (default: false)
///
/// # Example
/// ```
/// GET /api/recommendations/long-term/{portfolio_id}?goal=retirement&horizon=20&risk_tolerance=conservative
/// ```
#[axum::debug_handler]
pub async fn get_long_term_guidance(
    Path(portfolio_id): Path<Uuid>,
    Query(query): Query<LongTermGuidanceQuery>,
    State(state): State<AppState>,
) -> Result<Json<LongTermGuidanceResponse>, AppError> {
    // Parse and validate parameters
    let goal = if let Some(ref g) = query.goal {
        InvestmentGoal::from_str_opt(g).ok_or_else(|| {
            AppError::Validation(
                "Invalid goal. Must be one of: retirement, college, wealth".to_string(),
            )
        })?
    } else {
        InvestmentGoal::Retirement
    };

    let risk_tolerance = if let Some(ref rt) = query.risk_tolerance {
        RiskTolerance::from_str_opt(rt).ok_or_else(|| {
            AppError::Validation(
                "Invalid risk_tolerance. Must be one of: conservative, moderate, aggressive"
                    .to_string(),
            )
        })?
    } else {
        RiskTolerance::Moderate
    };

    let horizon_years = query.horizon.unwrap_or(20);
    if !(1..=40).contains(&horizon_years) {
        return Err(AppError::Validation(
            "Invalid horizon. Must be between 1 and 40 years.".to_string(),
        ));
    }

    if let Some(min_q) = query.min_quality {
        if !(0.0..=100.0).contains(&min_q) {
            return Err(AppError::Validation(
                "Invalid min_quality. Must be between 0 and 100.".to_string(),
            ));
        }
    }

    info!(
        "GET /recommendations/long-term/{} - goal={}, horizon={}, risk_tolerance={:?}",
        portfolio_id, goal, horizon_years, risk_tolerance
    );

    // Check that the portfolio exists
    let portfolio = crate::db::portfolio_queries::fetch_one(&state.pool, portfolio_id)
        .await
        .map_err(|e| {
            error!("Failed to fetch portfolio {}: {}", portfolio_id, e);
            AppError::Db(e)
        })?
        .ok_or_else(|| {
            AppError::NotFound(format!("Portfolio {} not found", portfolio_id))
        })?;

    info!("Generating long-term guidance for portfolio: {}", portfolio.name);

    // Check cache first (unless refresh requested)
    if !query.refresh {
        match crate::db::long_term_guidance_queries::get_cached_guidance(
            &state.pool,
            portfolio_id,
            &goal.to_string(),
            horizon_years,
            &format!("{:?}", risk_tolerance).to_lowercase(),
        ).await {
            Ok(Some(cached)) => {
                info!("Returning cached long-term guidance for portfolio {}", portfolio_id);
                return Ok(Json(cached));
            }
            Ok(None) => {
                info!("No cached guidance found, generating fresh analysis");
            }
            Err(e) => {
                error!("Cache lookup error: {}", e);
            }
        }
    }

    // Generate fresh guidance
    let service = LongTermGuidanceService::new(state.pool.clone(), state.risk_free_rate);

    let response = service
        .generate_guidance(
            portfolio_id,
            &goal,
            &risk_tolerance,
            horizon_years,
            query.min_quality,
        )
        .await
        .map_err(|e| {
            error!("Failed to generate long-term guidance: {}", e);
            AppError::External(format!("Failed to generate long-term guidance: {}", e))
        })?;

    // Cache the result
    if let Err(e) = crate::db::long_term_guidance_queries::cache_guidance(
        &state.pool,
        portfolio_id,
        &goal.to_string(),
        horizon_years,
        &format!("{:?}", risk_tolerance).to_lowercase(),
        &response,
    ).await {
        error!("Failed to cache long-term guidance: {}", e);
    }

    info!(
        "Generated long-term guidance for portfolio {} with {} recommendations",
        portfolio_id,
        response.recommendations.len()
    );

    Ok(Json(response))
}

/// GET /api/recommendations/:symbol/explanation
///
/// Generates an AI-powered explanation for a stock recommendation.
///
/// Uses Claude (Anthropic) API to generate insightful, educational analysis
/// based on the stock's technical signals, price data, and risk metrics.
///
/// Explanations are cached for 1 hour to manage LLM API costs.
///
/// # Query Parameters
/// - `narrative_type`: valuation | growth | risk | contrarian | dividend | balanced (default: balanced)
/// - `refresh`: true/false (default: false) - force refresh, bypassing cache
///
/// # Example
/// ```
/// GET /api/recommendations/AAPL/explanation?narrative_type=growth
/// GET /api/recommendations/MSFT/explanation?refresh=true
/// ```
#[axum::debug_handler]
pub async fn get_recommendation_explanation(
    Path(symbol): Path<String>,
    Query(query): Query<ExplanationQuery>,
    State(state): State<AppState>,
) -> Result<Json<RecommendationExplanation>, AppError> {
    let symbol = symbol.to_uppercase();

    info!(
        "GET /recommendations/{}/explanation - narrative_type={:?}, refresh={}",
        symbol, query.narrative_type, query.refresh
    );

    // Validate symbol
    if symbol.is_empty() || symbol.len() > 10 {
        return Err(AppError::Validation(
            "Invalid symbol. Must be 1-10 characters.".to_string(),
        ));
    }

    // Reject if this looks like a UUID (prevent conflict with portfolio_id routes)
    if symbol.contains('-') && symbol.len() > 10 {
        return Err(AppError::Validation(
            "Invalid symbol format. Use a stock ticker like AAPL.".to_string(),
        ));
    }

    // Parse narrative type
    let narrative_type = match query.narrative_type.as_deref() {
        Some("valuation") => NarrativeType::Valuation,
        Some("growth") => NarrativeType::Growth,
        Some("risk") => NarrativeType::Risk,
        Some("contrarian") => NarrativeType::Contrarian,
        Some("dividend") => NarrativeType::Dividend,
        Some("balanced") | None => NarrativeType::Balanced,
        Some(other) => {
            return Err(AppError::Validation(format!(
                "Invalid narrative_type: '{}'. Must be one of: valuation, growth, risk, contrarian, dividend, balanced",
                other
            )));
        }
    };

    let explanation = explanation_service::get_or_generate_explanation(
        &state.pool,
        state.llm_service.clone(),
        &symbol,
        narrative_type,
        query.refresh,
    )
    .await
    .map_err(|e| {
        error!("Failed to generate explanation for {}: {}", symbol, e);
        e
    })?;

    info!(
        "Returning explanation for {} (headline length: {}, narrative_type: {})",
        symbol,
        explanation.headline.len(),
        explanation.narrative_type,
    );

    Ok(Json(explanation))
}
