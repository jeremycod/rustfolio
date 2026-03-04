use axum::{
    extract::State,
    http::{header, StatusCode},
    response::IntoResponse,
    routing::{get, post, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};

use crate::auth;
use crate::db::auth_queries;
use crate::errors::AppError;
use crate::middleware::auth::AuthUser;
use crate::services::notification_service;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/logout", post(logout))
        .route("/me", get(me))
        .route("/profile", put(update_profile))
        .route("/change-password", put(change_password))
        .route("/request-password-reset", post(request_password_reset))
        .route("/reset-password", post(reset_password))
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

#[derive(Debug, Deserialize)]
struct UpdateProfileRequest {
    email: String,
    name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ChangePasswordRequest {
    current_password: String,
    new_password: String,
}

#[derive(Debug, Deserialize)]
struct ForgotPasswordRequest {
    email: String,
}

#[derive(Debug, Deserialize)]
struct ResetPasswordRequest {
    token: String,
    new_password: String,
}

#[derive(Debug, Serialize)]
struct UserResponse {
    id: uuid::Uuid,
    email: String,
    name: Option<String>,
}

#[derive(Debug, Serialize)]
struct MessageResponse {
    message: String,
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

async fn update_profile(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Json(req): Json<UpdateProfileRequest>,
) -> Result<impl IntoResponse, AppError> {
    let email = req.email.trim().to_lowercase();

    if email.is_empty() {
        return Err(AppError::Validation("Email is required".into()));
    }

    // Check if the new email is already taken by a different user
    if let Some(existing) = auth_queries::get_user_by_email(&state.pool, &email).await? {
        if existing.id != user_id {
            return Err(AppError::Validation("Email already in use".into()));
        }
    }

    let name = req.name.as_deref().map(|n| {
        let trimmed = n.trim();
        if trimmed.is_empty() { None } else { Some(trimmed) }
    }).flatten();

    let user = auth_queries::update_user_profile(&state.pool, user_id, &email, name).await?;

    Ok(Json(UserResponse {
        id: user.id,
        email: user.email,
        name: user.name,
    }))
}

async fn change_password(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Json(req): Json<ChangePasswordRequest>,
) -> Result<impl IntoResponse, AppError> {
    if req.new_password.len() < 8 {
        return Err(AppError::Validation(
            "New password must be at least 8 characters".into(),
        ));
    }

    let user = auth_queries::get_user(&state.pool, user_id).await?;
    let hash = user.password_hash.as_deref().ok_or(AppError::Unauthorized)?;

    let valid = auth::verify_password(hash, &req.current_password)
        .map_err(|e| AppError::External(format!("Password verification error: {}", e)))?;

    if !valid {
        return Err(AppError::Validation("Current password is incorrect".into()));
    }

    let new_hash = auth::hash_password(&req.new_password)
        .map_err(|e| AppError::External(format!("Password hashing failed: {}", e)))?;

    auth_queries::update_user_password(&state.pool, user_id, &new_hash).await?;

    Ok(StatusCode::NO_CONTENT)
}

async fn request_password_reset(
    State(state): State<AppState>,
    Json(req): Json<ForgotPasswordRequest>,
) -> Result<impl IntoResponse, AppError> {
    let email = req.email.trim().to_lowercase();

    // Always return the same generic response to prevent email enumeration.
    // Any error during token generation or sending is logged but not surfaced.
    let generic = Json(MessageResponse {
        message: "If that email is registered you will receive a reset token shortly.".into(),
    });

    let user = match auth_queries::get_user_by_email(&state.pool, &email).await? {
        Some(u) if u.password_hash.is_some() => u,
        _ => return Ok(generic),
    };

    let token = uuid::Uuid::new_v4().to_string();
    auth_queries::create_password_reset_token(&state.pool, user.id, &token).await?;

    // Send token via email only — it is never included in the HTTP response.
    if let Err(e) = notification_service::send_password_reset_email(&user.email, &token).await {
        tracing::error!("Failed to send password reset email to {}: {}", email, e);
    }

    Ok(generic)
}

async fn reset_password(
    State(state): State<AppState>,
    Json(req): Json<ResetPasswordRequest>,
) -> Result<impl IntoResponse, AppError> {
    if req.new_password.len() < 8 {
        return Err(AppError::Validation(
            "Password must be at least 8 characters".into(),
        ));
    }

    let user_id = auth_queries::consume_password_reset_token(&state.pool, &req.token)
        .await?
        .ok_or_else(|| AppError::Validation("Invalid or expired reset token".into()))?;

    let new_hash = auth::hash_password(&req.new_password)
        .map_err(|e| AppError::External(format!("Password hashing failed: {}", e)))?;

    auth_queries::update_user_password(&state.pool, user_id, &new_hash).await?;

    Ok(StatusCode::NO_CONTENT)
}
