use async_trait::async_trait;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use uuid::Uuid;
use crate::auth;
use crate::errors::AppError;
use crate::state::AppState;

/// Axum extractor that validates the `auth_token` httpOnly cookie and
/// provides the authenticated user's UUID to handlers.
pub struct AuthUser(pub Uuid);

#[async_trait]
impl FromRequestParts<AppState> for AuthUser {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        let cookie_header = parts
            .headers
            .get("cookie")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");

        let token = cookie_header
            .split(';')
            .map(|s| s.trim())
            .find_map(|s| {
                let mut kv = s.splitn(2, '=');
                let name = kv.next()?.trim();
                let value = kv.next()?.trim();
                if name == "auth_token" {
                    Some(value.to_owned())
                } else {
                    None
                }
            })
            .ok_or(AppError::Unauthorized)?;

        let user_id = auth::validate_jwt(&token, &state.jwt_secret)
            .map_err(|_| AppError::Unauthorized)?;

        Ok(AuthUser(user_id))
    }
}
