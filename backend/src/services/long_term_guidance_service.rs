use sqlx::PgPool;
use tracing::warn;
use uuid::Uuid;

use crate::db;
use crate::models::long_term_guidance::*;
use crate::services::price_service;

/// Service for computing long-term investment quality scores and recommendations
pub struct LongTermGuidanceService {
    pool: PgPool,
    risk_free_rate: f64,
}

impl LongTermGuidanceService {
    pub fn new(pool: PgPool, risk_free_rate: f64) -> Self {
        Self { pool, risk_free_rate }
    }

    /// Generate long-term guidance for a portfolio
    pub async fn generate_guidance(
        &self,
        portfolio_id: Uuid,
        goal: &InvestmentGoal,
        risk_tolerance: &RiskTolerance,
        horizon_years: i32,
        min_quality: Option<f64>,
    ) -> Result<LongTermGuidanceResponse, String> {
        // 1. Fetch current holdings for the portfolio
        let allocations = db::analytics_queries::fetch_allocations_at_latest_date(&self.pool, portfolio_id)
            .await
            .map_err(|e| format!("Failed to fetch portfolio allocations: {}", e))?;

        if allocations.is_empty() {
            return Err("No holdings found in portfolio".to_string());
        }

        // Fetch holding details (names, industries) from latest snapshot
        let holding_details = self.fetch_holding_details(portfolio_id).await?;

        let total_value: f64 = allocations.iter().map(|a| a.value).sum();

        // 2. Compute quality scores for each holding
        let mut recommendations = Vec::new();
        for alloc in &allocations {
            if alloc.ticker.is_empty() || alloc.value <= 0.0 {
                continue;
            }

            let details = holding_details.iter().find(|d| d.ticker == alloc.ticker);
            let holding_name = details.and_then(|d| d.holding_name.clone());
            let industry = details.and_then(|d| d.industry.clone());

            match self.compute_quality_score(
                &alloc.ticker,
                holding_name.as_deref(),
                industry.as_deref(),
            ).await {
                Ok(quality_score) => {
                    let weight = if total_value > 0.0 { alloc.value / total_value } else { 0.0 };
                    let recommendation = self.build_recommendation(
                        quality_score,
                        goal,
                        risk_tolerance,
                        horizon_years,
                        weight,
                    );
                    recommendations.push(recommendation);
                }
                Err(e) => {
                    warn!("Could not compute quality score for {}: {}", alloc.ticker, e);
                }
            }
        }

        // 3. Filter by min quality if specified
        if let Some(min_q) = min_quality {
            recommendations.retain(|r| r.quality_score.composite_score >= min_q);
        }

        // 4. Sort by goal suitability (descending)
        recommendations.sort_by(|a, b| b.goal_suitability.partial_cmp(&a.goal_suitability).unwrap_or(std::cmp::Ordering::Equal));

        // 5. Build allocation strategy
        let allocation_strategy = AllocationStrategy::for_profile(risk_tolerance, horizon_years);

        // 6. Build portfolio summary
        let summary = self.build_summary(&recommendations, total_value, &allocations, &holding_details);

        // 7. Assign suggested weights based on strategy
        self.assign_suggested_weights(&mut recommendations, &allocation_strategy);

        Ok(LongTermGuidanceResponse {
            portfolio_id: portfolio_id.to_string(),
            goal: goal.to_string(),
            risk_tolerance: format!("{:?}", risk_tolerance).to_lowercase(),
            horizon_years,
            allocation_strategy,
            recommendations,
            summary,
            analyzed_at: chrono::Utc::now(),
        })
    }

    /// Compute quality score for a single ticker
    async fn compute_quality_score(
        &self,
        ticker: &str,
        holding_name: Option<&str>,
        industry: Option<&str>,
    ) -> Result<QualityScore, String> {
        // Fetch price history
        let price_data = price_service::get_history(&self.pool, ticker)
            .await
            .map_err(|e| format!("Failed to fetch price data for {}: {}", ticker, e))?;

        if price_data.len() < 20 {
            return Err(format!("Insufficient price data for {} ({} points)", ticker, price_data.len()));
        }

        // Convert to f64 prices in chronological order (oldest first)
        let mut prices: Vec<f64> = price_data
            .iter()
            .filter_map(|p| p.close_price.to_string().parse::<f64>().ok())
            .collect();
        prices.reverse();

        if prices.len() < 20 {
            return Err(format!("Insufficient valid price data for {}", ticker));
        }

        // Compute daily returns
        let returns: Vec<f64> = prices
            .windows(2)
            .map(|w| (w[1] - w[0]) / w[0])
            .collect();

        // Compute component scores
        let growth_metrics = self.compute_growth_metrics(&prices, &returns);
        let dividend_metrics = self.compute_dividend_metrics(&prices, &returns, ticker).await;
        let moat_indicators = self.compute_moat_indicators(&prices, &returns);
        let management_metrics = self.compute_management_metrics(&prices, &returns);

        // Score each component (0-100)
        let growth_score = self.score_growth(&growth_metrics);
        let dividend_score = self.score_dividend(&dividend_metrics);
        let moat_score = self.score_moat(&moat_indicators);
        let management_score = self.score_management(&management_metrics);

        // Composite score: weighted average
        let composite_score = growth_score * 0.30
            + dividend_score * 0.20
            + moat_score * 0.25
            + management_score * 0.25;

        let quality_tier = QualityTier::from_score(composite_score);

        Ok(QualityScore {
            ticker: ticker.to_string(),
            holding_name: holding_name.map(|s| s.to_string()),
            industry: industry.map(|s| s.to_string()),
            growth_score,
            dividend_score,
            moat_score,
            management_score,
            composite_score,
            quality_tier,
            growth_metrics,
            dividend_metrics,
            moat_indicators,
            management_metrics,
        })
    }

    // ── Growth Metrics ───────────────────────────────────────────────

    fn compute_growth_metrics(&self, prices: &[f64], returns: &[f64]) -> GrowthMetrics {
        let n = prices.len();
        let trading_days_per_year = 252.0;

        // Annualized return
        let first = prices[0];
        let last = prices[n - 1];
        let years = n as f64 / trading_days_per_year;
        let cagr = if years > 0.0 && first > 0.0 && last > 0.0 {
            (last / first).powf(1.0 / years) - 1.0
        } else {
            0.0
        };
        let annualized_return = cagr * 100.0;

        // Return consistency: R-squared of cumulative returns vs linear fit
        let cum_returns: Vec<f64> = returns
            .iter()
            .scan(1.0, |acc, &r| {
                *acc *= 1.0 + r;
                Some(*acc)
            })
            .collect();

        let return_consistency = self.r_squared(&cum_returns);

        // 1-year return
        let return_1y = if n >= 252 {
            let idx = n - 252;
            Some(((prices[n - 1] - prices[idx]) / prices[idx]) * 100.0)
        } else {
            None
        };

        // 3-year return (annualized)
        let return_3y = if n >= 756 {
            let idx = n - 756;
            let ret_3y = prices[n - 1] / prices[idx];
            Some((ret_3y.powf(1.0 / 3.0) - 1.0) * 100.0)
        } else {
            None
        };

        GrowthMetrics {
            annualized_return,
            return_consistency,
            return_1y,
            return_3y,
            cagr: cagr * 100.0,
        }
    }

    // ── Dividend Metrics ─────────────────────────────────────────────

    async fn compute_dividend_metrics(
        &self,
        prices: &[f64],
        returns: &[f64],
        _ticker: &str,
    ) -> DividendMetrics {
        // Without direct dividend data, we estimate from price behavior:
        // - Stocks with stable, positive returns and low volatility are likely dividend payers
        // - We look for patterns consistent with income-generating securities

        let mean_return = if returns.is_empty() { 0.0 } else {
            returns.iter().sum::<f64>() / returns.len() as f64
        };
        let positive_ratio = if returns.is_empty() { 0.0 } else {
            returns.iter().filter(|&&r| r > 0.0).count() as f64 / returns.len() as f64
        };

        let volatility = self.compute_volatility(returns);

        // Estimate: low vol + positive returns + stability = likely dividend payer
        let has_positive_income = mean_return > 0.0 && positive_ratio > 0.48;
        let estimated_yield = if has_positive_income && volatility < 25.0 {
            // Rough proxy: dividend stocks tend to have lower total return variance
            Some((mean_return * 252.0 * 100.0).min(8.0).max(0.0))
        } else {
            None
        };

        // Payout sustainability: higher for stocks with consistent positive returns and low vol
        let payout_sustainability = if volatility < 15.0 && positive_ratio > 0.50 {
            0.9
        } else if volatility < 25.0 && positive_ratio > 0.48 {
            0.7
        } else if positive_ratio > 0.45 {
            0.5
        } else {
            0.3
        };

        // Growth indicator: positive price trend over time
        let n = prices.len();
        let growth_indicator = if n >= 60 {
            let recent = prices[n - 20..].iter().sum::<f64>() / 20.0;
            let older = prices[n - 60..n - 40].iter().sum::<f64>() / 20.0;
            if older > 0.0 {
                ((recent / older) - 1.0).max(-0.5).min(0.5)
            } else {
                0.0
            }
        } else {
            0.0
        };

        DividendMetrics {
            has_positive_income,
            estimated_yield,
            payout_sustainability,
            growth_indicator,
        }
    }

    // ── Moat Indicators ──────────────────────────────────────────────

    fn compute_moat_indicators(&self, prices: &[f64], returns: &[f64]) -> MoatIndicators {
        let volatility = self.compute_volatility(returns);

        // Price stability: inverse of volatility, normalized to 0-1
        let price_stability = (1.0 - (volatility / 50.0)).max(0.0).min(1.0);

        // Margin strength: proportion of positive returns (higher = more consistent profits)
        let positive_ratio = if returns.is_empty() { 0.0 } else {
            returns.iter().filter(|&&r| r > 0.0).count() as f64 / returns.len() as f64
        };
        let margin_strength = positive_ratio;

        // Relative strength: overall trend direction and magnitude
        let n = prices.len();
        let relative_strength = if n >= 60 {
            let recent_avg = prices[n - 20..].iter().sum::<f64>() / 20.0;
            let overall_avg = prices.iter().sum::<f64>() / n as f64;
            if overall_avg > 0.0 {
                ((recent_avg / overall_avg) - 1.0).max(-1.0).min(1.0)
            } else {
                0.0
            }
        } else {
            0.0
        };

        // Market presence: based on data availability and price level (proxy)
        let market_presence = if n >= 504 {
            1.0 // 2+ years of data
        } else if n >= 252 {
            0.8 // 1+ year
        } else if n >= 126 {
            0.6 // 6+ months
        } else {
            0.4
        };

        MoatIndicators {
            price_stability,
            margin_strength,
            relative_strength,
            market_presence,
        }
    }

    // ── Management Metrics ───────────────────────────────────────────

    fn compute_management_metrics(&self, prices: &[f64], returns: &[f64]) -> ManagementMetrics {
        let mean_return = if returns.is_empty() { 0.0 } else {
            returns.iter().sum::<f64>() / returns.len() as f64
        };
        let volatility = self.compute_volatility(returns);

        // Capital efficiency: risk-adjusted return (Sharpe-like)
        let daily_rf = self.risk_free_rate / 252.0;
        let capital_efficiency = if volatility > 0.0 {
            ((mean_return - daily_rf) / (volatility / 100.0)).max(-2.0).min(2.0)
        } else {
            0.0
        };

        // Recovery speed: how quickly the stock recovers from drawdowns
        let recovery_speed = self.compute_recovery_speed(prices);

        // Return consistency: proportion of positive monthly returns
        let monthly_returns = self.compute_monthly_returns(prices);
        let return_consistency = if monthly_returns.is_empty() { 0.0 } else {
            monthly_returns.iter().filter(|&&r| r > 0.0).count() as f64 / monthly_returns.len() as f64
        };

        ManagementMetrics {
            capital_efficiency,
            recovery_speed,
            return_consistency,
        }
    }

    // ── Scoring Functions ────────────────────────────────────────────

    fn score_growth(&self, metrics: &GrowthMetrics) -> f64 {
        let mut score = 50.0; // Baseline

        // CAGR contribution (up to +30 points)
        if metrics.cagr > 20.0 {
            score += 30.0;
        } else if metrics.cagr > 10.0 {
            score += 20.0 + (metrics.cagr - 10.0);
        } else if metrics.cagr > 5.0 {
            score += 10.0 + (metrics.cagr - 5.0) * 2.0;
        } else if metrics.cagr > 0.0 {
            score += metrics.cagr * 2.0;
        } else {
            score += metrics.cagr.max(-30.0); // Penalty for negative growth
        }

        // Return consistency contribution (up to +20 points)
        score += metrics.return_consistency * 20.0;

        score.max(0.0).min(100.0)
    }

    fn score_dividend(&self, metrics: &DividendMetrics) -> f64 {
        let mut score = 30.0; // Baseline (not all stocks pay dividends)

        if metrics.has_positive_income {
            score += 15.0;
        }

        if let Some(yield_est) = metrics.estimated_yield {
            // Sweet spot: 2-5% yield
            if yield_est >= 2.0 && yield_est <= 5.0 {
                score += 25.0;
            } else if yield_est > 0.5 {
                score += 15.0;
            } else if yield_est > 5.0 {
                score += 10.0; // Very high yield may indicate risk
            }
        }

        // Payout sustainability
        score += metrics.payout_sustainability * 15.0;

        // Growth indicator
        if metrics.growth_indicator > 0.05 {
            score += 15.0;
        } else if metrics.growth_indicator > 0.0 {
            score += 10.0;
        }

        score.max(0.0).min(100.0)
    }

    fn score_moat(&self, indicators: &MoatIndicators) -> f64 {
        let mut score = 0.0;

        // Price stability (up to 30 points)
        score += indicators.price_stability * 30.0;

        // Margin strength (up to 25 points)
        score += indicators.margin_strength * 25.0;

        // Relative strength (up to 25 points)
        let rs_contribution = if indicators.relative_strength > 0.0 {
            indicators.relative_strength * 25.0
        } else {
            indicators.relative_strength * 15.0 // Less penalty for negative
        };
        score += rs_contribution;

        // Market presence (up to 20 points)
        score += indicators.market_presence * 20.0;

        score.max(0.0).min(100.0)
    }

    fn score_management(&self, metrics: &ManagementMetrics) -> f64 {
        let mut score = 0.0;

        // Capital efficiency (up to 35 points)
        // Normalized Sharpe-like ratio: 1.0 is good, 2.0 is excellent
        let ce_score = if metrics.capital_efficiency > 1.5 {
            35.0
        } else if metrics.capital_efficiency > 0.5 {
            20.0 + (metrics.capital_efficiency - 0.5) * 15.0
        } else if metrics.capital_efficiency > 0.0 {
            10.0 + metrics.capital_efficiency * 20.0
        } else {
            (10.0 + metrics.capital_efficiency * 5.0).max(0.0)
        };
        score += ce_score;

        // Recovery speed (up to 30 points)
        score += metrics.recovery_speed * 30.0;

        // Return consistency (up to 35 points)
        score += metrics.return_consistency * 35.0;

        score.max(0.0).min(100.0)
    }

    // ── Recommendation Builder ───────────────────────────────────────

    fn build_recommendation(
        &self,
        quality_score: QualityScore,
        goal: &InvestmentGoal,
        risk_tolerance: &RiskTolerance,
        horizon_years: i32,
        current_weight: f64,
    ) -> LongTermRecommendation {
        let volatility = (1.0 - quality_score.moat_indicators.price_stability) * 50.0;
        let risk_class = HoldingRiskClass::from_volatility_and_industry(
            volatility,
            quality_score.industry.as_deref(),
        );

        // Dividend aristocrat candidate: consistent positive income, high quality
        let dividend_aristocrat_candidate = quality_score.dividend_score >= 60.0
            && quality_score.dividend_metrics.payout_sustainability >= 0.7
            && quality_score.dividend_metrics.growth_indicator > 0.0;

        // Blue-chip candidate: high overall quality, stable, established
        let blue_chip_candidate = quality_score.composite_score >= 60.0
            && quality_score.moat_indicators.market_presence >= 0.8
            && quality_score.moat_indicators.price_stability >= 0.5;

        // Goal suitability
        let goal_suitability = self.compute_goal_suitability(
            &quality_score, &risk_class, goal, risk_tolerance, horizon_years,
        );

        // Build rationale
        let rationale = self.build_rationale(
            &quality_score, &risk_class, goal, dividend_aristocrat_candidate, blue_chip_candidate,
        );

        LongTermRecommendation {
            ticker: quality_score.ticker.clone(),
            holding_name: quality_score.holding_name.clone(),
            industry: quality_score.industry.clone(),
            quality_score,
            risk_class,
            dividend_aristocrat_candidate,
            blue_chip_candidate,
            goal_suitability,
            rationale,
            suggested_weight: current_weight, // Will be adjusted later
        }
    }

    fn compute_goal_suitability(
        &self,
        quality: &QualityScore,
        risk_class: &HoldingRiskClass,
        goal: &InvestmentGoal,
        risk_tolerance: &RiskTolerance,
        horizon_years: i32,
    ) -> f64 {
        let base = quality.composite_score;

        let goal_bonus = match goal {
            InvestmentGoal::Retirement => {
                // Retirement favors income and stability
                let income_bonus = quality.dividend_score * 0.3;
                let stability_bonus = quality.moat_score * 0.2;
                income_bonus + stability_bonus
            }
            InvestmentGoal::College => {
                // College savings: medium horizon, balanced growth and safety
                let growth_bonus = quality.growth_score * 0.2;
                let stability_bonus = quality.moat_score * 0.15;
                growth_bonus + stability_bonus
            }
            InvestmentGoal::Wealth => {
                // Wealth building: maximize growth
                let growth_bonus = quality.growth_score * 0.3;
                let management_bonus = quality.management_score * 0.15;
                growth_bonus + management_bonus
            }
        };

        // Risk alignment bonus/penalty
        let risk_alignment = match (risk_tolerance, risk_class) {
            (RiskTolerance::Conservative, HoldingRiskClass::Low) => 10.0,
            (RiskTolerance::Conservative, HoldingRiskClass::Medium) => 0.0,
            (RiskTolerance::Conservative, HoldingRiskClass::High) => -15.0,
            (RiskTolerance::Moderate, HoldingRiskClass::Low) => 5.0,
            (RiskTolerance::Moderate, HoldingRiskClass::Medium) => 10.0,
            (RiskTolerance::Moderate, HoldingRiskClass::High) => -5.0,
            (RiskTolerance::Aggressive, HoldingRiskClass::Low) => -5.0,
            (RiskTolerance::Aggressive, HoldingRiskClass::Medium) => 5.0,
            (RiskTolerance::Aggressive, HoldingRiskClass::High) => 10.0,
        };

        // Horizon adjustment: longer horizon = more tolerance for growth/volatility
        let horizon_bonus = if horizon_years > 15 {
            quality.growth_score * 0.1
        } else if horizon_years > 5 {
            0.0
        } else {
            quality.dividend_score * 0.05 // Short horizon: favor income
        };

        (base * 0.5 + goal_bonus + risk_alignment + horizon_bonus).max(0.0).min(100.0)
    }

    fn build_rationale(
        &self,
        quality: &QualityScore,
        risk_class: &HoldingRiskClass,
        goal: &InvestmentGoal,
        is_dividend_aristocrat: bool,
        is_blue_chip: bool,
    ) -> String {
        let mut parts = Vec::new();

        // Quality assessment
        match quality.quality_tier {
            QualityTier::Premium => parts.push(format!(
                "{} is a premium-quality holding (score: {:.0}/100)",
                quality.ticker, quality.composite_score
            )),
            QualityTier::High => parts.push(format!(
                "{} is a high-quality holding (score: {:.0}/100)",
                quality.ticker, quality.composite_score
            )),
            QualityTier::Medium => parts.push(format!(
                "{} is a medium-quality holding (score: {:.0}/100)",
                quality.ticker, quality.composite_score
            )),
            QualityTier::Low => parts.push(format!(
                "{} has below-average quality metrics (score: {:.0}/100)",
                quality.ticker, quality.composite_score
            )),
        }

        // Special designations
        if is_dividend_aristocrat {
            parts.push("Qualifies as a dividend aristocrat candidate with consistent income generation.".to_string());
        }
        if is_blue_chip {
            parts.push("Meets blue-chip criteria: established market presence with stable performance.".to_string());
        }

        // Growth commentary
        if quality.growth_metrics.cagr > 15.0 {
            parts.push(format!("Strong growth trajectory with {:.1}% CAGR.", quality.growth_metrics.cagr));
        } else if quality.growth_metrics.cagr > 5.0 {
            parts.push(format!("Moderate growth at {:.1}% CAGR.", quality.growth_metrics.cagr));
        } else if quality.growth_metrics.cagr < 0.0 {
            parts.push(format!("Negative growth trend ({:.1}% CAGR) warrants review.", quality.growth_metrics.cagr));
        }

        // Risk classification
        parts.push(format!("Risk classification: {} risk.", risk_class));

        // Goal-specific advice
        match goal {
            InvestmentGoal::Retirement => {
                if *risk_class == HoldingRiskClass::Low && quality.dividend_score >= 60.0 {
                    parts.push("Well-suited for retirement portfolios with income focus.".to_string());
                }
            }
            InvestmentGoal::College => {
                if quality.growth_score >= 60.0 && *risk_class != HoldingRiskClass::High {
                    parts.push("Appropriate for college savings with balanced growth profile.".to_string());
                }
            }
            InvestmentGoal::Wealth => {
                if quality.growth_score >= 70.0 {
                    parts.push("Strong candidate for long-term wealth building.".to_string());
                }
            }
        }

        parts.join(" ")
    }

    // ── Summary Builder ──────────────────────────────────────────────

    fn build_summary(
        &self,
        recommendations: &[LongTermRecommendation],
        total_value: f64,
        allocations: &[db::analytics_queries::AllocationRow],
        _holding_details: &[HoldingDetail],
    ) -> PortfolioGuidanceSummary {
        let dividend_aristocrat_count = recommendations.iter()
            .filter(|r| r.dividend_aristocrat_candidate).count();
        let blue_chip_count = recommendations.iter()
            .filter(|r| r.blue_chip_candidate).count();

        let average_quality_score = if recommendations.is_empty() {
            0.0
        } else {
            recommendations.iter()
                .map(|r| r.quality_score.composite_score)
                .sum::<f64>() / recommendations.len() as f64
        };

        // Current risk allocation
        let mut low_value = 0.0;
        let mut medium_value = 0.0;
        let mut high_value = 0.0;

        for rec in recommendations {
            let alloc_value = allocations.iter()
                .find(|a| a.ticker == rec.ticker)
                .map(|a| a.value)
                .unwrap_or(0.0);

            match rec.risk_class {
                HoldingRiskClass::Low => low_value += alloc_value,
                HoldingRiskClass::Medium => medium_value += alloc_value,
                HoldingRiskClass::High => high_value += alloc_value,
            }
        }

        let current_risk_allocation = if total_value > 0.0 {
            CurrentRiskAllocation {
                low_risk_pct: (low_value / total_value) * 100.0,
                medium_risk_pct: (medium_value / total_value) * 100.0,
                high_risk_pct: (high_value / total_value) * 100.0,
            }
        } else {
            CurrentRiskAllocation {
                low_risk_pct: 0.0,
                medium_risk_pct: 0.0,
                high_risk_pct: 0.0,
            }
        };

        // Diversification assessment
        let num_holdings = recommendations.len();
        let unique_industries: std::collections::HashSet<_> = recommendations.iter()
            .filter_map(|r| r.industry.as_deref())
            .collect();
        let diversification_rating = if num_holdings >= 15 && unique_industries.len() >= 5 {
            "Excellent - Well diversified across holdings and industries".to_string()
        } else if num_holdings >= 8 && unique_industries.len() >= 3 {
            "Good - Reasonable diversification".to_string()
        } else if num_holdings >= 5 {
            "Fair - Consider adding more sector diversity".to_string()
        } else {
            "Needs improvement - Portfolio is concentrated in few holdings".to_string()
        };

        // Suggestions
        let mut suggestions = Vec::new();

        if dividend_aristocrat_count == 0 {
            suggestions.push("Consider adding dividend-paying stocks for income stability.".to_string());
        }
        if blue_chip_count < 3 && num_holdings >= 5 {
            suggestions.push("Portfolio could benefit from more blue-chip holdings for stability.".to_string());
        }
        if current_risk_allocation.high_risk_pct > 60.0 {
            suggestions.push("High-risk allocation exceeds 60%. Consider rebalancing toward more stable holdings.".to_string());
        }
        if current_risk_allocation.low_risk_pct < 10.0 && num_holdings >= 5 {
            suggestions.push("Very low allocation to defensive holdings. Consider adding some low-risk positions.".to_string());
        }
        if average_quality_score < 50.0 {
            suggestions.push("Average portfolio quality is below median. Review holdings with lowest quality scores.".to_string());
        }
        if unique_industries.len() < 3 && num_holdings >= 5 {
            suggestions.push("Industry concentration detected. Diversifying across sectors can reduce risk.".to_string());
        }

        if suggestions.is_empty() {
            suggestions.push("Portfolio is well-positioned for long-term investing.".to_string());
        }

        PortfolioGuidanceSummary {
            dividend_aristocrat_count,
            blue_chip_count,
            average_quality_score,
            diversification_rating,
            current_risk_allocation,
            suggestions,
        }
    }

    // ── Weight Assignment ────────────────────────────────────────────

    fn assign_suggested_weights(
        &self,
        recommendations: &mut [LongTermRecommendation],
        strategy: &AllocationStrategy,
    ) {
        if recommendations.is_empty() {
            return;
        }

        // Group by risk class
        let mut low_risk: Vec<usize> = Vec::new();
        let mut med_risk: Vec<usize> = Vec::new();
        let mut high_risk: Vec<usize> = Vec::new();

        for (i, rec) in recommendations.iter().enumerate() {
            match rec.risk_class {
                HoldingRiskClass::Low => low_risk.push(i),
                HoldingRiskClass::Medium => med_risk.push(i),
                HoldingRiskClass::High => high_risk.push(i),
            }
        }

        // Distribute allocation within each risk group by quality score
        self.distribute_weights(recommendations, &low_risk, strategy.low_risk_allocation);
        self.distribute_weights(recommendations, &med_risk, strategy.medium_risk_allocation);
        self.distribute_weights(recommendations, &high_risk, strategy.high_risk_allocation);
    }

    fn distribute_weights(
        &self,
        recommendations: &mut [LongTermRecommendation],
        indices: &[usize],
        total_allocation: f64,
    ) {
        if indices.is_empty() {
            return;
        }

        let total_quality: f64 = indices.iter()
            .map(|&i| recommendations[i].quality_score.composite_score.max(1.0))
            .sum();

        for &i in indices {
            let quality = recommendations[i].quality_score.composite_score.max(1.0);
            recommendations[i].suggested_weight = if total_quality > 0.0 {
                (quality / total_quality) * total_allocation
            } else {
                total_allocation / indices.len() as f64
            };
        }
    }

    // ── Helper Functions ─────────────────────────────────────────────

    fn compute_volatility(&self, returns: &[f64]) -> f64 {
        if returns.is_empty() {
            return 0.0;
        }
        let mean = returns.iter().sum::<f64>() / returns.len() as f64;
        let variance = returns.iter()
            .map(|r| (r - mean).powi(2))
            .sum::<f64>() / returns.len() as f64;
        (variance.sqrt() * (252.0_f64).sqrt()) * 100.0
    }

    fn r_squared(&self, values: &[f64]) -> f64 {
        let n = values.len() as f64;
        if n < 3.0 {
            return 0.0;
        }

        let x_mean = (n - 1.0) / 2.0;
        let y_mean = values.iter().sum::<f64>() / n;

        let mut ss_xy = 0.0;
        let mut ss_xx = 0.0;
        let mut ss_yy = 0.0;

        for (i, &y) in values.iter().enumerate() {
            let x = i as f64;
            ss_xy += (x - x_mean) * (y - y_mean);
            ss_xx += (x - x_mean).powi(2);
            ss_yy += (y - y_mean).powi(2);
        }

        if ss_xx == 0.0 || ss_yy == 0.0 {
            return 0.0;
        }

        let r = ss_xy / (ss_xx.sqrt() * ss_yy.sqrt());
        (r * r).min(1.0).max(0.0)
    }

    fn compute_recovery_speed(&self, prices: &[f64]) -> f64 {
        if prices.len() < 20 {
            return 0.5;
        }

        let mut max_price = prices[0];
        let mut in_drawdown = false;
        let mut recovery_times: Vec<usize> = Vec::new();
        let mut drawdown_start = 0;

        for (i, &price) in prices.iter().enumerate() {
            if price >= max_price {
                if in_drawdown {
                    recovery_times.push(i - drawdown_start);
                    in_drawdown = false;
                }
                max_price = price;
            } else {
                let drawdown = (max_price - price) / max_price;
                if drawdown > 0.05 && !in_drawdown {
                    in_drawdown = true;
                    drawdown_start = i;
                }
            }
        }

        if recovery_times.is_empty() {
            return 0.8; // No significant drawdowns = good
        }

        let avg_recovery = recovery_times.iter().sum::<usize>() as f64 / recovery_times.len() as f64;

        // Normalize: fast recovery (< 20 days) = 1.0, slow (> 120 days) = 0.1
        if avg_recovery < 20.0 {
            1.0
        } else if avg_recovery < 60.0 {
            0.7
        } else if avg_recovery < 120.0 {
            0.4
        } else {
            0.1
        }
    }

    fn compute_monthly_returns(&self, prices: &[f64]) -> Vec<f64> {
        let step = 21; // Approximate monthly trading days
        let mut monthly_returns = Vec::new();

        let mut i = 0;
        while i + step < prices.len() {
            let ret = (prices[i + step] - prices[i]) / prices[i];
            monthly_returns.push(ret);
            i += step;
        }

        monthly_returns
    }

    /// Fetch holding details (name, industry) from latest snapshot
    async fn fetch_holding_details(&self, portfolio_id: Uuid) -> Result<Vec<HoldingDetail>, String> {
        let rows = sqlx::query_as::<_, HoldingDetail>(
            r#"
            WITH latest AS (
                SELECT MAX(h.snapshot_date) as snapshot_date
                FROM holdings_snapshots h
                JOIN accounts a ON h.account_id = a.id
                WHERE a.portfolio_id = $1
            )
            SELECT DISTINCT ON (h.ticker)
                h.ticker,
                h.holding_name,
                h.industry,
                h.asset_category,
                h.market_value::double precision as market_value
            FROM holdings_snapshots h
            JOIN accounts a ON h.account_id = a.id
            JOIN latest l ON h.snapshot_date = l.snapshot_date
            WHERE a.portfolio_id = $1
              AND h.ticker != ''
            ORDER BY h.ticker, h.market_value DESC
            "#,
        )
        .bind(portfolio_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to fetch holding details: {}", e))?;

        Ok(rows)
    }
}

/// Internal struct for holding details
#[derive(Debug, Clone, sqlx::FromRow)]
#[allow(dead_code)]
pub struct HoldingDetail {
    pub ticker: String,
    pub holding_name: Option<String>,
    pub industry: Option<String>,
    pub asset_category: Option<String>,
    pub market_value: Option<f64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quality_tier_from_score() {
        assert_eq!(QualityTier::from_score(85.0), QualityTier::Premium);
        assert_eq!(QualityTier::from_score(65.0), QualityTier::High);
        assert_eq!(QualityTier::from_score(45.0), QualityTier::Medium);
        assert_eq!(QualityTier::from_score(20.0), QualityTier::Low);
    }

    #[test]
    fn test_risk_class_from_volatility() {
        assert_eq!(
            HoldingRiskClass::from_volatility_and_industry(10.0, None),
            HoldingRiskClass::Low
        );
        assert_eq!(
            HoldingRiskClass::from_volatility_and_industry(20.0, None),
            HoldingRiskClass::Medium
        );
        assert_eq!(
            HoldingRiskClass::from_volatility_and_industry(35.0, None),
            HoldingRiskClass::High
        );
    }

    #[test]
    fn test_risk_class_from_industry() {
        assert_eq!(
            HoldingRiskClass::from_volatility_and_industry(25.0, Some("Utilities")),
            HoldingRiskClass::Low
        );
        assert_eq!(
            HoldingRiskClass::from_volatility_and_industry(15.0, Some("Technology")),
            HoldingRiskClass::High
        );
    }

    #[test]
    fn test_allocation_strategy_conservative() {
        let strategy = AllocationStrategy::for_profile(&RiskTolerance::Conservative, 20);
        let total = strategy.low_risk_allocation + strategy.medium_risk_allocation + strategy.high_risk_allocation;
        assert!((total - 1.0).abs() < 0.01, "Allocations should sum to ~1.0, got {}", total);
        assert!(strategy.low_risk_allocation > strategy.high_risk_allocation);
    }

    #[test]
    fn test_allocation_strategy_aggressive() {
        let strategy = AllocationStrategy::for_profile(&RiskTolerance::Aggressive, 20);
        let total = strategy.low_risk_allocation + strategy.medium_risk_allocation + strategy.high_risk_allocation;
        assert!((total - 1.0).abs() < 0.01, "Allocations should sum to ~1.0, got {}", total);
        assert!(strategy.high_risk_allocation > strategy.low_risk_allocation);
    }

    #[test]
    fn test_investment_goal_display() {
        assert_eq!(InvestmentGoal::Retirement.to_string(), "retirement");
        assert_eq!(InvestmentGoal::College.to_string(), "college");
        assert_eq!(InvestmentGoal::Wealth.to_string(), "wealth");
    }

    #[test]
    fn test_goal_parsing() {
        assert_eq!(InvestmentGoal::from_str_opt("retirement"), Some(InvestmentGoal::Retirement));
        assert_eq!(InvestmentGoal::from_str_opt("COLLEGE"), Some(InvestmentGoal::College));
        assert_eq!(InvestmentGoal::from_str_opt("invalid"), None);
    }

    #[test]
    fn test_risk_tolerance_parsing() {
        assert_eq!(RiskTolerance::from_str_opt("conservative"), Some(RiskTolerance::Conservative));
        assert_eq!(RiskTolerance::from_str_opt("MODERATE"), Some(RiskTolerance::Moderate));
        assert_eq!(RiskTolerance::from_str_opt("invalid"), None);
    }

    #[test]
    fn test_r_squared() {
        let service = LongTermGuidanceService {
            pool: unsafe { std::mem::zeroed() }, // Not used in this test
            risk_free_rate: 0.045,
        };

        // Perfect linear data should have R-squared near 1.0
        let linear: Vec<f64> = (0..100).map(|i| i as f64 * 2.0 + 1.0).collect();
        let r2 = service.r_squared(&linear);
        assert!(r2 > 0.99, "Linear data R-squared should be near 1.0, got {}", r2);
    }

    #[test]
    fn test_compute_volatility() {
        let service = LongTermGuidanceService {
            pool: unsafe { std::mem::zeroed() },
            risk_free_rate: 0.045,
        };

        // Zero returns should give zero volatility
        let returns = vec![0.0; 100];
        assert_eq!(service.compute_volatility(&returns), 0.0);

        // Non-zero returns should give positive volatility
        let returns: Vec<f64> = (0..100).map(|i| if i % 2 == 0 { 0.01 } else { -0.01 }).collect();
        let vol = service.compute_volatility(&returns);
        assert!(vol > 0.0, "Volatility should be positive");
    }

    #[test]
    fn test_recovery_speed() {
        let service = LongTermGuidanceService {
            pool: unsafe { std::mem::zeroed() },
            risk_free_rate: 0.045,
        };

        // Steadily increasing prices: no drawdowns
        let prices: Vec<f64> = (1..=100).map(|i| i as f64).collect();
        let speed = service.compute_recovery_speed(&prices);
        assert!(speed >= 0.8, "No drawdowns should yield high recovery speed");
    }

    #[test]
    fn test_monthly_returns() {
        let service = LongTermGuidanceService {
            pool: unsafe { std::mem::zeroed() },
            risk_free_rate: 0.045,
        };

        let prices: Vec<f64> = (0..100).map(|i| 100.0 + i as f64).collect();
        let monthly = service.compute_monthly_returns(&prices);
        assert!(!monthly.is_empty());
        for r in &monthly {
            assert!(*r > 0.0, "Steadily increasing prices should have positive monthly returns");
        }
    }

    #[test]
    fn test_growth_scoring() {
        let service = LongTermGuidanceService {
            pool: unsafe { std::mem::zeroed() },
            risk_free_rate: 0.045,
        };

        let high_growth = GrowthMetrics {
            annualized_return: 25.0,
            return_consistency: 0.9,
            return_1y: Some(30.0),
            return_3y: Some(20.0),
            cagr: 25.0,
        };
        let score = service.score_growth(&high_growth);
        assert!(score > 80.0, "High growth should score above 80, got {}", score);

        let negative_growth = GrowthMetrics {
            annualized_return: -10.0,
            return_consistency: 0.2,
            return_1y: Some(-15.0),
            return_3y: None,
            cagr: -10.0,
        };
        let score = service.score_growth(&negative_growth);
        assert!(score < 50.0, "Negative growth should score below 50, got {}", score);
    }
}
