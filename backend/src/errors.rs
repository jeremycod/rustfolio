use axum::response::IntoResponse;
use reqwest::StatusCode;
use sqlx::Error;

pub enum AppError {
    Db(sqlx::Error),
    Validation(String),
    NotFound
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        StatusCode::INTERNAL_SERVER_ERROR.into_response()
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



