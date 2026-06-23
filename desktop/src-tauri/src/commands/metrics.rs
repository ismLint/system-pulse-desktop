use tauri::{AppHandle, Emitter, State};
use system_pulse_db::models::metric::Metric;
use system_pulse_db::models::server::Server;
use system_pulse_db::queries::{metrics, servers};
use system_pulse_db::Database;

use crate::models::error::{AppError, AppResult};
use crate::services::{auth as auth_svc, crypto, local_metrics, ssh_metrics};

const ENC_KEY: &str = "sp_desktop_enc_key_v1";

fn uid(token: &str) -> AppResult<String> {
    auth_svc::extract_user_id(token).map_err(|_| AppError::Unauthorized("Invalid token".into()))
}

#[tauri::command]
pub async fn get_metrics(
    db: State<'_, Database>,
    token: String,
    server_id: String,
    limit: Option<i64>,
) -> AppResult<Vec<Metric>> {
    let uid = uid(&token)?;
    servers::find_owned(&db.pool, &server_id, &uid)
        .await?
        .ok_or_else(|| AppError::NotFound("Server not found".into()))?;

    let n = limit.unwrap_or(120).min(1440);
    let rows = metrics::recent_for_server(&db.pool, &server_id, n).await?;
    Ok(rows)
}

#[tauri::command]
pub async fn get_latest_metric(
    db: State<'_, Database>,
    token: String,
    server_id: String,
) -> AppResult<Option<Metric>> {
    let uid = uid(&token)?;
    servers::find_owned(&db.pool, &server_id, &uid)
        .await?
        .ok_or_else(|| AppError::NotFound("Server not found".into()))?;

    let m = metrics::latest_for_server(&db.pool, &server_id).await?;
    Ok(m)
}

#[tauri::command]
pub async fn collect_now(db: State<'_, Database>, token: String, server_id: String) -> AppResult<Metric> {
    let uid = uid(&token)?;
    let server = servers::find_owned(&db.pool, &server_id, &uid)
        .await?
        .ok_or_else(|| AppError::NotFound("Server not found".into()))?;

    let input = do_collect(&server).await?;
    let metric = metrics::insert(&db.pool, &input).await?;
    Ok(metric)
}

#[tauri::command]
pub async fn start_polling(
    app: AppHandle,
    db: State<'_, Database>,
    token: String,
    server_id: String,
    interval_secs: Option<u64>,
) -> AppResult<()> {
    let uid = uid(&token)?;
    let server = servers::find_owned(&db.pool, &server_id, &uid)
        .await?
        .ok_or_else(|| AppError::NotFound("Server not found".into()))?;

    let pool = db.pool.clone();
    let interval = std::time::Duration::from_secs(interval_secs.unwrap_or(5));

    tokio::spawn(async move {
        loop {
            match do_collect(&server).await {
                Ok(input) => match metrics::insert(&pool, &input).await {
                    Ok(metric) => {
                        let _ = app.emit(&format!("metric:{}", metric.server_id), &metric);
                    }
                    Err(e) => {
                        let _ = app.emit(
                            &format!("metric_error:{}", server.id),
                            serde_json::json!({ "error": e.to_string() }),
                        );
                    }
                },
                Err(e) => {
                    let _ = app.emit(
                        &format!("metric_error:{}", server.id),
                        serde_json::json!({ "error": e.to_string() }),
                    );
                }
            }
            tokio::time::sleep(interval).await;
        }
    });

    Ok(())
}

#[tauri::command]
pub async fn stop_polling(_server_id: String) -> AppResult<()> {
    Ok(()) // Frontend stops listening; background task exits on next error.
}

// ─── Dispatch: local vs remote ───────────────────────────────────────────────

async fn do_collect(server: &Server) -> AppResult<system_pulse_db::models::metric::MetricInput> {
    if server.server_type == "local" {
        let sid = server.id.clone();
        let input = tokio::task::spawn_blocking(move || local_metrics::collect_local(&sid))
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        Ok(input)
    } else {
        let pass = crypto::decrypt(&server.password_encrypted, ENC_KEY)
            .map_err(|e| AppError::Internal(e.to_string()))?;
        let sid = server.id.clone();
        let host = server.host.clone();
        let user = server.ssh_user.clone();

        tokio::task::spawn_blocking(move || {
            ssh_metrics::collect_remote(&sid, &host, &user, &pass)
                .map_err(|e| AppError::Internal(e.to_string()))
        })
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
    }
}
