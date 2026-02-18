use std::sync::Arc;
use sqlx::PgPool;
use tracing::{info, warn};
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::{PortfolioQuestion, PortfolioAnswer, Confidence};
use crate::services::llm_service::LlmService;

/// Build comprehensive context about a portfolio for Q&A
pub async fn build_portfolio_context(
    pool: &PgPool,
    portfolio_id: Uuid,
) -> Result<String, AppError> {
    info!("Building Q&A context for portfolio {}", portfolio_id);

    // 1. Get portfolio basic info
    let portfolio = crate::db::portfolio_queries::fetch_one(pool, portfolio_id)
        .await
        .map_err(AppError::Db)?
        .ok_or_else(|| AppError::External("Portfolio not found".to_string()))?;

    // 2. Get holdings
    let holdings = crate::db::holding_snapshot_queries::fetch_portfolio_latest_holdings(
        pool,
        portfolio_id,
    )
    .await
    .map_err(AppError::Db)?;

    let total_value: f64 = holdings
        .iter()
        .filter_map(|h| h.market_value.to_string().parse::<f64>().ok())
        .sum();

    let position_count = holdings.len();

    // 3. Build context string
    let mut context = format!(
        "PORTFOLIO INFORMATION:\n\
         Portfolio Name: {}\n\
         Portfolio ID: {}\n\
         Total Value: ${:.2}\n\
         Number of Positions: {}\n\n",
        portfolio.name, portfolio.id, total_value, position_count
    );

    // 4. Add top holdings
    if !holdings.is_empty() {
        context.push_str("TOP HOLDINGS:\n");
        for (i, holding) in holdings.iter().take(10).enumerate() {
            let value = holding.market_value.to_string().parse::<f64>().unwrap_or(0.0);
            let weight = if total_value > 0.0 {
                (value / total_value) * 100.0
            } else {
                0.0
            };
            context.push_str(&format!(
                "{}. {} - ${:.2} ({:.1}%)\n",
                i + 1,
                holding.ticker,
                value,
                weight
            ));
        }
        context.push('\n');
    }

    // 5. Add recent performance (if available via analytics)
    // This is a simplified version - in production you'd fetch actual performance data
    context.push_str("ADDITIONAL CONTEXT:\n");
    context.push_str("- Portfolio risk metrics are available via the risk analysis system\n");
    context.push_str("- Historical performance data is tracked\n");
    context.push_str("- News and market sentiment analysis is available\n");

    Ok(context)
}

/// Generate an answer to a portfolio question using LLM
pub async fn answer_portfolio_question(
    llm_service: Arc<LlmService>,
    pool: &PgPool,
    user_id: Uuid,
    portfolio_id: Uuid,
    question: PortfolioQuestion,
) -> Result<PortfolioAnswer, AppError> {
    info!(
        "Answering question for portfolio {}: {}",
        portfolio_id, question.question
    );

    // Check if LLM is enabled
    if !llm_service.is_enabled() {
        return Err(AppError::External(
            "Q&A assistant requires LLM service to be enabled".to_string(),
        ));
    }

    // Build portfolio context
    let context = build_portfolio_context(pool, portfolio_id).await?;

    // Build the Q&A prompt
    let prompt = build_qa_prompt(&context, &question);

    // Generate answer
    let response = llm_service
        .generate_completion_for_user(user_id, prompt)
        .await?;

    // Parse the response
    parse_qa_response(&response, question)
}

/// Build a prompt for Q&A
fn build_qa_prompt(context: &str, question: &PortfolioQuestion) -> String {
    let context_hint = question
        .context_hint
        .as_deref()
        .unwrap_or("general portfolio question");

    format!(
        r#"You are a portfolio analysis assistant. Answer the user's question using the portfolio data provided.

{}

USER CONTEXT: {}

USER QUESTION: {}

Provide a helpful, educational answer. Format your response as valid JSON:
{{
  "answer": "Your detailed answer here (2-4 sentences)",
  "sources": ["Portfolio holdings", "Risk metrics", etc.],
  "confidence": "high|medium|low",
  "follow_up_questions": [
    "Suggested follow-up question 1",
    "Suggested follow-up question 2",
    "Suggested follow-up question 3"
  ]
}}

IMPORTANT GUIDELINES:
- Be educational and factual, NOT investment advice
- Do NOT recommend specific buy/sell actions
- Cite which data sources informed your answer
- If you're unsure, say so (use "low" confidence)
- Suggest relevant follow-up questions
- Keep answers concise (2-4 sentences)
- Focus on helping users understand their portfolio"#,
        context, context_hint, question.question
    )
}

#[derive(Debug, serde::Deserialize)]
struct QAResponse {
    answer: String,
    sources: Vec<String>,
    confidence: String,
    follow_up_questions: Vec<String>,
}

/// Parse the LLM response into a PortfolioAnswer
fn parse_qa_response(
    response: &str,
    _question: PortfolioQuestion,
) -> Result<PortfolioAnswer, AppError> {
    // Try to parse JSON response
    match serde_json::from_str::<QAResponse>(response) {
        Ok(parsed) => {
            let confidence = match parsed.confidence.to_lowercase().as_str() {
                "high" => Confidence::High,
                "low" => Confidence::Low,
                _ => Confidence::Medium,
            };

            Ok(PortfolioAnswer {
                answer: parsed.answer,
                sources: parsed.sources,
                confidence,
                follow_up_questions: parsed.follow_up_questions,
                generated_at: chrono::Utc::now(),
            })
        }
        Err(e) => {
            warn!("Failed to parse Q&A response: {}", e);
            // Fallback: return the raw response
            Ok(PortfolioAnswer {
                answer: response.to_string(),
                sources: vec!["Portfolio data".to_string()],
                confidence: Confidence::Medium,
                follow_up_questions: vec![
                    "What are my biggest holdings?".to_string(),
                    "How is my portfolio performing?".to_string(),
                    "What are the main risks?".to_string(),
                ],
                generated_at: chrono::Utc::now(),
            })
        }
    }
}
