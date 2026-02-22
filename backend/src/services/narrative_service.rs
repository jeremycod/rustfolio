use chrono::Utc;
use tracing::{info, warn};
use uuid::Uuid;

use crate::errors::{AppError, LlmError};
use crate::models::{PortfolioNarrative, PortfolioRisk};
use crate::services::llm_service::LlmService;
use std::sync::Arc;

/// Generate a narrative summary for a portfolio
pub async fn generate_portfolio_narrative(
    llm_service: Arc<LlmService>,
    user_id: Uuid,
    portfolio_risk: &PortfolioRisk,
    time_period: &str,
) -> Result<PortfolioNarrative, AppError> {
    info!("Generating narrative for portfolio (time_period: {})", time_period);

    // Check if LLM is enabled
    if !llm_service.is_enabled() {
        return Err(AppError::Llm(LlmError::Disabled));
    }

    // Build the prompt
    let prompt = build_narrative_prompt(portfolio_risk, time_period);

    // Generate completion with rate limiting
    let response = llm_service
        .generate_completion_for_user(user_id, prompt)
        .await?;

    // Parse the response
    parse_narrative_response(&response, portfolio_risk)
}

/// Build a detailed prompt for portfolio narrative generation
fn build_narrative_prompt(portfolio_risk: &PortfolioRisk, time_period: &str) -> String {
    let position_count = portfolio_risk.position_risks.len();
    let avg_volatility = if !portfolio_risk.position_risks.is_empty() {
        portfolio_risk.position_risks.iter()
            .map(|p| p.risk_assessment.metrics.volatility)
            .sum::<f64>() / position_count as f64
    } else {
        0.0
    };

    // Get top 3 positions by value
    let mut sorted_positions = portfolio_risk.position_risks.clone();
    sorted_positions.sort_by(|a, b| {
        b.market_value
            .partial_cmp(&a.market_value)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let top_positions: Vec<String> = sorted_positions
        .iter()
        .take(3)
        .map(|p| format!("{} (${:.0})", p.ticker, p.market_value))
        .collect();

    // Get highest risk positions
    let mut risk_sorted = portfolio_risk.position_risks.clone();
    risk_sorted.sort_by(|a, b| {
        b.risk_assessment.metrics.volatility
            .partial_cmp(&a.risk_assessment.metrics.volatility)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let high_risk_positions: Vec<String> = risk_sorted
        .iter()
        .take(3)
        .map(|p| format!("{} ({:.1}% volatility)", p.ticker, p.risk_assessment.metrics.volatility))
        .collect();

    format!(
        r#"Analyze this investment portfolio's {} performance and provide educational insights:

PORTFOLIO OVERVIEW:
- Total Value: ${:.2}
- Number of Positions: {}
- Portfolio Risk Score: {:.1}/100
- Portfolio Volatility: {:.2}%
- Average Position Volatility: {:.2}%

TOP HOLDINGS:
{}

HIGHEST RISK POSITIONS:
{}

INSTRUCTIONS:
Generate a concise portfolio analysis with the following sections. Use clear, educational language suitable for retail investors.

1. SUMMARY (2-3 sentences):
   - Overall portfolio health assessment
   - Key performance driver this {}
   - Primary risk consideration

2. PERFORMANCE EXPLANATION (2-3 sentences):
   - What factors drove the portfolio's current state
   - Notable position movements or sector trends
   - How diversification is working

3. RISK HIGHLIGHTS (3-4 bullet points):
   - List 3-4 specific risk factors to monitor
   - Each should be actionable and specific
   - Include both position-level and portfolio-level risks

4. TOP CONTRIBUTORS (3 items):
   - List 3 positions that most impact the portfolio (positive or negative)
   - Brief explanation of why each matters
   - Focus on their influence on overall portfolio metrics

IMPORTANT REQUIREMENTS:
- Use educational language, NOT investment advice
- Do NOT recommend buying or selling specific securities
- Do NOT predict future prices or returns
- Focus on risk analysis and portfolio construction insights
- Be objective and factual based on the metrics provided
- Keep the tone professional but accessible

Format your response as valid JSON with this structure:
{{
  "summary": "...",
  "performance_explanation": "...",
  "risk_highlights": ["...", "...", "..."],
  "top_contributors": ["...", "...", "..."]
}}"#,
        time_period,
        portfolio_risk.total_value,
        position_count,
        portfolio_risk.portfolio_risk_score,
        portfolio_risk.portfolio_volatility,
        avg_volatility,
        top_positions.join("\n"),
        high_risk_positions.join("\n"),
        time_period
    )
}

/// Parse the LLM response into a structured narrative
fn parse_narrative_response(
    response: &str,
    portfolio_risk: &PortfolioRisk,
) -> Result<PortfolioNarrative, AppError> {
    // Try to parse as JSON first
    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(response) {
        let summary = parsed["summary"]
            .as_str()
            .unwrap_or("Unable to generate summary")
            .to_string();

        let performance_explanation = parsed["performance_explanation"]
            .as_str()
            .unwrap_or("Unable to generate performance explanation")
            .to_string();

        let risk_highlights = parsed["risk_highlights"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(String::from)
                    .collect()
            })
            .unwrap_or_else(|| vec!["Risk analysis unavailable".to_string()]);

        let top_contributors = parsed["top_contributors"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(String::from)
                    .collect()
            })
            .unwrap_or_else(|| vec!["Contributor analysis unavailable".to_string()]);

        return Ok(PortfolioNarrative {
            summary,
            performance_explanation,
            risk_highlights,
            top_contributors,
            generated_at: Utc::now(),
        });
    }

    // Fallback: if JSON parsing fails, create a basic narrative
    warn!("Failed to parse LLM response as JSON, using fallback");

    Ok(PortfolioNarrative {
        summary: format!(
            "Your portfolio contains {} positions with a total value of ${:.2} and a risk score of {:.1}/100.",
            portfolio_risk.position_risks.len(),
            portfolio_risk.total_value,
            portfolio_risk.portfolio_risk_score
        ),
        performance_explanation: "AI-generated analysis is temporarily unavailable. Portfolio metrics are calculated from your historical data.".to_string(),
        risk_highlights: vec![
            format!("Portfolio volatility: {:.2}%", portfolio_risk.portfolio_volatility),
            format!("Portfolio max drawdown: {:.2}%", portfolio_risk.portfolio_max_drawdown),
            "Monitor individual position volatility regularly".to_string(),
        ],
        top_contributors: portfolio_risk
            .position_risks
            .iter()
            .take(3)
            .map(|p| format!("{}: ${:.2} ({:.1}% volatility)", p.ticker, p.market_value, p.risk_assessment.metrics.volatility))
            .collect(),
        generated_at: Utc::now(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{PositionRisk, PositionRiskContribution, RiskAssessment, RiskLevel};

    #[test]
    fn test_build_narrative_prompt() {
        let portfolio_risk = PortfolioRisk {
            portfolio_id: "test".to_string(),
            total_value: 100000.0,
            portfolio_volatility: 15.5,
            portfolio_max_drawdown: -12.0,
            portfolio_beta: Some(1.1),
            portfolio_sharpe: Some(1.3),
            portfolio_var_95: Some(-4.5),
            portfolio_var_99: Some(-7.0),
            portfolio_expected_shortfall_95: Some(-5.5),
            portfolio_expected_shortfall_99: Some(-8.5),
            portfolio_risk_score: 65.0,
            risk_level: RiskLevel::Moderate,
            position_risks: vec![
                PositionRiskContribution {
                    ticker: "AAPL".to_string(),
                    market_value: 50000.0,
                    weight: 0.5,
                    risk_assessment: RiskAssessment {
                        ticker: "AAPL".to_string(),
                        metrics: PositionRisk {
                            volatility: 20.0,
                            max_drawdown: -15.0,
                            beta: Some(1.2),
                            beta_spy: Some(1.2),
                            beta_qqq: None,
                            beta_iwm: None,
                            risk_decomposition: None,
                            sharpe: Some(1.5),
                            sortino: Some(2.0),
                            annualized_return: Some(12.0),
                            value_at_risk: Some(-5.0),
                            var_95: Some(-5.0),
                            var_99: Some(-8.0),
                            expected_shortfall_95: Some(-6.0),
                            expected_shortfall_99: Some(-9.0),
                        },
                        risk_score: 60.0,
                        risk_level: RiskLevel::Moderate,
                    },
                },
            ],
        };

        let prompt = build_narrative_prompt(&portfolio_risk, "30 days");

        assert!(prompt.contains("Total Value: $100000.00"));
        assert!(prompt.contains("Portfolio Risk Score: 65.0/100"));
        assert!(prompt.contains("AAPL"));
        assert!(prompt.contains("valid JSON"));
    }
}
