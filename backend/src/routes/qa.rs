use axum::extract::{Path, State};
use axum::{Json, Router};
use axum::routing::post;
use tracing::{error, info};
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::{PortfolioQuestion, PortfolioAnswer};
use crate::services::qa_service;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/portfolios/:portfolio_id/ask", post(ask_question))
}

/// POST /api/qa/portfolios/:portfolio_id/ask
///
/// Ask a question about the portfolio and get an AI-powered answer
///
/// Request body: PortfolioQuestion
/// {
///   "question": "What are my top holdings?",
///   "context_hint": "Looking at risk analysis" (optional)
/// }
///
/// Returns: PortfolioAnswer with answer, sources, confidence, and follow-up questions
async fn ask_question(
    Path(portfolio_id): Path<Uuid>,
    State(state): State<AppState>,
    Json(question): Json<PortfolioQuestion>,
) -> Result<Json<PortfolioAnswer>, AppError> {
    info!(
        "POST /api/qa/portfolios/{}/ask - Question: {}",
        portfolio_id, question.question
    );

    // Use demo user ID (in production, extract from auth token)
    let demo_user_id = Uuid::parse_str("00000000-0000-0000-0000-000000000001")
        .expect("Invalid demo user UUID");

    let answer = qa_service::answer_portfolio_question(
        state.llm_service.clone(),
        &state.pool,
        demo_user_id,
        portfolio_id,
        question,
    )
    .await
    .map_err(|e| {
        error!("Failed to answer question: {}", e);
        e
    })?;

    info!(
        "Successfully answered question with {} confidence",
        answer.confidence
    );

    Ok(Json(answer))
}
