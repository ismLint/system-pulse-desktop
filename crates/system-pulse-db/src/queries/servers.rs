use sqlx::SqlitePool;
use uuid::Uuid;

use crate::error::{DbError, DbResult};
use crate::models::server::{CreateServerInput, Server, UpdateServerInput};

pub async fn list_for_user(pool: &SqlitePool, user_id: &str) -> DbResult<Vec<Server>> {
    sqlx::query_as("SELECT * FROM servers WHERE user_id = ?1 ORDER BY created_at ASC")
        .bind(user_id)
        .fetch_all(pool)
        .await
        .map_err(DbError::Sqlx)
}

pub async fn find_owned(pool: &SqlitePool, id: &str, user_id: &str) -> DbResult<Option<Server>> {
    sqlx::query_as("SELECT * FROM servers WHERE id = ?1 AND user_id = ?2")
        .bind(id)
        .bind(user_id)
        .fetch_optional(pool)
        .await
        .map_err(DbError::Sqlx)
}

/// Lookup without ownership check — used by the metrics ingestion path,
/// where the caller (agent / background poller) authenticates differently.
pub async fn find_by_id(pool: &SqlitePool, id: &str) -> DbResult<Option<Server>> {
    sqlx::query_as("SELECT * FROM servers WHERE id = ?1")
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(DbError::Sqlx)
}

pub async fn create(pool: &SqlitePool, user_id: &str, input: &CreateServerInput) -> DbResult<Server> {
    let id = Uuid::new_v4().to_string();

    sqlx::query(
        "INSERT INTO servers (id, user_id, name, host, ssh_user, password_encrypted, description, server_type)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8)",
    )
    .bind(&id)
    .bind(user_id)
    .bind(&input.name)
    .bind(&input.host)
    .bind(&input.ssh_user)
    .bind(&input.password_encrypted)
    .bind(&input.description)
    .bind(&input.server_type)
    .execute(pool)
    .await
    .map_err(|e| {
        if e.to_string().contains("UNIQUE") {
            DbError::Conflict("server name already exists for this user".into())
        } else {
            DbError::Sqlx(e)
        }
    })?;

    find_by_id(pool, &id)
        .await?
        .ok_or_else(|| DbError::NotFound("server not found after insert".into()))
}

pub async fn update(
    pool: &SqlitePool,
    id: &str,
    user_id: &str,
    input: &UpdateServerInput,
) -> DbResult<Server> {
    sqlx::query(
        "UPDATE servers SET
           name        = COALESCE(?3, name),
           host        = COALESCE(?4, host),
           ssh_user    = COALESCE(?5, ssh_user),
           description = COALESCE(?6, description),
           is_active   = COALESCE(?7, is_active),
           updated_at  = datetime('now')
         WHERE id = ?1 AND user_id = ?2",
    )
    .bind(id)
    .bind(user_id)
    .bind(&input.name)
    .bind(&input.host)
    .bind(&input.ssh_user)
    .bind(&input.description)
    .bind(input.is_active.map(|v| v as i64))
    .execute(pool)
    .await
    .map_err(DbError::Sqlx)?;

    find_owned(pool, id, user_id)
        .await?
        .ok_or_else(|| DbError::NotFound("server not found".into()))
}

pub async fn delete(pool: &SqlitePool, id: &str, user_id: &str) -> DbResult<()> {
    sqlx::query("DELETE FROM servers WHERE id = ?1 AND user_id = ?2")
        .bind(id)
        .bind(user_id)
        .execute(pool)
        .await
        .map_err(DbError::Sqlx)?;
    Ok(())
}
