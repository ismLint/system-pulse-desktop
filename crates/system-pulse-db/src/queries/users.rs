use sqlx::SqlitePool;
use uuid::Uuid;

use crate::error::{DbError, DbResult};
use crate::models::user::{CreateUserInput, UpdateUserInput, User};

pub async fn login_or_email_exists(pool: &SqlitePool, login: &str, email: &str) -> DbResult<bool> {
    sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users WHERE login = ?1 OR email = ?2)")
        .bind(login)
        .bind(email)
        .fetch_one(pool)
        .await
        .map_err(DbError::Sqlx)
}

pub async fn login_taken_by_other(pool: &SqlitePool, login: &str, exclude_id: &str) -> DbResult<bool> {
    sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users WHERE login = ?1 AND id != ?2)")
        .bind(login)
        .bind(exclude_id)
        .fetch_one(pool)
        .await
        .map_err(DbError::Sqlx)
}

pub async fn email_taken_by_other(pool: &SqlitePool, email: &str, exclude_id: &str) -> DbResult<bool> {
    sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users WHERE email = ?1 AND id != ?2)")
        .bind(email)
        .bind(exclude_id)
        .fetch_one(pool)
        .await
        .map_err(DbError::Sqlx)
}

pub async fn create(pool: &SqlitePool, input: &CreateUserInput) -> DbResult<User> {
    let id = Uuid::new_v4().to_string();

    sqlx::query(
        "INSERT INTO users (id, login, email, password_hash, first_name, last_name)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
    )
    .bind(&id)
    .bind(&input.login)
    .bind(&input.email)
    .bind(&input.password_hash)
    .bind(&input.first_name)
    .bind(&input.last_name)
    .execute(pool)
    .await
    .map_err(DbError::Sqlx)?;

    find_by_id(pool, &id)
        .await?
        .ok_or_else(|| DbError::NotFound("user not found after insert".into()))
}

pub async fn find_by_id(pool: &SqlitePool, id: &str) -> DbResult<Option<User>> {
    sqlx::query_as("SELECT * FROM users WHERE id = ?1")
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(DbError::Sqlx)
}

pub async fn find_by_login(pool: &SqlitePool, login: &str) -> DbResult<Option<User>> {
    sqlx::query_as("SELECT * FROM users WHERE login = ?1")
        .bind(login)
        .fetch_optional(pool)
        .await
        .map_err(DbError::Sqlx)
}

pub async fn update(pool: &SqlitePool, id: &str, input: &UpdateUserInput) -> DbResult<User> {
    sqlx::query(
        "UPDATE users SET
           login         = COALESCE(?2, login),
           email         = COALESCE(?3, email),
           password_hash = COALESCE(?4, password_hash),
           first_name    = COALESCE(?5, first_name),
           last_name     = COALESCE(?6, last_name),
           avatar_url    = COALESCE(?7, avatar_url),
           updated_at    = datetime('now')
         WHERE id = ?1",
    )
    .bind(id)
    .bind(&input.login)
    .bind(&input.email)
    .bind(&input.password_hash)
    .bind(&input.first_name)
    .bind(&input.last_name)
    .bind(&input.avatar_url)
    .execute(pool)
    .await
    .map_err(DbError::Sqlx)?;

    find_by_id(pool, id)
        .await?
        .ok_or_else(|| DbError::NotFound("user not found after update".into()))
}
