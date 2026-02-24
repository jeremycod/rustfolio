use std::collections::HashMap;

use bigdecimal::ToPrimitive;
use sqlx::PgPool;
use tracing::{info, warn};
use uuid::Uuid;

use crate::db::holding_snapshot_queries;
use crate::errors::AppError;
use crate::external::price_provider::PriceProvider;
use crate::models::factor::*;
use crate::services::failure_cache::FailureCache;
use crate::services::price_service;
use crate::services::rate_limiter::RateLimiter;

// ============================================================================
// Public entry point
// ============================================================================

/// Perform full factor analysis for a portfolio.
pub async fn analyze_portfolio_factors(
    pool: &PgPool,
    portfolio_id: Uuid,
    price_provider: &dyn PriceProvider,
    failure_cache: &FailureCache,
    rate_limiter: &RateLimiter,
    risk_free_rate: f64,
    days: i64,
    include_backtest: bool,
    include_etfs: bool,
) -> Result<FactorAnalysisResponse, AppError> {
    info!("Starting factor analysis for portfolio {}", portfolio_id);

    // 1. Fetch portfolio holdings
    let holdings =
        holding_snapshot_queries::fetch_portfolio_latest_holdings(pool, portfolio_id).await
            .map_err(AppError::Db)?;

    if holdings.is_empty() {
        return Err(AppError::Validation(
            "Portfolio has no holdings to analyze".to_string(),
        ));
    }

    // 2. Aggregate holdings by ticker
    let mut ticker_aggregates: HashMap<String, (f64, f64, Option<String>)> = HashMap::new();
    let mut total_value = 0.0;

    for h in &holdings {
        let mv = h.market_value.to_string().parse::<f64>().unwrap_or(0.0);
        total_value += mv;
        let qty = h.quantity.to_string().parse::<f64>().unwrap_or(0.0);
        ticker_aggregates
            .entry(h.ticker.clone())
            .and_modify(|(q, v, _)| {
                *q += qty;
                *v += mv;
            })
            .or_insert((qty, mv, h.holding_name.clone()));
    }

    if total_value <= 0.0 {
        return Err(AppError::Validation(
            "Portfolio total value is zero".to_string(),
        ));
    }

    // 3. Score each holding on every factor
    let mut holdings_scores = Vec::new();
    for (ticker, (_qty, mv, name)) in &ticker_aggregates {
        // Pre-check: Skip tickers without sufficient price data to avoid slow API calls
        let has_data = match price_service::get_history(pool, ticker).await {
            Ok(p) if p.len() >= 20 => true,
            _ => {
                info!("Skipping {} - insufficient price data for factor analysis", ticker);
                false
            }
        };

        if !has_data {
            continue;
        }

        let weight = *mv / total_value;
        let scores = score_ticker(
            pool,
            ticker,
            price_provider,
            failure_cache,
            rate_limiter,
            risk_free_rate,
            days,
        )
        .await;
        let composite = FactorWeights::default().composite(&TickerFactorScores {
            ticker: ticker.clone(),
            holding_name: name.clone(),
            weight,
            value_score: scores.0,
            growth_score: scores.1,
            momentum_score: scores.2,
            quality_score: scores.3,
            low_volatility_score: scores.4,
            composite_score: 0.0,
        });
        holdings_scores.push(TickerFactorScores {
            ticker: ticker.clone(),
            holding_name: name.clone(),
            weight,
            value_score: scores.0,
            growth_score: scores.1,
            momentum_score: scores.2,
            quality_score: scores.3,
            low_volatility_score: scores.4,
            composite_score: composite,
        });
    }
    holdings_scores.sort_by(|a, b| b.composite_score.partial_cmp(&a.composite_score).unwrap_or(std::cmp::Ordering::Equal));

    // 4. Aggregate portfolio-level factor exposures
    let factor_exposures = compute_portfolio_exposures(&holdings_scores);

    // 5. Optimize factor weights (mean-variance inspired)
    let factor_weights = optimize_factor_weights(&holdings_scores, &factor_exposures);

    // 6. ETF suggestions
    let etf_suggestions = if include_etfs {
        generate_etf_suggestions(&factor_exposures)
    } else {
        vec![]
    };

    // 7. Back-testing
    let backtest_results = if include_backtest {
        run_factor_backtests(pool, &ticker_aggregates, total_value, days).await
    } else {
        vec![]
    };

    // 8. Summary
    let summary = build_summary(&factor_exposures, &holdings_scores);

    // 9. Portfolio name
    let portfolio_name = sqlx::query!("SELECT name FROM portfolios WHERE id = $1", portfolio_id)
        .fetch_optional(pool)
        .await?
        .map(|r| r.name)
        .unwrap_or_else(|| format!("Portfolio {}", portfolio_id));

    Ok(FactorAnalysisResponse {
        portfolio_id: portfolio_id.to_string(),
        portfolio_name,
        analysis_date: chrono::Utc::now().format("%Y-%m-%d").to_string(),
        holdings_scores,
        factor_exposures,
        factor_weights,
        etf_suggestions,
        backtest_results,
        summary,
    })
}

// ============================================================================
// Factor scoring for individual tickers
// ============================================================================

/// Returns (value, growth, momentum, quality, low_vol) scores in 0-100.
async fn score_ticker(
    pool: &PgPool,
    ticker: &str,
    price_provider: &dyn PriceProvider,
    failure_cache: &FailureCache,
    rate_limiter: &RateLimiter,
    risk_free_rate: f64,
    days: i64,
) -> (f64, f64, f64, f64, f64) {
    // Fetch price history
    let prices = match price_service::get_history(pool, ticker).await {
        Ok(p) if p.len() >= 2 => p,
        _ => {
            warn!("Insufficient price data for factor scoring of {}", ticker);
            return (50.0, 50.0, 50.0, 50.0, 50.0);
        }
    };

    // Take only the last `days` trading days
    let prices: Vec<_> = if prices.len() > days as usize {
        prices[prices.len() - days as usize..].to_vec()
    } else {
        prices
    };

    let closes: Vec<f64> = prices
        .iter()
        .filter_map(|p| p.close_price.to_f64())
        .collect();

    if closes.len() < 20 {
        return (50.0, 50.0, 50.0, 50.0, 50.0);
    }

    let value_score = compute_value_score(&closes);
    let growth_score = compute_growth_score(&closes);
    let momentum_score = compute_momentum_score(&closes);
    let quality_score = compute_quality_score(&closes);
    let low_vol_score = compute_low_volatility_score(
        pool,
        ticker,
        price_provider,
        failure_cache,
        rate_limiter,
        risk_free_rate,
        days,
    )
    .await;

    (
        value_score,
        growth_score,
        momentum_score,
        quality_score,
        low_vol_score,
    )
}

/// Value factor: uses price-to-moving-average ratio as a proxy for valuation
/// (lower ratio = more "value"). Also considers mean reversion.
fn compute_value_score(closes: &[f64]) -> f64 {
    let n = closes.len();
    if n < 50 {
        return 50.0;
    }
    // Long-term average as a proxy for intrinsic value
    let long_avg: f64 = closes.iter().sum::<f64>() / n as f64;
    let current = closes[n - 1];
    // Price-to-average ratio: < 1.0 means trading below average (value)
    let ratio = current / long_avg;

    // Also compute dividend-yield proxy: recent drawdown depth (deeper = higher yield proxy)
    let peak = closes.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let drawdown = if peak > 0.0 {
        (current - peak) / peak
    } else {
        0.0
    };

    // Score: lower ratio and deeper drawdown = higher value score
    let ratio_score = ((1.0 - (ratio - 0.7)) / 0.6 * 100.0).clamp(0.0, 100.0);
    let drawdown_score = ((-drawdown) / 0.5 * 100.0).clamp(0.0, 100.0);

    (ratio_score * 0.6 + drawdown_score * 0.4).clamp(0.0, 100.0)
}

/// Growth factor: measures recent price acceleration and trend strength.
fn compute_growth_score(closes: &[f64]) -> f64 {
    let n = closes.len();
    if n < 60 {
        return 50.0;
    }
    // 3-month return (approx 63 trading days, or use what we have)
    let lookback = (n / 4).max(20);
    let start = closes[n - lookback];
    let end = closes[n - 1];
    let return_pct = if start > 0.0 {
        (end - start) / start * 100.0
    } else {
        0.0
    };

    // Growth acceleration: compare first-half return to second-half return
    let mid = n - lookback + lookback / 2;
    let mid_price = closes[mid];
    let first_half_ret = if start > 0.0 {
        (mid_price - start) / start
    } else {
        0.0
    };
    let second_half_ret = if mid_price > 0.0 {
        (end - mid_price) / mid_price
    } else {
        0.0
    };
    let acceleration = second_half_ret - first_half_ret;

    // Map returns to score: +30% in lookback = 100, -30% = 0
    let return_score = ((return_pct + 30.0) / 60.0 * 100.0).clamp(0.0, 100.0);
    let accel_score = ((acceleration + 0.15) / 0.30 * 100.0).clamp(0.0, 100.0);

    (return_score * 0.7 + accel_score * 0.3).clamp(0.0, 100.0)
}

/// Momentum factor: 12-month return minus 1-month return (classic 12-1 momentum).
fn compute_momentum_score(closes: &[f64]) -> f64 {
    let n = closes.len();
    if n < 22 {
        return 50.0;
    }
    // 12-month minus 1-month momentum
    let twelve_month_start = if n > 252 { closes[n - 252] } else { closes[0] };
    let one_month_start = closes[n - 22];
    let current = closes[n - 1];

    let twelve_month_ret = if twelve_month_start > 0.0 {
        (current - twelve_month_start) / twelve_month_start
    } else {
        0.0
    };
    let one_month_ret = if one_month_start > 0.0 {
        (current - one_month_start) / one_month_start
    } else {
        0.0
    };
    // 12-1 momentum
    let momentum = twelve_month_ret - one_month_ret;

    // Relative strength: compare current price to 50-day and 200-day moving averages
    let sma50 = if n >= 50 {
        closes[n - 50..].iter().sum::<f64>() / 50.0
    } else {
        closes.iter().sum::<f64>() / n as f64
    };
    let sma200 = if n >= 200 {
        closes[n - 200..].iter().sum::<f64>() / 200.0
    } else {
        closes.iter().sum::<f64>() / n as f64
    };

    let above_50: f64 = if current > sma50 { 1.0 } else { 0.0 };
    let above_200: f64 = if current > sma200 { 1.0 } else { 0.0 };
    let trend_score = (above_50 * 50.0 + above_200 * 50.0).clamp(0.0, 100.0);

    // Map 12-1 momentum: +0.50 = 100, -0.50 = 0
    let momentum_score = ((momentum + 0.50) / 1.0 * 100.0).clamp(0.0, 100.0);

    (momentum_score * 0.6 + trend_score * 0.4).clamp(0.0, 100.0)
}

/// Quality factor: uses return consistency (R-squared of trend) and
/// low drawdown as proxies for earnings stability and profitability.
fn compute_quality_score(closes: &[f64]) -> f64 {
    let n = closes.len();
    if n < 30 {
        return 50.0;
    }

    // R-squared of linear trend (earnings stability proxy)
    let r_squared = compute_r_squared(closes);

    // Max drawdown (low debt / quality proxy - quality stocks have smaller drawdowns)
    let mut peak = closes[0];
    let mut max_dd = 0.0_f64;
    for &c in closes {
        if c > peak {
            peak = c;
        }
        let dd = (c - peak) / peak;
        if dd < max_dd {
            max_dd = dd;
        }
    }

    // Positive return consistency: fraction of positive daily returns
    let daily_returns: Vec<f64> = closes.windows(2).map(|w| (w[1] - w[0]) / w[0]).collect();
    let positive_frac = if !daily_returns.is_empty() {
        daily_returns.iter().filter(|&&r| r > 0.0).count() as f64 / daily_returns.len() as f64
    } else {
        0.5
    };

    // Score components
    let r2_score = (r_squared * 100.0).clamp(0.0, 100.0);
    let dd_score = ((1.0 + max_dd) * 100.0).clamp(0.0, 100.0); // max_dd is negative
    let consistency_score = (positive_frac * 100.0).clamp(0.0, 100.0);

    (r2_score * 0.4 + dd_score * 0.3 + consistency_score * 0.3).clamp(0.0, 100.0)
}

/// R-squared of a linear regression on the price series (0-1).
fn compute_r_squared(values: &[f64]) -> f64 {
    let n = values.len() as f64;
    if n < 3.0 {
        return 0.0;
    }
    let x_mean = (n - 1.0) / 2.0;
    let y_mean = values.iter().sum::<f64>() / n;

    let mut ss_xy = 0.0;
    let mut ss_xx = 0.0;
    let mut ss_yy = 0.0;
    for (i, &v) in values.iter().enumerate() {
        let x = i as f64 - x_mean;
        let y = v - y_mean;
        ss_xy += x * y;
        ss_xx += x * x;
        ss_yy += y * y;
    }

    if ss_xx == 0.0 || ss_yy == 0.0 {
        return 0.0;
    }
    let r = ss_xy / (ss_xx.sqrt() * ss_yy.sqrt());
    (r * r).clamp(0.0, 1.0)
}

/// Low-volatility factor: uses annualized volatility computed from existing price data.
/// This version does NOT call ensure_fresh_price_data to avoid slow external API calls during factor analysis.
async fn compute_low_volatility_score(
    pool: &PgPool,
    ticker: &str,
    _price_provider: &dyn PriceProvider,
    _failure_cache: &FailureCache,
    _rate_limiter: &RateLimiter,
    _risk_free_rate: f64,
    days: i64,
) -> f64 {
    // Use existing price data from database without fetching fresh data
    let prices = match price_service::get_history(pool, ticker).await {
        Ok(p) if p.len() >= 20 => p,
        _ => return 50.0,
    };

    let prices: Vec<_> = if prices.len() > days as usize {
        prices[prices.len() - days as usize..].to_vec()
    } else {
        prices
    };

    let closes: Vec<f64> = prices
        .iter()
        .filter_map(|p| p.close_price.to_f64())
        .collect();

    if closes.len() < 20 {
        return 50.0;
    }

    // Compute daily returns
    let daily_returns: Vec<f64> = closes
        .windows(2)
        .map(|w| (w[1] - w[0]) / w[0])
        .collect();

    if daily_returns.is_empty() {
        return 50.0;
    }

    // Calculate volatility (standard deviation of returns)
    let mean_return = daily_returns.iter().sum::<f64>() / daily_returns.len() as f64;
    let variance = daily_returns
        .iter()
        .map(|r| (r - mean_return).powi(2))
        .sum::<f64>()
        / daily_returns.len() as f64;
    let daily_vol = variance.sqrt();

    // Annualize volatility (assuming 252 trading days)
    let annualized_vol = daily_vol * (252.0_f64).sqrt() * 100.0; // Convert to percentage

    // Lower volatility = higher score
    // Vol of 10% => 90, vol of 50% => 10
    let vol_score = ((50.0 - annualized_vol) / 40.0 * 100.0).clamp(0.0, 100.0);

    // For factor analysis, we only use volatility score (no beta calculation to avoid external calls)
    vol_score
}

// ============================================================================
// Portfolio-level aggregation
// ============================================================================

fn compute_portfolio_exposures(scores: &[TickerFactorScores]) -> Vec<PortfolioFactorExposure> {
    let total_weight: f64 = scores.iter().map(|s| s.weight).sum();
    if total_weight <= 0.0 {
        return vec![];
    }

    let factors: Vec<(FactorType, fn(&TickerFactorScores) -> f64)> = vec![
        (FactorType::Value, |s: &TickerFactorScores| s.value_score),
        (FactorType::Growth, |s: &TickerFactorScores| s.growth_score),
        (FactorType::Momentum, |s: &TickerFactorScores| s.momentum_score),
        (FactorType::Quality, |s: &TickerFactorScores| s.quality_score),
        (FactorType::LowVolatility, |s: &TickerFactorScores| s.low_volatility_score),
    ];

    factors
        .iter()
        .map(|(ft, extractor)| {
            let weighted_score: f64 = scores
                .iter()
                .map(|s| s.weight * extractor(s))
                .sum::<f64>()
                / total_weight;

            let exposure = ExposureLevel::from_score(weighted_score);
            let premium = expected_risk_premium(ft);
            let recommendation = factor_recommendation(ft, &exposure, weighted_score);

            PortfolioFactorExposure {
                factor: ft.clone(),
                label: ft.label().to_string(),
                description: ft.description().to_string(),
                score: (weighted_score * 10.0).round() / 10.0,
                exposure_level: exposure,
                expected_risk_premium: premium,
                recommendation,
            }
        })
        .collect()
}

/// Historical expected risk premiums (annualised, in percentage points).
/// Based on long-run factor return research (Fama-French, AQR, etc.)
fn expected_risk_premium(factor: &FactorType) -> f64 {
    match factor {
        FactorType::Value => 4.5,        // HML historically ~4-5% per year
        FactorType::Growth => 2.0,       // Growth premium is debated; moderate estimate
        FactorType::Momentum => 6.0,     // UMD historically ~6-8%
        FactorType::Quality => 3.5,      // QMJ ~3-4%
        FactorType::LowVolatility => 2.5, // BAB ~2-3%
    }
}

fn factor_recommendation(factor: &FactorType, exposure: &ExposureLevel, score: f64) -> String {
    match exposure {
        ExposureLevel::Underweight => format!(
            "Your portfolio has low {} exposure ({:.0}/100). Consider adding {} stocks or ETFs to capture the estimated {:.1}% annual risk premium.",
            factor.label().to_lowercase(),
            score,
            factor.label().to_lowercase(),
            expected_risk_premium(factor),
        ),
        ExposureLevel::Neutral => format!(
            "Your {} exposure is balanced ({:.0}/100). This factor has a historical risk premium of ~{:.1}% per year.",
            factor.label().to_lowercase(),
            score,
            expected_risk_premium(factor),
        ),
        ExposureLevel::Overweight => format!(
            "Your portfolio is heavily tilted toward {} ({:.0}/100). This may concentrate risk, though {} stocks have historically earned a {:.1}% annual premium.",
            factor.label().to_lowercase(),
            score,
            factor.label().to_lowercase(),
            expected_risk_premium(factor),
        ),
    }
}

// ============================================================================
// Multi-factor weight optimizer
// ============================================================================

/// A simplified mean-variance optimizer for factor weights.
/// Tilts toward factors with higher Sharpe-like ratios observed in the portfolio.
fn optimize_factor_weights(
    scores: &[TickerFactorScores],
    exposures: &[PortfolioFactorExposure],
) -> FactorWeights {
    if exposures.is_empty() {
        return FactorWeights::default();
    }

    // Compute a "return per unit of dispersion" for each factor across holdings
    let factor_attractiveness: Vec<f64> = exposures
        .iter()
        .map(|exp| {
            let premium = exp.expected_risk_premium;
            let dispersion = compute_factor_dispersion(scores, &exp.factor);
            // Sharpe-like ratio: premium / dispersion
            if dispersion > 0.01 {
                premium / dispersion
            } else {
                premium
            }
        })
        .collect();

    let total: f64 = factor_attractiveness.iter().sum();
    if total <= 0.0 {
        return FactorWeights::default();
    }

    // Normalise to weights (floor at 5% each)
    let raw: Vec<f64> = factor_attractiveness
        .iter()
        .map(|&a| (a / total).max(0.05))
        .collect();
    let raw_total: f64 = raw.iter().sum();

    FactorWeights {
        value: raw[0] / raw_total,
        growth: raw[1] / raw_total,
        momentum: raw[2] / raw_total,
        quality: raw[3] / raw_total,
        low_volatility: raw[4] / raw_total,
    }
}

/// Standard deviation of a factor's scores across all holdings (weighted).
fn compute_factor_dispersion(scores: &[TickerFactorScores], factor: &FactorType) -> f64 {
    let extractor: fn(&TickerFactorScores) -> f64 = match factor {
        FactorType::Value => |s| s.value_score,
        FactorType::Growth => |s| s.growth_score,
        FactorType::Momentum => |s| s.momentum_score,
        FactorType::Quality => |s| s.quality_score,
        FactorType::LowVolatility => |s| s.low_volatility_score,
    };

    let total_weight: f64 = scores.iter().map(|s| s.weight).sum();
    if total_weight <= 0.0 || scores.is_empty() {
        return 0.0;
    }

    let mean: f64 = scores.iter().map(|s| s.weight * extractor(s)).sum::<f64>() / total_weight;
    let variance: f64 = scores
        .iter()
        .map(|s| {
            let diff = extractor(s) - mean;
            s.weight * diff * diff
        })
        .sum::<f64>()
        / total_weight;

    variance.sqrt()
}

// ============================================================================
// ETF suggestion system
// ============================================================================

/// Static ETF database mapping factors to well-known ETFs.
fn generate_etf_suggestions(exposures: &[PortfolioFactorExposure]) -> Vec<FactorEtfSuggestion> {
    let etf_db: Vec<FactorEtfSuggestion> = vec![
        // Value ETFs
        FactorEtfSuggestion {
            ticker: "VTV".to_string(),
            name: "Vanguard Value ETF".to_string(),
            factor: FactorType::Value,
            expense_ratio: 0.04,
            aum_billions: 115.0,
            description: "Large-cap US value stocks selected by fundamental ratios".to_string(),
        },
        FactorEtfSuggestion {
            ticker: "VLUE".to_string(),
            name: "iShares MSCI USA Value Factor ETF".to_string(),
            factor: FactorType::Value,
            expense_ratio: 0.15,
            aum_billions: 8.0,
            description: "Targets undervalued large/mid-cap US equities using P/E, P/B, and EV/CF".to_string(),
        },
        FactorEtfSuggestion {
            ticker: "IUSV".to_string(),
            name: "iShares Core S&P US Value ETF".to_string(),
            factor: FactorType::Value,
            expense_ratio: 0.04,
            aum_billions: 14.0,
            description: "Broad US value exposure tracking the S&P 900 Value Index".to_string(),
        },
        // Growth ETFs
        FactorEtfSuggestion {
            ticker: "VUG".to_string(),
            name: "Vanguard Growth ETF".to_string(),
            factor: FactorType::Growth,
            expense_ratio: 0.04,
            aum_billions: 120.0,
            description: "Large-cap US growth stocks with strong earnings and revenue growth".to_string(),
        },
        FactorEtfSuggestion {
            ticker: "IWF".to_string(),
            name: "iShares Russell 1000 Growth ETF".to_string(),
            factor: FactorType::Growth,
            expense_ratio: 0.19,
            aum_billions: 90.0,
            description: "Russell 1000 companies with above-average growth characteristics".to_string(),
        },
        // Momentum ETFs
        FactorEtfSuggestion {
            ticker: "MTUM".to_string(),
            name: "iShares MSCI USA Momentum Factor ETF".to_string(),
            factor: FactorType::Momentum,
            expense_ratio: 0.15,
            aum_billions: 12.0,
            description: "US large/mid-cap stocks exhibiting strong recent price momentum".to_string(),
        },
        FactorEtfSuggestion {
            ticker: "PDP".to_string(),
            name: "Invesco DWA Momentum ETF".to_string(),
            factor: FactorType::Momentum,
            expense_ratio: 0.62,
            aum_billions: 1.8,
            description: "Momentum strategy based on relative strength from Dorsey Wright".to_string(),
        },
        // Quality ETFs
        FactorEtfSuggestion {
            ticker: "QUAL".to_string(),
            name: "iShares MSCI USA Quality Factor ETF".to_string(),
            factor: FactorType::Quality,
            expense_ratio: 0.15,
            aum_billions: 40.0,
            description: "US large/mid-cap stocks with high ROE, stable earnings, and low leverage".to_string(),
        },
        FactorEtfSuggestion {
            ticker: "DGRW".to_string(),
            name: "WisdomTree US Quality Dividend Growth ETF".to_string(),
            factor: FactorType::Quality,
            expense_ratio: 0.28,
            aum_billions: 13.0,
            description: "Dividend-paying companies selected for quality growth characteristics".to_string(),
        },
        // Low-Volatility ETFs
        FactorEtfSuggestion {
            ticker: "USMV".to_string(),
            name: "iShares MSCI USA Min Vol Factor ETF".to_string(),
            factor: FactorType::LowVolatility,
            expense_ratio: 0.15,
            aum_billions: 28.0,
            description: "US equities selected and weighted to minimise portfolio volatility".to_string(),
        },
        FactorEtfSuggestion {
            ticker: "SPLV".to_string(),
            name: "Invesco S&P 500 Low Volatility ETF".to_string(),
            factor: FactorType::LowVolatility,
            expense_ratio: 0.25,
            aum_billions: 10.0,
            description: "100 least-volatile S&P 500 constituents, equal weighted".to_string(),
        },
    ];

    // Filter: only suggest ETFs for factors where the portfolio is underweight
    // Also always include 1-2 per factor for context
    let mut suggestions = Vec::new();
    for exp in exposures {
        let factor_etfs: Vec<&FactorEtfSuggestion> = etf_db
            .iter()
            .filter(|e| e.factor == exp.factor)
            .collect();

        if exp.exposure_level == ExposureLevel::Underweight {
            // Suggest all matching ETFs for underweight factors
            for etf in &factor_etfs {
                suggestions.push((*etf).clone());
            }
        } else {
            // Just suggest the lowest-cost option for neutral/overweight
            if let Some(cheapest) = factor_etfs.iter().min_by(|a, b| {
                a.expense_ratio
                    .partial_cmp(&b.expense_ratio)
                    .unwrap_or(std::cmp::Ordering::Equal)
            }) {
                suggestions.push((*cheapest).clone());
            }
        }
    }

    // Sort by expense ratio (cheapest first)
    suggestions.sort_by(|a, b| {
        a.expense_ratio
            .partial_cmp(&b.expense_ratio)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    suggestions
}

// ============================================================================
// Back-testing
// ============================================================================

/// Run simple factor back-tests using portfolio holdings' price data.
/// For each factor, constructs a hypothetical top-quintile portfolio
/// weighted by factor score and computes performance metrics.
async fn run_factor_backtests(
    pool: &PgPool,
    ticker_aggregates: &HashMap<String, (f64, f64, Option<String>)>,
    _total_value: f64,
    days: i64,
) -> Vec<FactorBacktestResult> {
    let mut results = Vec::new();

    // Fetch prices for all tickers
    let mut all_prices: HashMap<String, Vec<f64>> = HashMap::new();
    let mut min_len = usize::MAX;

    for ticker in ticker_aggregates.keys() {
        match price_service::get_history(pool, ticker).await {
            Ok(prices) if prices.len() >= 20 => {
                let closes: Vec<f64> = prices
                    .iter()
                    .filter_map(|p| p.close_price.to_f64())
                    .collect();
                let trimmed = if closes.len() > days as usize {
                    closes[closes.len() - days as usize..].to_vec()
                } else {
                    closes
                };
                if trimmed.len() < min_len {
                    min_len = trimmed.len();
                }
                all_prices.insert(ticker.clone(), trimmed);
            }
            _ => continue,
        }
    }

    if all_prices.len() < 2 || min_len < 20 {
        return results;
    }

    // Align all series to the same length
    for series in all_prices.values_mut() {
        let start = series.len().saturating_sub(min_len);
        *series = series[start..].to_vec();
    }

    for factor in FactorType::all() {
        if let Some(bt) = backtest_single_factor(&all_prices, &factor, min_len) {
            results.push(bt);
        }
    }

    results
}

fn backtest_single_factor(
    all_prices: &HashMap<String, Vec<f64>>,
    factor: &FactorType,
    series_len: usize,
) -> Option<FactorBacktestResult> {
    if series_len < 20 {
        return None;
    }

    // Score each ticker for this factor at the start of the period
    let mut scored: Vec<(String, f64)> = all_prices
        .iter()
        .filter_map(|(ticker, prices)| {
            if prices.len() < 20 {
                return None;
            }
            let score = match factor {
                FactorType::Value => compute_value_score(&prices[..prices.len() / 2]),
                FactorType::Growth => compute_growth_score(&prices[..prices.len() / 2]),
                FactorType::Momentum => compute_momentum_score(&prices[..prices.len() / 2]),
                FactorType::Quality => compute_quality_score(&prices[..prices.len() / 2]),
                FactorType::LowVolatility => {
                    // Use inverse volatility of first half
                    let returns: Vec<f64> = prices[..prices.len() / 2]
                        .windows(2)
                        .map(|w| (w[1] - w[0]) / w[0])
                        .collect();
                    let vol = if returns.len() > 1 {
                        let mean = returns.iter().sum::<f64>() / returns.len() as f64;
                        let var = returns.iter().map(|r| (r - mean).powi(2)).sum::<f64>()
                            / (returns.len() - 1) as f64;
                        var.sqrt() * (252.0_f64).sqrt() * 100.0
                    } else {
                        30.0
                    };
                    ((50.0 - vol) / 40.0 * 100.0).clamp(0.0, 100.0)
                }
            };
            Some((ticker.clone(), score))
        })
        .collect();

    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    // Take top quintile (at least 1)
    let top_n = (scored.len() / 5).max(1);
    let top_tickers: Vec<&str> = scored.iter().take(top_n).map(|(t, _)| t.as_str()).collect();

    if top_tickers.is_empty() {
        return None;
    }

    // Compute equal-weight portfolio returns from the second half of the period
    let half = series_len / 2;
    let mut portfolio_values = vec![1.0_f64];

    for day in half..series_len - 1 {
        let mut daily_return = 0.0;
        let mut count = 0;
        for &ticker in &top_tickers {
            if let Some(prices) = all_prices.get(ticker) {
                if day + 1 < prices.len() && prices[day] > 0.0 {
                    daily_return += (prices[day + 1] - prices[day]) / prices[day];
                    count += 1;
                }
            }
        }
        if count > 0 {
            daily_return /= count as f64;
        }
        let prev = portfolio_values.last().copied().unwrap_or(1.0);
        portfolio_values.push(prev * (1.0 + daily_return));
    }

    if portfolio_values.len() < 10 {
        return None;
    }

    // Compute metrics
    let daily_returns: Vec<f64> = portfolio_values
        .windows(2)
        .map(|w| (w[1] - w[0]) / w[0])
        .collect();

    let mean_ret = daily_returns.iter().sum::<f64>() / daily_returns.len() as f64;
    let var = daily_returns
        .iter()
        .map(|r| (r - mean_ret).powi(2))
        .sum::<f64>()
        / (daily_returns.len() - 1).max(1) as f64;
    let daily_vol = var.sqrt();
    let ann_return = mean_ret * 252.0 * 100.0;
    let ann_vol = daily_vol * (252.0_f64).sqrt() * 100.0;
    let sharpe = if ann_vol > 0.0 {
        ann_return / ann_vol
    } else {
        0.0
    };

    let mut peak = portfolio_values[0];
    let mut max_dd = 0.0_f64;
    for &v in &portfolio_values {
        if v > peak {
            peak = v;
        }
        let dd = (v - peak) / peak;
        if dd < max_dd {
            max_dd = dd;
        }
    }

    let cumulative = (portfolio_values.last().unwrap_or(&1.0) - 1.0) * 100.0;

    Some(FactorBacktestResult {
        factor: factor.clone(),
        annualized_return: (ann_return * 10.0).round() / 10.0,
        annualized_volatility: (ann_vol * 10.0).round() / 10.0,
        sharpe_ratio: (sharpe * 100.0).round() / 100.0,
        max_drawdown: (max_dd * 1000.0).round() / 10.0,
        observation_days: portfolio_values.len(),
        cumulative_return: (cumulative * 10.0).round() / 10.0,
    })
}

// ============================================================================
// Summary builder
// ============================================================================

fn build_summary(
    exposures: &[PortfolioFactorExposure],
    scores: &[TickerFactorScores],
) -> FactorAnalysisSummary {
    let dominant = exposures
        .iter()
        .max_by(|a, b| a.score.partial_cmp(&b.score).unwrap_or(std::cmp::Ordering::Equal))
        .map(|e| e.label.clone())
        .unwrap_or_else(|| "N/A".to_string());

    let weakest = exposures
        .iter()
        .min_by(|a, b| a.score.partial_cmp(&b.score).unwrap_or(std::cmp::Ordering::Equal))
        .map(|e| e.label.clone())
        .unwrap_or_else(|| "N/A".to_string());

    let total_weight: f64 = scores.iter().map(|s| s.weight).sum();
    let overall_composite = if total_weight > 0.0 {
        scores.iter().map(|s| s.weight * s.composite_score).sum::<f64>() / total_weight
    } else {
        50.0
    };

    let mut findings = Vec::new();

    // Identify underweight factors
    let underweight: Vec<_> = exposures
        .iter()
        .filter(|e| e.exposure_level == ExposureLevel::Underweight)
        .collect();
    if !underweight.is_empty() {
        findings.push(format!(
            "Portfolio is underweight in: {}",
            underweight
                .iter()
                .map(|e| e.label.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }

    // Identify overweight factors
    let overweight: Vec<_> = exposures
        .iter()
        .filter(|e| e.exposure_level == ExposureLevel::Overweight)
        .collect();
    if !overweight.is_empty() {
        findings.push(format!(
            "Portfolio is overweight in: {}",
            overweight
                .iter()
                .map(|e| e.label.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }

    // Composite assessment
    if overall_composite >= 65.0 {
        findings.push("Overall multi-factor composite is strong, indicating good factor diversification.".to_string());
    } else if overall_composite >= 40.0 {
        findings.push("Overall multi-factor composite is moderate. Consider rebalancing toward underweight factors.".to_string());
    } else {
        findings.push("Overall multi-factor composite is weak. Significant factor rebalancing is recommended.".to_string());
    }

    // Best individual stock
    if let Some(best) = scores.first() {
        findings.push(format!(
            "Highest composite score: {} ({:.0}/100)",
            best.ticker, best.composite_score
        ));
    }

    FactorAnalysisSummary {
        dominant_factor: dominant,
        weakest_factor: weakest,
        overall_composite_score: (overall_composite * 10.0).round() / 10.0,
        key_findings: findings,
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_score_basic() {
        // Flat prices => moderate value score
        let prices: Vec<f64> = (0..100).map(|_| 100.0).collect();
        let score = compute_value_score(&prices);
        assert!(score >= 0.0 && score <= 100.0);
    }

    #[test]
    fn test_value_score_declining() {
        // Declining prices => higher value (cheaper)
        let prices: Vec<f64> = (0..100).map(|i| 100.0 - i as f64 * 0.3).collect();
        let score = compute_value_score(&prices);
        assert!(score > 40.0, "Declining prices should have decent value score, got {}", score);
    }

    #[test]
    fn test_momentum_score_uptrend() {
        // Strong uptrend => high momentum
        let prices: Vec<f64> = (0..260).map(|i| 100.0 + i as f64 * 0.5).collect();
        let score = compute_momentum_score(&prices);
        assert!(score > 50.0, "Uptrend should have high momentum, got {}", score);
    }

    #[test]
    fn test_momentum_score_downtrend() {
        // Downtrend => low momentum
        let prices: Vec<f64> = (0..260).map(|i| 200.0 - i as f64 * 0.5).collect();
        let score = compute_momentum_score(&prices);
        assert!(score < 60.0, "Downtrend should have low momentum, got {}", score);
    }

    #[test]
    fn test_growth_score_accelerating() {
        // Accelerating growth
        let prices: Vec<f64> = (0..100).map(|i| 100.0 + (i as f64).powi(2) * 0.01).collect();
        let score = compute_growth_score(&prices);
        assert!(score > 40.0, "Accelerating growth should score above average, got {}", score);
    }

    #[test]
    fn test_quality_score_stable() {
        // Stable uptrend => high quality
        let prices: Vec<f64> = (0..200).map(|i| 100.0 + i as f64 * 0.1).collect();
        let score = compute_quality_score(&prices);
        assert!(score > 50.0, "Stable uptrend should score well on quality, got {}", score);
    }

    #[test]
    fn test_r_squared_perfect_line() {
        let values: Vec<f64> = (0..50).map(|i| 10.0 + i as f64 * 2.0).collect();
        let r2 = compute_r_squared(&values);
        assert!((r2 - 1.0).abs() < 0.01, "Perfect line should have R2 near 1.0, got {}", r2);
    }

    #[test]
    fn test_exposure_level_classification() {
        assert_eq!(ExposureLevel::from_score(20.0), ExposureLevel::Underweight);
        assert_eq!(ExposureLevel::from_score(50.0), ExposureLevel::Neutral);
        assert_eq!(ExposureLevel::from_score(80.0), ExposureLevel::Overweight);
    }

    #[test]
    fn test_factor_weights_default_sum() {
        let w = FactorWeights::default();
        let sum = w.value + w.growth + w.momentum + w.quality + w.low_volatility;
        assert!((sum - 1.0).abs() < 0.001, "Default weights should sum to 1.0, got {}", sum);
    }

    #[test]
    fn test_composite_score_calculation() {
        let weights = FactorWeights::default();
        let scores = TickerFactorScores {
            ticker: "TEST".to_string(),
            holding_name: None,
            weight: 1.0,
            value_score: 80.0,
            growth_score: 60.0,
            momentum_score: 70.0,
            quality_score: 90.0,
            low_volatility_score: 50.0,
            composite_score: 0.0,
        };
        let composite = weights.composite(&scores);
        let expected = 0.20 * 80.0 + 0.20 * 60.0 + 0.20 * 70.0 + 0.20 * 90.0 + 0.20 * 50.0;
        assert!((composite - expected).abs() < 0.001, "Composite should be {}, got {}", expected, composite);
    }

    #[test]
    fn test_backtest_single_factor_insufficient_data() {
        let prices: HashMap<String, Vec<f64>> = HashMap::new();
        let result = backtest_single_factor(&prices, &FactorType::Value, 5);
        assert!(result.is_none());
    }

    #[test]
    fn test_backtest_single_factor_with_data() {
        let mut prices = HashMap::new();
        // Two tickers with 100 data points each
        let up: Vec<f64> = (0..100).map(|i| 100.0 + i as f64 * 0.5).collect();
        let down: Vec<f64> = (0..100).map(|i| 100.0 - i as f64 * 0.2).collect();
        prices.insert("UP".to_string(), up);
        prices.insert("DOWN".to_string(), down);

        let result = backtest_single_factor(&prices, &FactorType::Momentum, 100);
        assert!(result.is_some());
        let bt = result.unwrap();
        assert!(bt.observation_days > 0);
    }

    #[test]
    fn test_portfolio_exposures() {
        let scores = vec![
            TickerFactorScores {
                ticker: "A".to_string(),
                holding_name: None,
                weight: 0.6,
                value_score: 80.0,
                growth_score: 30.0,
                momentum_score: 50.0,
                quality_score: 70.0,
                low_volatility_score: 60.0,
                composite_score: 58.0,
            },
            TickerFactorScores {
                ticker: "B".to_string(),
                holding_name: None,
                weight: 0.4,
                value_score: 40.0,
                growth_score: 70.0,
                momentum_score: 60.0,
                quality_score: 50.0,
                low_volatility_score: 80.0,
                composite_score: 60.0,
            },
        ];
        let exposures = compute_portfolio_exposures(&scores);
        assert_eq!(exposures.len(), 5);
        // Value exposure: 0.6*80 + 0.4*40 = 48+16 = 64
        assert!((exposures[0].score - 64.0).abs() < 1.0, "Value exposure should be ~64, got {}", exposures[0].score);
    }

    #[test]
    fn test_etf_suggestions_underweight() {
        let exposures = vec![PortfolioFactorExposure {
            factor: FactorType::Value,
            label: "Value".to_string(),
            description: "desc".to_string(),
            score: 20.0,
            exposure_level: ExposureLevel::Underweight,
            expected_risk_premium: 4.5,
            recommendation: "".to_string(),
        }];
        let suggestions = generate_etf_suggestions(&exposures);
        assert!(suggestions.iter().any(|e| e.factor == FactorType::Value));
        // Should suggest multiple value ETFs for underweight
        let value_count = suggestions.iter().filter(|e| e.factor == FactorType::Value).count();
        assert!(value_count >= 2, "Should suggest multiple ETFs for underweight factor, got {}", value_count);
    }
}
