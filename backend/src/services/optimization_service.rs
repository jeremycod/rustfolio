use sqlx::PgPool;
use std::collections::HashMap;
use tracing::{info, warn};
use uuid::Uuid;

use crate::db::holding_snapshot_queries;
use crate::errors::AppError;
use crate::external::price_provider::PriceProvider;
use crate::models::*;
use crate::services::{failure_cache::FailureCache, risk_service};

/// Analyze portfolio and generate optimization recommendations
pub async fn analyze_portfolio(
    pool: &PgPool,
    portfolio_id: Uuid,
    price_provider: &dyn PriceProvider,
    failure_cache: &FailureCache,
    risk_free_rate: f64,
) -> Result<OptimizationAnalysis, AppError> {
    info!("Analyzing portfolio {} for optimization", portfolio_id);

    // 1. Fetch portfolio holdings
    let holdings = holding_snapshot_queries::fetch_portfolio_latest_holdings(pool, portfolio_id)
        .await
        .map_err(AppError::Db)?;

    if holdings.is_empty() {
        return Err(AppError::External(
            "Portfolio has no holdings to analyze".to_string()
        ));
    }

    // 2. Aggregate holdings by ticker
    let mut ticker_aggregates: HashMap<String, (f64, f64, Option<String>)> = HashMap::new();
    let mut total_value = 0.0;

    for holding in &holdings {
        let market_value = holding.market_value.to_string().parse::<f64>().unwrap_or(0.0);
        let quantity = holding.quantity.to_string().parse::<f64>().unwrap_or(0.0);
        total_value += market_value;

        ticker_aggregates
            .entry(holding.ticker.clone())
            .and_modify(|(q, mv, _)| {
                *q += quantity;
                *mv += market_value;
            })
            .or_insert((quantity, market_value, holding.holding_name.clone()));
    }

    // 3. Calculate current metrics
    let current_metrics = calculate_current_metrics(
        pool,
        &ticker_aggregates,
        total_value,
        price_provider,
        failure_cache,
        risk_free_rate,
    )
    .await?;

    // 4. Generate recommendations
    let mut recommendations = Vec::new();

    // Check concentration risk
    if let Some(rec) = detect_concentration_risk(&ticker_aggregates, total_value, &current_metrics) {
        recommendations.push(rec);
    }

    // Check risk contributors
    let risk_contributions = calculate_risk_contributions(
        pool,
        &ticker_aggregates,
        total_value,
        price_provider,
        failure_cache,
        risk_free_rate,
    )
    .await?;

    if let Some(rec) = detect_excessive_risk_contributors(&risk_contributions, total_value) {
        recommendations.push(rec);
    }

    // Check diversification
    if let Some(rec) = assess_diversification(&ticker_aggregates, &current_metrics) {
        recommendations.push(rec);
    }

    // 5. Calculate summary
    let summary = calculate_summary(&recommendations, &current_metrics);

    // 6. Get portfolio name (for now, use ID; can be fetched from DB if needed)
    let portfolio_name = format!("Portfolio {}", portfolio_id);

    Ok(OptimizationAnalysis {
        portfolio_id: portfolio_id.to_string(),
        portfolio_name,
        total_value,
        analysis_date: chrono::Utc::now().format("%Y-%m-%d").to_string(),
        current_metrics,
        recommendations,
        summary,
    })
}

/// Calculate current portfolio metrics
async fn calculate_current_metrics(
    pool: &PgPool,
    ticker_aggregates: &HashMap<String, (f64, f64, Option<String>)>,
    total_value: f64,
    price_provider: &dyn PriceProvider,
    failure_cache: &FailureCache,
    risk_free_rate: f64,
) -> Result<CurrentMetrics, AppError> {
    let mut weighted_volatility = 0.0;
    let mut weighted_max_drawdown = 0.0;
    let mut weighted_sharpe = 0.0;
    let mut sharpe_count = 0;
    let mut risk_score_sum = 0.0;
    let mut risk_count = 0;

    // Calculate weighted metrics
    for (ticker, (_quantity, market_value, _name)) in ticker_aggregates {
        let weight = market_value / total_value;

        // Compute risk metrics for this ticker
        match risk_service::compute_risk_metrics(
            pool,
            ticker,
            90, // 90-day window
            "SPY",
            price_provider,
            failure_cache,
            risk_free_rate,
        )
        .await
        {
            Ok(assessment) => {
                weighted_volatility += assessment.metrics.volatility * weight;
                weighted_max_drawdown += assessment.metrics.max_drawdown.abs() * weight;

                if let Some(sharpe) = assessment.metrics.sharpe {
                    weighted_sharpe += sharpe * weight;
                    sharpe_count += 1;
                }

                risk_score_sum += assessment.risk_score * weight;
                risk_count += 1;
            }
            Err(e) => {
                warn!("Could not compute risk for {} in optimization: {}", ticker, e);
            }
        }
    }

    // Calculate diversification score
    let diversification_score = calculate_diversification_score(ticker_aggregates, total_value);

    // Calculate correlation-adjusted diversification score (if we have enough positions)
    let (correlation_adjusted_score, average_correlation) = if ticker_aggregates.len() >= 2 {
        match calculate_correlation_adjusted_diversification(
            pool,
            ticker_aggregates,
            total_value,
            price_provider,
            failure_cache,
        )
        .await
        {
            Ok((adj_score, avg_corr)) => (Some(adj_score), Some(avg_corr)),
            Err(e) => {
                warn!("Could not calculate correlation-adjusted diversification: {}", e);
                (None, None)
            }
        }
    } else {
        (None, None)
    };

    // Find largest position
    let mut positions: Vec<(String, f64)> = ticker_aggregates
        .iter()
        .map(|(ticker, (_, value, _))| (ticker.clone(), *value / total_value * 100.0))
        .collect();
    positions.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    let largest_position_weight = positions.first().map(|(_, w)| *w).unwrap_or(0.0);
    let top_3_concentration: f64 = positions.iter().take(3).map(|(_, w)| w).sum();

    Ok(CurrentMetrics {
        risk_score: if risk_count > 0 { risk_score_sum } else { 0.0 },
        volatility: weighted_volatility,
        max_drawdown: weighted_max_drawdown,
        sharpe_ratio: if sharpe_count > 0 {
            Some(weighted_sharpe)
        } else {
            None
        },
        diversification_score,
        correlation_adjusted_diversification_score: correlation_adjusted_score,
        average_correlation,
        position_count: ticker_aggregates.len(),
        largest_position_weight,
        top_3_concentration,
    })
}

/// Detect concentration risk in portfolio
fn detect_concentration_risk(
    ticker_aggregates: &HashMap<String, (f64, f64, Option<String>)>,
    total_value: f64,
    current_metrics: &CurrentMetrics,
) -> Option<OptimizationRecommendation> {
    // Calculate position weights
    let mut positions: Vec<(String, f64, f64, Option<String>)> = ticker_aggregates
        .iter()
        .map(|(ticker, (_, value, name))| {
            (ticker.clone(), *value, *value / total_value * 100.0, name.clone())
        })
        .collect();
    positions.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap());

    let largest = &positions[0];
    let (ticker, value, weight, name) = largest;

    // Define thresholds
    let severity = if *weight > 30.0 {
        Severity::Critical
    } else if *weight > 20.0 {
        Severity::High
    } else if *weight > 15.0 {
        Severity::Warning
    } else {
        return None; // No concentration risk
    };

    // Calculate recommended adjustment
    let target_weight = 15.0; // Target max weight
    let target_value = total_value * (target_weight / 100.0);
    let amount_to_sell = value - target_value;

    let affected_positions = vec![PositionAdjustment {
        ticker: ticker.clone(),
        holding_name: name.clone(),
        current_value: *value,
        current_weight: *weight,
        recommended_value: target_value,
        recommended_weight: target_weight,
        action: AdjustmentAction::Sell,
        amount_change: -amount_to_sell,
        shares_change: None,
    }];

    // Estimate impact (simplified - assumes proportional risk reduction)
    let risk_reduction = (weight - target_weight) / weight * 0.3; // Rough estimate
    let expected_impact = ExpectedImpact {
        risk_score_before: current_metrics.risk_score,
        risk_score_after: current_metrics.risk_score * (1.0 - risk_reduction),
        risk_score_change: -current_metrics.risk_score * risk_reduction,
        volatility_before: current_metrics.volatility,
        volatility_after: current_metrics.volatility * (1.0 - risk_reduction * 0.8),
        volatility_change: -current_metrics.volatility * risk_reduction * 0.8,
        sharpe_before: current_metrics.sharpe_ratio,
        sharpe_after: current_metrics.sharpe_ratio.map(|s| s * 1.1),
        sharpe_change: current_metrics.sharpe_ratio.map(|s| s * 0.1),
        diversification_before: current_metrics.diversification_score,
        diversification_after: current_metrics.diversification_score + 1.5,
        diversification_change: 1.5,
        max_drawdown_before: current_metrics.max_drawdown,
        max_drawdown_after: current_metrics.max_drawdown * (1.0 - risk_reduction * 0.5),
    };

    let display_name = name.as_ref().unwrap_or(ticker);

    Some(OptimizationRecommendation {
        id: "concentration-1".to_string(),
        recommendation_type: RecommendationType::ReduceConcentration,
        severity,
        title: format!("High Concentration Risk in {}", ticker),
        rationale: format!(
            "{} represents {:.1}% of your portfolio, which is above the recommended maximum of 15%. \
             High concentration in a single position significantly increases portfolio risk. \
             If {} experiences a decline, it will have an outsized impact on your total portfolio value.",
            display_name, weight, display_name
        ),
        affected_positions,
        expected_impact,
        suggested_actions: vec![
            format!("Sell ${:.0} ({:.1}% of position) in {}", amount_to_sell, (amount_to_sell / value) * 100.0, ticker),
            "Reinvest proceeds into diversified assets (index funds or other low-correlation positions)".to_string(),
            format!("This will reduce {} position to 15% of portfolio", ticker),
        ],
    })
}

/// Calculate risk contributions for each position
async fn calculate_risk_contributions(
    pool: &PgPool,
    ticker_aggregates: &HashMap<String, (f64, f64, Option<String>)>,
    total_value: f64,
    price_provider: &dyn PriceProvider,
    failure_cache: &FailureCache,
    risk_free_rate: f64,
) -> Result<Vec<RiskContribution>, AppError> {
    let mut contributions = Vec::new();
    let mut total_risk = 0.0;

    // First pass: calculate individual volatilities
    let mut ticker_volatilities = HashMap::new();
    for (ticker, (_quantity, market_value, _name)) in ticker_aggregates {
        let weight = market_value / total_value;

        if let Ok(assessment) = risk_service::compute_risk_metrics(
            pool,
            ticker,
            90,
            "SPY",
            price_provider,
            failure_cache,
            risk_free_rate,
        )
        .await
        {
            let vol = assessment.metrics.volatility;
            ticker_volatilities.insert(ticker.clone(), vol);
            total_risk += weight * vol;
        }
    }

    // Second pass: calculate contributions
    for (ticker, (_quantity, market_value, _name)) in ticker_aggregates {
        let weight = market_value / total_value;
        if let Some(&volatility) = ticker_volatilities.get(ticker) {
            let contribution = if total_risk > 0.0 {
                (weight * volatility / total_risk) * 100.0
            } else {
                0.0
            };

            contributions.push(RiskContribution {
                ticker: ticker.clone(),
                weight,
                volatility,
                risk_contribution: contribution,
                is_excessive: contribution > 20.0,
            });
        }
    }

    contributions.sort_by(|a, b| b.risk_contribution.partial_cmp(&a.risk_contribution).unwrap());
    Ok(contributions)
}

/// Detect positions with excessive risk contribution
fn detect_excessive_risk_contributors(
    risk_contributions: &[RiskContribution],
    total_value: f64,
) -> Option<OptimizationRecommendation> {
    let excessive: Vec<&RiskContribution> = risk_contributions
        .iter()
        .filter(|rc| rc.is_excessive)
        .collect();

    if excessive.is_empty() {
        return None;
    }

    let top_contributor = excessive[0];
    let current_value = total_value * top_contributor.weight;

    // Recommend reducing to contribute max 15% of portfolio risk
    let target_contribution = 15.0;
    let reduction_factor = target_contribution / top_contributor.risk_contribution;
    let target_weight = top_contributor.weight * reduction_factor;
    let target_value = total_value * target_weight;

    let affected_positions = vec![PositionAdjustment {
        ticker: top_contributor.ticker.clone(),
        holding_name: None,
        current_value,
        current_weight: top_contributor.weight * 100.0,
        recommended_value: target_value,
        recommended_weight: target_weight * 100.0,
        action: AdjustmentAction::Sell,
        amount_change: target_value - current_value,
        shares_change: None,
    }];

    let expected_impact = ExpectedImpact {
        risk_score_before: 0.0, // Would need full calculation
        risk_score_after: 0.0,
        risk_score_change: -10.0, // Rough estimate
        volatility_before: 0.0,
        volatility_after: 0.0,
        volatility_change: -5.0,
        sharpe_before: None,
        sharpe_after: None,
        sharpe_change: None,
        diversification_before: 0.0,
        diversification_after: 0.0,
        diversification_change: 1.0,
        max_drawdown_before: 0.0,
        max_drawdown_after: 0.0,
    };

    Some(OptimizationRecommendation {
        id: "risk-contributor-1".to_string(),
        recommendation_type: RecommendationType::ReduceRisk,
        severity: if top_contributor.risk_contribution > 30.0 {
            Severity::High
        } else {
            Severity::Warning
        },
        title: format!("{} Contributing Excessive Risk", top_contributor.ticker),
        rationale: format!(
            "{} is contributing {:.1}% of your total portfolio risk, despite being only {:.1}% of your holdings. \
             This disproportionate risk contribution (volatility: {:.1}%) makes your portfolio more volatile than necessary.",
            top_contributor.ticker,
            top_contributor.risk_contribution,
            top_contributor.weight * 100.0,
            top_contributor.volatility
        ),
        affected_positions,
        expected_impact,
        suggested_actions: vec![
            format!(
                "Reduce {} position by {:.0}% to balance risk contribution",
                top_contributor.ticker,
                (1.0 - reduction_factor) * 100.0
            ),
            "Consider replacing with lower-volatility alternatives in the same sector".to_string(),
        ],
    })
}

/// Calculate diversification score (0-10)
fn calculate_diversification_score(
    ticker_aggregates: &HashMap<String, (f64, f64, Option<String>)>,
    total_value: f64,
) -> f64 {
    let position_count = ticker_aggregates.len() as f64;

    // Calculate Herfindahl index (concentration measure)
    let herfindahl: f64 = ticker_aggregates
        .values()
        .map(|(_, value, _)| {
            let weight = value / total_value;
            weight * weight
        })
        .sum();

    // Diversification score components:
    // 1. Number of positions (0-4 points): more positions = better (up to 20)
    let position_score = (position_count / 5.0).min(4.0);

    // 2. Herfindahl index (0-6 points): lower concentration = better
    //    Perfect diversification (20 equal positions) = 0.05
    //    Single position = 1.0
    let concentration_score = ((1.0 - herfindahl) / (1.0 - 0.05) * 6.0).max(0.0);

    let total_score = position_score + concentration_score;
    total_score.min(10.0)
}

/// Calculate correlation-adjusted diversification score (0-10)
/// Returns (adjusted_score, average_correlation)
async fn calculate_correlation_adjusted_diversification(
    pool: &PgPool,
    ticker_aggregates: &HashMap<String, (f64, f64, Option<String>)>,
    total_value: f64,
    _price_provider: &dyn PriceProvider,
    _failure_cache: &FailureCache,
) -> Result<(f64, f64), AppError> {
    // Calculate Herfindahl index (concentration measure) for base score
    let herfindahl: f64 = ticker_aggregates
        .values()
        .map(|(_, value, _)| {
            let weight = value / total_value;
            weight * weight
        })
        .sum();

    // Base score from concentration (0-6 points)
    let concentration_score = ((1.0 - herfindahl) / (1.0 - 0.05) * 6.0).max(0.0);

    // Limit to top 10 positions to avoid excessive computation
    let mut ticker_values: Vec<(String, f64)> = ticker_aggregates
        .iter()
        .map(|(ticker, (_, value, _))| (ticker.clone(), *value))
        .collect();
    ticker_values.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    let limited_tickers: Vec<String> = ticker_values.iter().take(10).map(|(t, _)| t.clone()).collect();

    // Fetch price data for all tickers
    let mut ticker_prices: HashMap<String, Vec<crate::models::PricePoint>> = HashMap::new();
    for ticker in &limited_tickers {
        match crate::services::price_service::get_history(pool, ticker).await {
            Ok(prices) if !prices.is_empty() => {
                ticker_prices.insert(ticker.clone(), prices);
            }
            _ => {
                warn!("Could not fetch price data for ticker {} in correlation calculation", ticker);
            }
        }
    }

    // Compute correlations between all pairs
    let mut correlations = Vec::new();
    for i in 0..limited_tickers.len() {
        for j in (i + 1)..limited_tickers.len() {
            let ticker1 = &limited_tickers[i];
            let ticker2 = &limited_tickers[j];

            if let (Some(prices1), Some(prices2)) = (
                ticker_prices.get(ticker1),
                ticker_prices.get(ticker2),
            ) {
                if let Some(corr) = risk_service::compute_correlation(prices1, prices2) {
                    correlations.push(corr.abs()); // Use absolute correlation
                }
            }
        }
    }

    let average_correlation = if !correlations.is_empty() {
        correlations.iter().sum::<f64>() / correlations.len() as f64
    } else {
        0.5 // Default to moderate correlation if we can't compute
    };

    // Correlation bonus (0-4 points): lower correlation = better
    // avg_corr = 0 (uncorrelated) → 4 points
    // avg_corr = 1 (perfectly correlated) → 0 points
    let correlation_bonus = (1.0 - average_correlation) * 4.0;

    let adjusted_score = (concentration_score + correlation_bonus).min(10.0);

    Ok((adjusted_score, average_correlation))
}

/// Assess diversification and generate recommendation if needed
fn assess_diversification(
    ticker_aggregates: &HashMap<String, (f64, f64, Option<String>)>,
    current_metrics: &CurrentMetrics,
) -> Option<OptimizationRecommendation> {
    let div_score = current_metrics.diversification_score;

    if div_score >= 7.0 {
        return None; // Good diversification
    }

    let position_count = ticker_aggregates.len();
    let severity = if div_score < 4.0 {
        Severity::High
    } else if div_score < 6.0 {
        Severity::Warning
    } else {
        Severity::Info
    };

    let rationale = if position_count < 5 {
        format!(
            "Your portfolio has only {} positions, which limits diversification (score: {:.1}/10). \
             A well-diversified portfolio typically holds 10-20 positions across different sectors and asset classes.",
            position_count, div_score
        )
    } else {
        format!(
            "Your portfolio's diversification score is {:.1}/10. While you have {} positions, \
             they may be concentrated in similar sectors or highly correlated assets.",
            div_score, position_count
        )
    };

    let suggested_actions = if position_count < 10 {
        vec![
            format!("Add {} more positions to reach 10-15 total positions", 10 - position_count),
            "Focus on uncorrelated assets (e.g., bonds, international stocks, commodities)".to_string(),
            "Consider broad-market ETFs for instant diversification".to_string(),
        ]
    } else {
        vec![
            "Review sector allocation - ensure no single sector exceeds 30%".to_string(),
            "Add asset classes with low correlation (bonds, REITs, commodities)".to_string(),
            "Consider rebalancing concentrated positions".to_string(),
        ]
    };

    Some(OptimizationRecommendation {
        id: "diversification-1".to_string(),
        recommendation_type: RecommendationType::IncreaseDiversification,
        severity,
        title: "Improve Portfolio Diversification".to_string(),
        rationale,
        affected_positions: vec![],
        expected_impact: ExpectedImpact {
            risk_score_before: current_metrics.risk_score,
            risk_score_after: current_metrics.risk_score * 0.85,
            risk_score_change: -current_metrics.risk_score * 0.15,
            volatility_before: current_metrics.volatility,
            volatility_after: current_metrics.volatility * 0.90,
            volatility_change: -current_metrics.volatility * 0.10,
            sharpe_before: current_metrics.sharpe_ratio,
            sharpe_after: current_metrics.sharpe_ratio.map(|s| s * 1.15),
            sharpe_change: current_metrics.sharpe_ratio.map(|s| s * 0.15),
            diversification_before: div_score,
            diversification_after: (div_score + 2.0).min(10.0),
            diversification_change: 2.0,
            max_drawdown_before: current_metrics.max_drawdown,
            max_drawdown_after: current_metrics.max_drawdown * 0.85,
        },
        suggested_actions,
    })
}

/// Calculate analysis summary
fn calculate_summary(
    recommendations: &[OptimizationRecommendation],
    current_metrics: &CurrentMetrics,
) -> AnalysisSummary {
    let total_recommendations = recommendations.len();
    let critical_issues = recommendations.iter().filter(|r| r.severity == Severity::Critical).count();
    let high_priority = recommendations.iter().filter(|r| r.severity == Severity::High).count();
    let warnings = recommendations.iter().filter(|r| r.severity == Severity::Warning).count();

    let overall_health = if critical_issues > 0 {
        PortfolioHealth::Critical
    } else if high_priority > 0 {
        PortfolioHealth::Poor
    } else if warnings > 1 {
        PortfolioHealth::Fair
    } else if warnings > 0 || current_metrics.diversification_score < 7.0 {
        PortfolioHealth::Good
    } else {
        PortfolioHealth::Excellent
    };

    let mut key_findings = Vec::new();

    if current_metrics.largest_position_weight > 20.0 {
        key_findings.push(format!(
            "Largest position ({:.1}%) exceeds recommended maximum",
            current_metrics.largest_position_weight
        ));
    }

    if current_metrics.diversification_score < 6.0 {
        key_findings.push(format!(
            "Diversification score ({:.1}/10) could be improved",
            current_metrics.diversification_score
        ));
    }

    if current_metrics.risk_score > 70.0 {
        key_findings.push("Overall portfolio risk is high".to_string());
    }

    if key_findings.is_empty() {
        key_findings.push("Portfolio is well-balanced with no major concerns".to_string());
    }

    AnalysisSummary {
        total_recommendations,
        critical_issues,
        high_priority,
        warnings,
        overall_health,
        key_findings,
    }
}
