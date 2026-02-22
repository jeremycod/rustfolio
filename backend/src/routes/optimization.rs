use axum::extract::{Path, State};
use axum::{Json, Router};
use axum::routing::get;
use tracing::{error, info};
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::{OptimizationAnalysis, OptimizationRecommendation, CurrentMetrics, AnalysisSummary, PortfolioHealth, Severity};
use crate::state::AppState;
use bigdecimal::ToPrimitive;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/portfolios/:portfolio_id", get(get_portfolio_optimization))
        .route("/portfolios/:portfolio_id/generate", axum::routing::post(generate_portfolio_optimization))
}

/// GET /api/optimization/portfolios/:portfolio_id
///
/// Get portfolio optimization recommendations from cache
///
/// Example: GET /api/optimization/portfolios/{uuid}
#[axum::debug_handler]
pub async fn get_portfolio_optimization(
    Path(portfolio_id): Path<Uuid>,
    State(state): State<AppState>,
) -> Result<Json<OptimizationAnalysis>, AppError> {
    info!(
        "GET /api/optimization/portfolios/{} - Fetching cached optimization",
        portfolio_id
    );

    // Try to read from cache first
    let cached = sqlx::query!(
        r#"
        SELECT
            recommendations,
            risk_free_rate,
            calculated_at
        FROM portfolio_optimization_cache
        WHERE portfolio_id = $1
          AND expires_at > NOW()
        "#,
        portfolio_id
    )
    .fetch_optional(&state.pool)
    .await?;

    if let Some(cache) = cached {
        // Parse cached recommendations
        let recommendations: Vec<OptimizationRecommendation> =
            serde_json::from_value(cache.recommendations)
                .map_err(|e| AppError::External(format!("Failed to deserialize recommendations: {}", e)))?;

        info!(
            "✅ Returning {} cached recommendations for portfolio {} (cached at {})",
            recommendations.len(),
            portfolio_id,
            cache.calculated_at
        );

        // Get portfolio info for response
        let portfolio = sqlx::query!(
            "SELECT name FROM portfolios WHERE id = $1",
            portfolio_id
        )
        .fetch_one(&state.pool)
        .await?;

        // Get total value from latest holdings
        let total_value_decimal = sqlx::query!(
            r#"
            SELECT COALESCE(SUM(hs.quantity * pp.close_price), 0) as "total_value!"
            FROM holdings_snapshots hs
            JOIN accounts a ON hs.account_id = a.id
            LEFT JOIN price_points pp ON pp.ticker = hs.ticker
                AND pp.date = (
                    SELECT MAX(date) FROM price_points WHERE ticker = hs.ticker
                )
            WHERE a.portfolio_id = $1
              AND hs.quantity > 0
            "#,
            portfolio_id
        )
        .fetch_one(&state.pool)
        .await?
        .total_value;

        let total_value = total_value_decimal.to_f64().unwrap_or(0.0);

        // Get current metrics (simplified for cached version)
        let current_metrics = CurrentMetrics {
            risk_score: 0.0,
            volatility: 0.0,
            max_drawdown: 0.0,
            sharpe_ratio: Some(0.0),
            diversification_score: 0.0,
            correlation_adjusted_diversification_score: Some(0.0),
            average_correlation: Some(0.0),
            position_count: 0,
            largest_position_weight: 0.0,
            top_3_concentration: 0.0,
        };

        let high_priority = recommendations.iter().filter(|r| r.severity == Severity::High).count();
        let critical = recommendations.iter().filter(|r| r.severity == Severity::Critical).count();

        let summary = AnalysisSummary {
            total_recommendations: recommendations.len(),
            critical_issues: critical,
            high_priority,
            warnings: 0,
            overall_health: PortfolioHealth::Good,
            key_findings: vec![],
        };

        let analysis = OptimizationAnalysis {
            portfolio_id: portfolio_id.to_string(),
            portfolio_name: portfolio.name,
            total_value,
            analysis_date: cache.calculated_at.to_string(),
            current_metrics,
            recommendations,
            summary,
        };

        return Ok(Json(analysis));
    }

    // Cache miss - check if there's expired cache
    let expired_cache = sqlx::query!(
        r#"
        SELECT
            recommendations,
            risk_free_rate,
            calculated_at,
            expires_at
        FROM portfolio_optimization_cache
        WHERE portfolio_id = $1
        ORDER BY calculated_at DESC
        LIMIT 1
        "#,
        portfolio_id
    )
    .fetch_optional(&state.pool)
    .await?;

    if let Some(cache) = expired_cache {
        // We have expired cache - return it with a stale warning
        info!(
            "⚠️  Returning STALE cache for portfolio {} (expired at {})",
            portfolio_id,
            cache.expires_at
        );

        let recommendations: Vec<OptimizationRecommendation> =
            serde_json::from_value(cache.recommendations)
                .map_err(|e| AppError::External(format!("Failed to deserialize recommendations: {}", e)))?;

        // Get portfolio info
        let portfolio = sqlx::query!(
            "SELECT name FROM portfolios WHERE id = $1",
            portfolio_id
        )
        .fetch_one(&state.pool)
        .await?;

        // Get total value
        let total_value_decimal = sqlx::query!(
            r#"
            SELECT COALESCE(SUM(hs.quantity * pp.close_price), 0) as "total_value!"
            FROM holdings_snapshots hs
            JOIN accounts a ON hs.account_id = a.id
            LEFT JOIN price_points pp ON pp.ticker = hs.ticker
                AND pp.date = (
                    SELECT MAX(date) FROM price_points WHERE ticker = hs.ticker
                )
            WHERE a.portfolio_id = $1
              AND hs.quantity > 0
            "#,
            portfolio_id
        )
        .fetch_one(&state.pool)
        .await?
        .total_value;

        let total_value = total_value_decimal.to_f64().unwrap_or(0.0);

        let current_metrics = CurrentMetrics {
            risk_score: 0.0,
            volatility: 0.0,
            max_drawdown: 0.0,
            sharpe_ratio: Some(0.0),
            diversification_score: 0.0,
            correlation_adjusted_diversification_score: Some(0.0),
            average_correlation: Some(0.0),
            position_count: 0,
            largest_position_weight: 0.0,
            top_3_concentration: 0.0,
        };

        let high_priority = recommendations.iter().filter(|r| r.severity == Severity::High).count();
        let critical = recommendations.iter().filter(|r| r.severity == Severity::Critical).count();

        let summary = AnalysisSummary {
            total_recommendations: recommendations.len(),
            critical_issues: critical,
            high_priority,
            warnings: 0,
            overall_health: PortfolioHealth::Good,
            key_findings: vec![format!(
                "⚠️ This analysis is stale. Last updated: {}. Click 'Refresh Analysis' to update.",
                cache.calculated_at.format("%Y-%m-%d %H:%M UTC")
            )],
        };

        let analysis = OptimizationAnalysis {
            portfolio_id: portfolio_id.to_string(),
            portfolio_name: portfolio.name,
            total_value,
            analysis_date: cache.calculated_at.to_string(),
            current_metrics,
            recommendations,
            summary,
        };

        return Ok(Json(analysis));
    }

    // No cache at all - return empty state
    error!(
        "❌ No optimization cache found (not even expired) for portfolio {}",
        portfolio_id
    );

    // Get portfolio info for empty state
    let portfolio = sqlx::query!(
        "SELECT name FROM portfolios WHERE id = $1",
        portfolio_id
    )
    .fetch_one(&state.pool)
    .await?;

    let total_value_decimal = sqlx::query!(
        r#"
        SELECT COALESCE(SUM(hs.quantity * pp.close_price), 0) as "total_value!"
        FROM holdings_snapshots hs
        JOIN accounts a ON hs.account_id = a.id
        LEFT JOIN price_points pp ON pp.ticker = hs.ticker
            AND pp.date = (
                SELECT MAX(date) FROM price_points WHERE ticker = hs.ticker
            )
        WHERE a.portfolio_id = $1
          AND hs.quantity > 0
        "#,
        portfolio_id
    )
    .fetch_one(&state.pool)
    .await?
    .total_value;

    let total_value = total_value_decimal.to_f64().unwrap_or(0.0);

    // Return empty analysis with helpful message
    let analysis = OptimizationAnalysis {
        portfolio_id: portfolio_id.to_string(),
        portfolio_name: portfolio.name,
        total_value,
        analysis_date: chrono::Utc::now().to_string(),
        current_metrics: CurrentMetrics {
            risk_score: 0.0,
            volatility: 0.0,
            max_drawdown: 0.0,
            sharpe_ratio: Some(0.0),
            diversification_score: 0.0,
            correlation_adjusted_diversification_score: Some(0.0),
            average_correlation: Some(0.0),
            position_count: 0,
            largest_position_weight: 0.0,
            top_3_concentration: 0.0,
        },
        recommendations: vec![],
        summary: AnalysisSummary {
            total_recommendations: 0,
            critical_issues: 0,
            high_priority: 0,
            warnings: 0,
            overall_health: PortfolioHealth::Good,
            key_findings: vec![
                "No optimization analysis available yet.".to_string(),
                "Click 'Generate Analysis' to create your first optimization report.".to_string(),
            ],
        },
    };

    Ok(Json(analysis))
}

/// POST /api/optimization/portfolios/:portfolio_id/generate
///
/// Manually trigger optimization calculation for a portfolio
#[axum::debug_handler]
pub async fn generate_portfolio_optimization(
    Path(portfolio_id): Path<Uuid>,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    info!(
        "POST /api/optimization/portfolios/{}/generate - Manual optimization trigger",
        portfolio_id
    );

    // Verify portfolio exists
    let _portfolio = sqlx::query!(
        "SELECT id FROM portfolios WHERE id = $1",
        portfolio_id
    )
    .fetch_one(&state.pool)
    .await
    .map_err(|_| AppError::Validation("Portfolio not found".to_string()))?;

    // Create JobContext from AppState
    use std::sync::Arc;
    use crate::services::job_scheduler_service::JobContext;

    let ctx = JobContext {
        pool: Arc::new(state.pool.clone()),
        price_provider: state.price_provider.clone(),
        failure_cache: Arc::new(state.failure_cache.clone()),
        rate_limiter: state.rate_limiter.clone(),
    };

    // Import the job function
    use crate::jobs::populate_optimization_cache_job::calculate_single_portfolio_optimization;

    // Run the optimization job for this portfolio
    match calculate_single_portfolio_optimization(&ctx, portfolio_id).await {
        Ok(_) => {
            info!("✅ Successfully generated optimization for portfolio {}", portfolio_id);
            Ok(Json(serde_json::json!({
                "message": "Optimization analysis generated successfully",
                "portfolio_id": portfolio_id.to_string()
            })))
        }
        Err(e) => {
            error!("❌ Failed to generate optimization for portfolio {}: {:?}", portfolio_id, e);
            Err(AppError::External(format!("Failed to generate optimization: {}", e)))
        }
    }
}
