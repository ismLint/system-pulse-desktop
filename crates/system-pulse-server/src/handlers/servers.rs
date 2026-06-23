use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use system_pulse_db::models::server::{CreateServerInput, ServerPublic, UpdateServerInput};
use system_pulse_db::queries::servers;

use crate::{error::ApiResult, extractors::AuthUser, AppState};

fn enc_key() -> String {
    std::env::var("SERVER_ENC_KEY").unwrap_or_else(|_| "sp_server_enc_key_dev_only".into())
}

#[derive(Debug, Deserialize)]
pub struct CreateServerRequest {
    pub name: String,
    pub host: String,
    pub ssh_user: String,
    pub password: String,
    pub description: Option<String>,
    pub server_type: String,
}

pub async fn list(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
) -> ApiResult<Json<Vec<ServerPublic>>> {
    let list = servers::list_for_user(&state.db.pool, &user_id).await?;
    Ok(Json(list.into_iter().map(ServerPublic::from).collect()))
}

pub async fn get(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Path(id): Path<String>,
) -> ApiResult<Json<ServerPublic>> {
    let s = servers::find_owned(&state.db.pool, &id, &user_id)
        .await?
        .ok_or_else(|| crate::error::ApiError::NotFound("server not found".into()))?;
    Ok(Json(ServerPublic::from(s)))
}

pub async fn create(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Json(body): Json<CreateServerRequest>,
) -> ApiResult<(StatusCode, Json<ServerPublic>)> {
    let enc = crate::crypto::encrypt(&body.password, &enc_key());

    let s = servers::create(
        &state.db.pool,
        &user_id,
        &CreateServerInput {
            name: body.name,
            host: body.host,
            ssh_user: body.ssh_user,
            password_encrypted: enc,
            description: body.description,
            server_type: body.server_type,
        },
    )
    .await?;

    Ok((StatusCode::CREATED, Json(ServerPublic::from(s))))
}

pub async fn update(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Path(id): Path<String>,
    Json(body): Json<UpdateServerInput>,
) -> ApiResult<Json<ServerPublic>> {
    let s = servers::update(&state.db.pool, &id, &user_id, &body).await?;
    Ok(Json(ServerPublic::from(s)))
}

pub async fn delete(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Path(id): Path<String>,
) -> ApiResult<StatusCode> {
    servers::delete(&state.db.pool, &id, &user_id).await?;
    Ok(StatusCode::NO_CONTENT)
}
