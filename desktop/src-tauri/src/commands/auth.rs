use serde::Deserialize;
use tauri::State;
use system_pulse_db::models::user::{CreateUserInput, UserPublic};
use system_pulse_db::queries::users;
use system_pulse_db::Database;

use crate::models::error::{AppError, AppResult};
use crate::services::{auth as auth_svc, crypto};

#[derive(Debug, Deserialize)]
pub struct RegisterInput {
    pub login: String,
    pub email: String,
    pub password: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LoginInput {
    pub login: String,
    pub password: String,
}

#[derive(Debug, serde::Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: UserPublic,
}

#[tauri::command]
pub async fn register(db: State<'_, Database>, input: RegisterInput) -> AppResult<AuthResponse> {
    if input.login.trim().len() < 3 {
        return Err(AppError::BadRequest("Login must be at least 3 characters".into()));
    }
    if input.password.len() < 8 {
        return Err(AppError::BadRequest("Password must be at least 8 characters".into()));
    }

    if users::login_or_email_exists(&db.pool, &input.login, &input.email).await? {
        return Err(AppError::Conflict("Login or email already taken".into()));
    }

    let password_hash = crypto::hash_password(&input.password)
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let user = users::create(
        &db.pool,
        &CreateUserInput {
            login: input.login,
            email: input.email,
            password_hash,
            first_name: input.first_name,
            last_name: input.last_name,
        },
    )
    .await?;

    let token = auth_svc::create_token(&user.id, &user.login)
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(AuthResponse { token, user: UserPublic::from(user) })
}

#[tauri::command]
pub async fn login(db: State<'_, Database>, input: LoginInput) -> AppResult<AuthResponse> {
    let user = users::find_by_login(&db.pool, &input.login)
        .await?
        .ok_or_else(|| AppError::Unauthorized("Invalid login or password".into()))?;

    let ok = crypto::verify_password(&input.password, &user.password_hash)
        .map_err(|e| AppError::Internal(e.to_string()))?;

    if !ok {
        return Err(AppError::Unauthorized("Invalid login or password".into()));
    }

    let token = auth_svc::create_token(&user.id, &user.login)
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(AuthResponse { token, user: UserPublic::from(user) })
}

#[tauri::command]
pub async fn logout() -> AppResult<()> {
    Ok(())
}

#[tauri::command]
pub async fn get_me(db: State<'_, Database>, token: String) -> AppResult<UserPublic> {
    let uid = auth_svc::extract_user_id(&token)
        .map_err(|_| AppError::Unauthorized("Invalid token".into()))?;

    let user = users::find_by_id(&db.pool, &uid)
        .await?
        .ok_or_else(|| AppError::NotFound("User not found".into()))?;

    Ok(UserPublic::from(user))
}
