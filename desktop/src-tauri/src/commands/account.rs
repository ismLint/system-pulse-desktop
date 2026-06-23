use serde::Deserialize;
use tauri::State;
use system_pulse_db::models::user::UpdateUserInput;
use system_pulse_db::queries::users;
use system_pulse_db::Database;

use crate::models::error::{AppError, AppResult};
use crate::services::{auth as auth_svc, crypto};

#[derive(Debug, Deserialize)]
pub struct ChangePasswordInput {
    pub current_password: String,
    pub new_password: String,
    pub confirm_password: String,
}

#[derive(Debug, Deserialize)]
pub struct ChangeEmailInput {
    pub new_email: String,
}

#[derive(Debug, Deserialize)]
pub struct ChangeLoginInput {
    pub new_login: String,
}

#[tauri::command]
pub async fn change_password(
    db: State<'_, Database>,
    token: String,
    input: ChangePasswordInput,
) -> AppResult<()> {
    let uid = auth_svc::extract_user_id(&token)
        .map_err(|_| AppError::Unauthorized("Invalid token".into()))?;

    if input.new_password != input.confirm_password {
        return Err(AppError::BadRequest("Passwords do not match".into()));
    }
    if input.new_password.len() < 8 {
        return Err(AppError::BadRequest("Password must be at least 8 characters".into()));
    }

    let user = users::find_by_id(&db.pool, &uid)
        .await?
        .ok_or_else(|| AppError::NotFound("User not found".into()))?;

    let ok = crypto::verify_password(&input.current_password, &user.password_hash)
        .map_err(|e| AppError::Internal(e.to_string()))?;
    if !ok {
        return Err(AppError::Unauthorized("Current password is incorrect".into()));
    }

    let new_hash = crypto::hash_password(&input.new_password)
        .map_err(|e| AppError::Internal(e.to_string()))?;

    users::update(
        &db.pool,
        &uid,
        &UpdateUserInput { password_hash: Some(new_hash), ..Default::default() },
    )
    .await?;

    Ok(())
}

#[tauri::command]
pub async fn change_email(
    db: State<'_, Database>,
    token: String,
    input: ChangeEmailInput,
) -> AppResult<()> {
    let uid = auth_svc::extract_user_id(&token)
        .map_err(|_| AppError::Unauthorized("Invalid token".into()))?;

    if users::email_taken_by_other(&db.pool, &input.new_email, &uid).await? {
        return Err(AppError::Conflict("Email already in use".into()));
    }

    users::update(
        &db.pool,
        &uid,
        &UpdateUserInput { email: Some(input.new_email), ..Default::default() },
    )
    .await?;

    Ok(())
}

#[tauri::command]
pub async fn change_login(
    db: State<'_, Database>,
    token: String,
    input: ChangeLoginInput,
) -> AppResult<()> {
    let uid = auth_svc::extract_user_id(&token)
        .map_err(|_| AppError::Unauthorized("Invalid token".into()))?;

    if input.new_login.trim().len() < 3 {
        return Err(AppError::BadRequest("Login must be at least 3 characters".into()));
    }
    if users::login_taken_by_other(&db.pool, &input.new_login, &uid).await? {
        return Err(AppError::Conflict("Login already taken".into()));
    }

    users::update(
        &db.pool,
        &uid,
        &UpdateUserInput { login: Some(input.new_login), ..Default::default() },
    )
    .await?;

    Ok(())
}
