use serde::Deserialize;
use tauri::State;
use system_pulse_db::models::server::{CreateServerInput, ServerPublic, UpdateServerInput};
use system_pulse_db::queries::servers;
use system_pulse_db::Database;

use crate::models::error::{AppError, AppResult};
use crate::services::{auth as auth_svc, crypto, ssh_metrics};

const ENC_KEY: &str = "sp_desktop_enc_key_v1";

#[derive(Debug, Deserialize)]
pub struct CreateServerRequest {
    pub name: String,
    pub host: String,
    pub ssh_user: String,
    pub password: String,
    pub description: Option<String>,
    pub server_type: String,
}

fn uid(token: &str) -> AppResult<String> {
    auth_svc::extract_user_id(token).map_err(|_| AppError::Unauthorized("Invalid token".into()))
}

#[tauri::command]
pub async fn list_servers(db: State<'_, Database>, token: String) -> AppResult<Vec<ServerPublic>> {
    let uid = uid(&token)?;
    let list = servers::list_for_user(&db.pool, &uid).await?;
    Ok(list.into_iter().map(ServerPublic::from).collect())
}

#[tauri::command]
pub async fn get_server(db: State<'_, Database>, token: String, id: String) -> AppResult<ServerPublic> {
    let uid = uid(&token)?;
    let s = servers::find_owned(&db.pool, &id, &uid)
        .await?
        .ok_or_else(|| AppError::NotFound("Server not found".into()))?;
    Ok(ServerPublic::from(s))
}

#[tauri::command]
pub async fn create_server(
    db: State<'_, Database>,
    token: String,
    input: CreateServerRequest,
) -> AppResult<ServerPublic> {
    let uid = uid(&token)?;

    if input.name.trim().is_empty() {
        return Err(AppError::BadRequest("Name is required".into()));
    }
    if input.server_type == "remote" {
        if input.host.trim().is_empty() {
            return Err(AppError::BadRequest("Host is required for remote servers".into()));
        }
        if input.ssh_user.trim().is_empty() {
            return Err(AppError::BadRequest("SSH user is required for remote servers".into()));
        }
    }

    let enc = if input.server_type == "remote" {
        crypto::encrypt(&input.password, ENC_KEY)
    } else {
        String::new()
    };

    let s = servers::create(
        &db.pool,
        &uid,
        &CreateServerInput {
            name: input.name.trim().to_string(),
            host: input.host.trim().to_string(),
            ssh_user: input.ssh_user.trim().to_string(),
            password_encrypted: enc,
            description: input.description,
            server_type: input.server_type,
        },
    )
    .await?;

    Ok(ServerPublic::from(s))
}

#[tauri::command]
pub async fn update_server(
    db: State<'_, Database>,
    token: String,
    id: String,
    input: UpdateServerInput,
) -> AppResult<ServerPublic> {
    let uid = uid(&token)?;
    let s = servers::update(&db.pool, &id, &uid, &input).await?;
    Ok(ServerPublic::from(s))
}

#[tauri::command]
pub async fn delete_server(db: State<'_, Database>, token: String, id: String) -> AppResult<()> {
    let uid = uid(&token)?;
    servers::delete(&db.pool, &id, &uid).await?;
    Ok(())
}

#[tauri::command]
pub async fn test_connection(db: State<'_, Database>, token: String, id: String) -> AppResult<String> {
    let uid = uid(&token)?;
    let server = servers::find_owned(&db.pool, &id, &uid)
        .await?
        .ok_or_else(|| AppError::NotFound("Server not found".into()))?;

    if server.server_type == "local" {
        return Ok("Local machine — no SSH needed".to_string());
    }

    let pass = crypto::decrypt(&server.password_encrypted, ENC_KEY)
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let host = server.host.clone();
    let user = server.ssh_user.clone();

    tokio::task::spawn_blocking(move || {
        ssh_metrics::test_connection(&host, &user, &pass).map_err(|e| AppError::Internal(e.to_string()))
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?
}
