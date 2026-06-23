use axum::{
    extract::{FromRequestParts,FromRef},
    http::{request::Parts, header::AUTHORIZATION},
};

use crate::{auth, error::ApiError, AppState};
pub struct AuthUser(pub String);

impl<S> FromRequestParts<S> for AuthUser
where
    AppState: axum::extract::FromRef<S>,
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &S
    ) -> Result<Self, Self::Rejection> {
        let app_state = AppState::from_ref(state);

        let header = parts
            .headers
            .get(AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| ApiError::Unauthorized("missing authorization header".into()))?;

        let token = auth::extract_bearer(header)
            .ok_or_else(|| ApiError::Unauthorized("expected Bearer token".into()))?;

        let claims = auth::verify_token(token, &app_state.config.jwt_secret)
            .map_err(|_| ApiError::Unauthorized("invalid or expired token".into()))?;

        // Reject tokens whose session has been revoked (logout).
        let token_hash = auth::hash_token(token);
        let valid = system_pulse_db::queries::sessions::is_valid(&app_state.db.pool, &token_hash)
            .await
            .map_err(|e| ApiError::Internal(e.to_string()))?;

        if !valid {
            return Err(ApiError::Unauthorized("session revoked or expired".into()));
        }

        Ok(AuthUser(claims.sub))
    }
}
