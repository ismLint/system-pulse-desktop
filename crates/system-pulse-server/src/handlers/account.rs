use axum::{extract::State, Json};
use serde::Deserialize;
use system_pulse_db::models::user::UpdateUserInput;
use system_pulse_db::queries::{sessions, users};

use crate::{auth, error::{ApiError, ApiResult}, extractors::AuthUser, AppState};

#[derive(Debug, Deserialize)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
    pub confirm_password: String,
}

#[derive(Debug, Deserialize)]
pub struct ChangeEmailRequest {
    pub new_email: String,
}

#[derive(Debug, Deserialize)]
pub struct ChangeLoginRequest {
    pub new_login: String,
}

pub async fn change_password(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Json(body): Json<ChangePasswordRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    if body.new_password != body.confirm_password {
        return Err(ApiError::BadRequest("passwords do not match".into()));
    }
    if body.new_password.len() < 8 {
        return Err(ApiError::BadRequest("password must be at least 8 characters".into()));
    }

    let user = users::find_by_id(&state.db.pool, &user_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("user not found".into()))?;

    if !auth::verify_password(&body.current_password, &user.password_hash)? {
        return Err(ApiError::Unauthorized("current password is incorrect".into()));
    }

    let new_hash = auth::hash_password(&body.new_password)?;
    users::update(
        &state.db.pool,
        &user_id,
        &UpdateUserInput { password_hash: Some(new_hash), ..Default::default() },
    )
    .await?;

    sessions::revoke_all_for_user(&state.db.pool, &user_id).await?;

    Ok(Json(serde_json::json!({ "status": "ok" })))
}

pub async fn change_email(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Json(body): Json<ChangeEmailRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    if users::email_taken_by_other(&state.db.pool, &body.new_email, &user_id).await? {
        return Err(ApiError::Conflict("email already in use".into()));
    }

    users::update(
        &state.db.pool,
        &user_id,
        &UpdateUserInput { email: Some(body.new_email), ..Default::default() },
    )
    .await?;

    Ok(Json(serde_json::json!({ "status": "ok" })))
}

pub async fn change_login(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Json(body): Json<ChangeLoginRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    if body.new_login.trim().len() < 3 {
        return Err(ApiError::BadRequest("login must be at least 3 characters".into()));
    }
    if users::login_taken_by_other(&state.db.pool, &body.new_login, &user_id).await? {
        return Err(ApiError::Conflict("login already taken".into()));
    }

    users::update(
        &state.db.pool,
        &user_id,
        &UpdateUserInput { login: Some(body.new_login), ..Default::default() },
    )
    .await?;

    Ok(Json(serde_json::json!({ "status": "ok" })))
}
