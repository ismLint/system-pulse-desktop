use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use system_pulse_db::models::user::{CreateUserInput, UserPublic};
use system_pulse_db::queries::{sessions, users};

use crate::{auth, error::{ApiError, ApiResult}, extractors::AuthUser, AppState};

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub login: String,
    pub email: String,
    pub password: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub login: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: UserPublic,
}

pub async fn register(
    State(state): State<AppState>,
    Json(body): Json<RegisterRequest>,
) -> ApiResult<Json<AuthResponse>> {
    if body.login.trim().len() < 3 {
        return Err(ApiError::BadRequest("login must be at least 3 characters".into()));
    }
    if body.password.len() < 8 {
        return Err(ApiError::BadRequest("password must be at least 8 characters".into()));
    }

    if users::login_or_email_exists(&state.db.pool, &body.login, &body.email).await? {
        return Err(ApiError::Conflict("login or email already taken".into()));
    }

    let password_hash = auth::hash_password(&body.password)?;

    let user = users::create(
        &state.db.pool,
        &CreateUserInput {
            login: body.login,
            email: body.email,
            password_hash,
            first_name: body.first_name,
            last_name: body.last_name,
        },
    )
    .await?;

    let (token, expires_at) =
        auth::create_token(&user.id, &user.login, &state.config.jwt_secret, state.config.jwt_expiration_hours)?;

    sessions::create(
        &state.db.pool,
        &system_pulse_db::models::session::CreateSessionInput {
            user_id: user.id.clone(),
            token_hash: auth::hash_token(&token),
            expires_at,
        },
    )
    .await?;

    Ok(Json(AuthResponse { token, user: UserPublic::from(user) }))
}

pub async fn login(
    State(state): State<AppState>,
    Json(body): Json<LoginRequest>,
) -> ApiResult<Json<AuthResponse>> {
    let user = users::find_by_login(&state.db.pool, &body.login)
        .await?
        .ok_or_else(|| ApiError::Unauthorized("invalid login or password".into()))?;

    let ok = auth::verify_password(&body.password, &user.password_hash)?;
    if !ok {
        return Err(ApiError::Unauthorized("invalid login or password".into()));
    }

    let (token, expires_at) =
        auth::create_token(&user.id, &user.login, &state.config.jwt_secret, state.config.jwt_expiration_hours)?;

    sessions::create(
        &state.db.pool,
        &system_pulse_db::models::session::CreateSessionInput {
            user_id: user.id.clone(),
            token_hash: auth::hash_token(&token),
            expires_at,
        },
    )
    .await?;

    Ok(Json(AuthResponse { token, user: UserPublic::from(user) }))
}

pub async fn logout(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
) -> ApiResult<Json<serde_json::Value>> {
    if let Some(h) = headers.get(axum::http::header::AUTHORIZATION).and_then(|v| v.to_str().ok()) {
        if let Some(token) = auth::extract_bearer(h) {
            sessions::revoke(&state.db.pool, &auth::hash_token(token)).await?;
        }
    }
    Ok(Json(serde_json::json!({ "status": "ok" })))
}

pub async fn me(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
) -> ApiResult<Json<UserPublic>> {
    let user = users::find_by_id(&state.db.pool, &user_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("user not found".into()))?;
    Ok(Json(UserPublic::from(user)))
}
