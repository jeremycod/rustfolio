use axum::{
    extract::State,
    http::{header, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};

use crate::auth;
use crate::db::auth_queries;
use crate::errors::AppError;
use crate::middleware::auth::AuthUser;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/logout", post(logout))
        .route("/me", get(me))
}

// ==============================================================================
// Request / Response types
// ==============================================================================

#[derive(Debug, Deserialize)]
struct RegisterRequest {
    email: String,
    password: String,
    name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct LoginRequest {
    email: String,
    password: String,
}

#[derive(Debug, Serialize)]
struct UserResponse {
    id: uuid::Uuid,
    email: String,
    name: Option<String>,
}

// ==============================================================================
// Handlers
// ==============================================================================

async fn register(
    State(state): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> Result<impl IntoResponse, AppError> {
    let email = req.email.trim().to_lowercase();

    if email.is_empty() {
        return Err(AppError::Validation("Email is required".into()));
    }
    if req.password.len() < 8 {
        return Err(AppError::Validation(
            "Password must be at least 8 characters".into(),
        ));
    }

    let existing = auth_queries::get_user_by_email(&state.pool, &email).await?;

    // Reject if a real account (with a password) already exists for this email
    if let Some(ref u) = existing {
        if u.password_hash.is_some() {
            return Err(AppError::Validation("Email already registered".into()));
        }
    }

    let password_hash = auth::hash_password(&req.password)
        .map_err(|e| AppError::External(format!("Password hashing failed: {}", e)))?;

    let user = if let Some(existing_user) = existing {
        // Claim the existing passwordless account (e.g. the default seed user)
        // All data already belongs to this account's ID — no migration needed.
        auth_queries::set_user_password(
            &state.pool,
            existing_user.id,
            &password_hash,
            req.name.as_deref(),
        )
        .await?
    } else {
        // Check BEFORE inserting so we know if this is the first real user
        let is_first_user = auth_queries::count_non_default_users(&state.pool).await? == 0;

        let new_user = auth_queries::create_user_with_password(
            &state.pool,
            &email,
            req.name.as_deref(),
            &password_hash,
        )
        .await?;

        // Migrate existing default-user data to the first real registrant
        if is_first_user {
            if let Err(e) =
                auth_queries::migrate_default_user_data(&state.pool, new_user.id).await
            {
                tracing::warn!("Data migration for first user failed: {}", e);
            }
        }

        new_user
    };

    let response = UserResponse {
        id: user.id,
        email: user.email,
        name: user.name,
    };

    Ok((StatusCode::CREATED, Json(response)))
}

async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<impl IntoResponse, AppError> {
    let email = req.email.trim().to_lowercase();

    let user = auth_queries::get_user_by_email(&state.pool, &email)
        .await?
        .ok_or(AppError::Unauthorized)?;

    let hash = user.password_hash.as_deref().ok_or(AppError::Unauthorized)?;

    let valid = auth::verify_password(hash, &req.password)
        .map_err(|e| AppError::External(format!("Password verification error: {}", e)))?;

    if !valid {
        return Err(AppError::Unauthorized);
    }

    let token = auth::create_jwt(user.id, &state.jwt_secret)
        .map_err(|e| AppError::External(format!("Token creation failed: {}", e)))?;

    let cookie = format!(
        "auth_token={}; HttpOnly; SameSite=Strict; Path=/; Max-Age=86400",
        token
    );

    let response = UserResponse {
        id: user.id,
        email: user.email,
        name: user.name,
    };

    Ok((
        StatusCode::OK,
        [(header::SET_COOKIE, cookie)],
        Json(response),
    ))
}

async fn logout() -> impl IntoResponse {
    let clear_cookie = "auth_token=; HttpOnly; SameSite=Strict; Path=/; Max-Age=0";
    (
        StatusCode::NO_CONTENT,
        [(header::SET_COOKIE, clear_cookie)],
    )
}

async fn me(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
) -> Result<impl IntoResponse, AppError> {
    let user = auth_queries::get_user(&state.pool, user_id).await?;

    let response = UserResponse {
        id: user.id,
        email: user.email,
        name: user.name,
    };

    Ok(Json(response))
}
