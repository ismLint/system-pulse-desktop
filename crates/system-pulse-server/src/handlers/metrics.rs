use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::Deserialize;
use system_pulse_db::models::metric::Metric;
use system_pulse_db::queries::{metrics, servers};

use crate::{error::{ApiError, ApiResult}, extractors::AuthUser, ssh_metrics, AppState};

#[derive(Debug, Deserialize)]
pub struct MetricsQuery {
    pub limit: Option<i64>,
}

pub async fn get_recent(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Path(server_id): Path<String>,
    Query(q): Query<MetricsQuery>,
) -> ApiResult<Json<Vec<Metric>>> {
    servers::find_owned(&state.db.pool, &server_id, &user_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("server not found".into()))?;

    let limit = q.limit.unwrap_or(120).min(1440);
    let rows = metrics::recent_for_server(&state.db.pool, &server_id, limit).await?;
    Ok(Json(rows))
}

pub async fn get_latest(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Path(server_id): Path<String>,
) -> ApiResult<Json<Option<Metric>>> {
    servers::find_owned(&state.db.pool, &server_id, &user_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("server not found".into()))?;

    let m = metrics::latest_for_server(&state.db.pool, &server_id).await?;
    Ok(Json(m))
}

/// Trigger an immediate collection for a remote server (SSH) and persist it.
/// Local-machine collection only makes sense on the desktop app, so the
/// server only ever does the "remote" branch.
pub async fn collect_now(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Path(server_id): Path<String>,
) -> ApiResult<Json<Metric>> {
    let server = servers::find_owned(&state.db.pool, &server_id, &user_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("server not found".into()))?;

    if server.server_type != "remote" {
        return Err(ApiError::BadRequest(
            "only 'remote' servers can be collected by the standalone server".into(),
        ));
    }

    let key = std::env::var("SERVER_ENC_KEY").unwrap_or_else(|_| "sp_server_enc_key_dev_only".into());
    let password = crate::crypto::decrypt(&server.password_encrypted, &key)
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    let host = server.host.clone();
    let user = server.ssh_user.clone();
    let sid = server.id.clone();

    let input = tokio::task::spawn_blocking(move || ssh_metrics::collect_remote(&sid, &host, &user, &password))
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    let metric = metrics::insert(&state.db.pool, &input).await?;
    Ok(Json(metric))
}
