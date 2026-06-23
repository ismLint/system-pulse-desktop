use axum::{extract::State, Json};
use serde::Serialize;

use crate::AppState;

#[derive(Serialize)]
pub struct Health {
    status: &'static str,
    db: &'static str,
}

pub async fn health(State(state): State<AppState>) -> Json<Health> {
    let db_ok = sqlx::query("SELECT 1").execute(&state.db.pool).await.is_ok();
    Json(Health {
        status: "ok",
        db: if db_ok { "ok" } else { "error" },
    })
}
