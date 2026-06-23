use sqlx::SqlitePool;
use uuid::Uuid;

use crate::error::{DbError, DbResult};
use crate::models::session::{CreateSessionInput, Session};

pub async fn create(pool: &SqlitePool, input: &CreateSessionInput) -> DbResult<Session> {
    let id = Uuid::new_v4().to_string();

    sqlx::query(
        "INSERT INTO sessions (id, user_id, token_hash, expires_at) VALUES (?1,?2,?3,?4)",
    )
    .bind(&id)
    .bind(&input.user_id)
    .bind(&input.token_hash)
    .bind(&input.expires_at)
    .execute(pool)
    .await
    .map_err(DbError::Sqlx)?;

    sqlx::query_as("SELECT * FROM sessions WHERE id = ?1")
        .bind(&id)
        .fetch_one(pool)
        .await
        .map_err(DbError::Sqlx)
}

pub async fn is_valid(pool: &SqlitePool, token_hash: &str) -> DbResult<bool> {
    let valid: bool = sqlx::query_scalar(
        "SELECT EXISTS(
            SELECT 1 FROM sessions
            WHERE token_hash = ?1
              AND revoked = 0
              AND expires_at > datetime('now')
         )",
    )
    .bind(token_hash)
    .fetch_one(pool)
    .await
    .map_err(DbError::Sqlx)?;
    Ok(valid)
}

pub async fn revoke(pool: &SqlitePool, token_hash: &str) -> DbResult<()> {
    sqlx::query("UPDATE sessions SET revoked = 1 WHERE token_hash = ?1")
        .bind(token_hash)
        .execute(pool)
        .await
        .map_err(DbError::Sqlx)?;
    Ok(())
}

pub async fn revoke_all_for_user(pool: &SqlitePool, user_id: &str) -> DbResult<()> {
    sqlx::query("UPDATE sessions SET revoked = 1 WHERE user_id = ?1")
        .bind(user_id)
        .execute(pool)
        .await
        .map_err(DbError::Sqlx)?;
    Ok(())
}
