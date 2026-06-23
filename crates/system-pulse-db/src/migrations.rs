//! Schema migrations.
//!
//! These run automatically on every `Database::connect()` call, so both the
//! desktop app and the server self-migrate on startup — no separate migration
//! step is required when deploying a new server version, as long as all
//! migrations here remain additive (`CREATE TABLE IF NOT EXISTS`, `ALTER TABLE
//! ADD COLUMN` guarded with `let _ =`) so older rows are never destroyed.

use sqlx::SqlitePool;
use tracing::info;

use crate::error::{DbError, DbResult};

pub async fn run_migrations(pool: &SqlitePool) -> DbResult<()> {
    info!("running sqlite migrations");

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS users (
            id            TEXT PRIMARY KEY NOT NULL,
            login         TEXT UNIQUE NOT NULL,
            email         TEXT UNIQUE NOT NULL,
            password_hash TEXT NOT NULL,
            first_name    TEXT,
            last_name     TEXT,
            avatar_url    TEXT,
            subscription  TEXT NOT NULL DEFAULT 'free',
            created_at    TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at    TEXT NOT NULL DEFAULT (datetime('now'))
        )",
    )
    .execute(pool)
    .await
    .map_err(DbError::Sqlx)?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS servers (
            id                 TEXT PRIMARY KEY NOT NULL,
            user_id            TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
            name               TEXT NOT NULL,
            host               TEXT NOT NULL DEFAULT '',
            ssh_user           TEXT NOT NULL DEFAULT '',
            password_encrypted TEXT NOT NULL DEFAULT '',
            description        TEXT,
            is_active          INTEGER NOT NULL DEFAULT 1,
            server_type        TEXT NOT NULL DEFAULT 'remote',
            created_at         TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at         TEXT NOT NULL DEFAULT (datetime('now')),
            UNIQUE(user_id, name)
        )",
    )
    .execute(pool)
    .await
    .map_err(DbError::Sqlx)?;

    // Upgrade path for databases created before `server_type` existed.
    let _ = sqlx::query("ALTER TABLE servers ADD COLUMN server_type TEXT NOT NULL DEFAULT 'remote'")
        .execute(pool)
        .await;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS metrics (
            id                   INTEGER PRIMARY KEY AUTOINCREMENT,
            server_id            TEXT    NOT NULL REFERENCES servers(id) ON DELETE CASCADE,
            collected_at         TEXT    NOT NULL DEFAULT (datetime('now')),
            cpu_usage            REAL    NOT NULL,
            cpu_cores            INTEGER,
            ram_used             INTEGER NOT NULL,
            ram_total            INTEGER NOT NULL,
            ram_usage_pct        REAL    NOT NULL,
            temperature_cpu      REAL,
            temperature_gpu      REAL,
            disk_used            INTEGER,
            disk_total           INTEGER,
            disk_read_bytes_sec  INTEGER,
            disk_write_bytes_sec INTEGER,
            net_rx_bytes_sec     INTEGER,
            net_tx_bytes_sec     INTEGER,
            load_avg_1           REAL,
            load_avg_5           REAL,
            load_avg_15          REAL,
            uptime_seconds       INTEGER
        )",
    )
    .execute(pool)
    .await
    .map_err(DbError::Sqlx)?;

    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_metrics_server_collected
         ON metrics(server_id, collected_at DESC)",
    )
    .execute(pool)
    .await
    .map_err(DbError::Sqlx)?;

    // Auto-cleanup: keep only the last 7 days of metrics. On the server this
    // matters a lot more than on desktop, since multiple servers' worth of
    // metrics accumulate in the same file.
    sqlx::query(
        "CREATE TRIGGER IF NOT EXISTS cleanup_old_metrics
         AFTER INSERT ON metrics
         BEGIN
           DELETE FROM metrics
           WHERE collected_at < datetime('now', '-7 days');
         END",
    )
    .execute(pool)
    .await
    .map_err(DbError::Sqlx)?;

    // Refresh/session tokens — used by the server to support logout /
    // multi-device revocation. The desktop app keeps tokens client-side only,
    // but having the table always present keeps the schema identical between
    // the two binaries.
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS sessions (
            id          TEXT PRIMARY KEY NOT NULL,
            user_id     TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
            token_hash  TEXT NOT NULL UNIQUE,
            created_at  TEXT NOT NULL DEFAULT (datetime('now')),
            expires_at  TEXT NOT NULL,
            revoked     INTEGER NOT NULL DEFAULT 0
        )",
    )
    .execute(pool)
    .await
    .map_err(DbError::Sqlx)?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_sessions_user ON sessions(user_id)")
        .execute(pool)
        .await
        .map_err(DbError::Sqlx)?;

    info!("migrations complete");
    Ok(())
}
