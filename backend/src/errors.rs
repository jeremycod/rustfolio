use axum::http::{HeaderMap, HeaderValue};
use axum::response::IntoResponse;
use reqwest::StatusCode;
use sqlx::Error;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    Db(sqlx::Error),
    #[error("Validation error: {0}")]
    Validation(String),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Rate limited by external provider")]
    RateLimited,
    #[error("External service error: {0}")]
    External(String),
    #[error("Unauthorized")]
    #[allow(dead_code)]
    Unauthorized,
    #[error("LLM error: {0}")]
    Llm(LlmError),
}

#[derive(Debug, Error)]
pub enum LlmError {
    #[error("LLM API error: {0}")]
    ApiError(String),
    #[error("LLM rate limited")]
    RateLimited,
    #[error("LLM features are disabled")]
    Disabled,
    #[error("Invalid LLM response: {0}")]
    InvalidResponse(String),
    #[error("Network error: {0}")]
    NetworkError(String),
    #[error("Timeout")]
    Timeout,
}



impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        match self {
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg).into_response(),
            AppError::Validation(msg) => (StatusCode::BAD_REQUEST, msg).into_response(),
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, "Unauthorized").into_response(),
            AppError::RateLimited => {
                let mut headers = HeaderMap::new();
                headers.insert("Retry-After", HeaderValue::from_static("60"));
                (StatusCode::TOO_MANY_REQUESTS, headers, "Rate limited").into_response()
            },
            // Use 503 Service Unavailable only for actual external service failures
            AppError::External(msg) => (StatusCode::SERVICE_UNAVAILABLE, msg).into_response(),
            AppError::Db(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error").into_response(),
            AppError::Llm(llm_err) => match llm_err {
                LlmError::RateLimited => {
                    let mut headers = HeaderMap::new();
                    headers.insert("Retry-After", HeaderValue::from_static("60"));
                    (StatusCode::TOO_MANY_REQUESTS, headers, "LLM rate limit exceeded").into_response()
                },
                LlmError::Disabled => (StatusCode::SERVICE_UNAVAILABLE, "AI features are not enabled").into_response(),
                LlmError::Timeout => (StatusCode::GATEWAY_TIMEOUT, "LLM request timed out").into_response(),
                _ => (StatusCode::INTERNAL_SERVER_ERROR, format!("AI service error: {}", llm_err)).into_response(),
            },
        }
    }
}

impl From<sqlx::Error> for AppError {
    fn from(value: Error) -> Self {
        AppError::Db(value)
    }
}

impl From<String> for AppError {
    fn from(value: String) -> Self {
        AppError::Validation(value)
    }
}

impl From<LlmError> for AppError {
    fn from(value: LlmError) -> Self {
        AppError::Llm(value)
    }
}



