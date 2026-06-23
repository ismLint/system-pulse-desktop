//! Database connection setup.
//!
//! Desktop usage:
//! ```ignore
//! let config = DatabaseConfig::at_path(app_data_dir.join("system_pulse.db"));
//! let db = Database::connect(config).await?;
//! ```
//!
//! Server usage (reads `DATABASE_PATH` env var, defaults to a Docker volume path):
//! ```ignore
//! let config = DatabaseConfig::from_env();
//! let db = Database::connect(config).await?;
//! ```

use std::path::{Path, PathBuf};

use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::SqlitePool;
use tracing::info;

use crate::error::{DbError, DbResult};
use crate::migrations::run_migrations;

/// Default path used by the server when no env var is set.
/// Matches the volume mount configured in `docker-compose.yml`.
pub const DEFAULT_SERVER_DB_PATH: &str = "/data/system_pulse.db";

/// Env var the server reads to locate the SQLite file.
pub const DATABASE_PATH_ENV: &str = "DATABASE_PATH";

/// Env var to override the max pool size (both desktop and server).
pub const MAX_CONNECTIONS_ENV: &str = "DATABASE_MAX_CONNECTIONS";

#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub path: PathBuf,
    pub max_connections: u32,
    /// WAL mode dramatically improves concurrent read/write throughput —
    /// important once multiple clients hit the server binary at once.
    pub enable_wal: bool,
}

impl DatabaseConfig {
    /// Use an explicit file path. This is what the desktop app calls,
    /// passing `app.path().app_data_dir()?.join("system_pulse.db")`.
    pub fn at_path(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            max_connections: 5,
            enable_wal: true,
        }
    }

    /// Build config for the standalone server from environment variables.
    /// Falls back to `/data/system_pulse.db`, which is the path mounted
    /// as a Docker volume in `docker/docker-compose.yml`.
    pub fn from_env() -> Self {
        let path = std::env::var(DATABASE_PATH_ENV)
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from(DEFAULT_SERVER_DB_PATH));

        let max_connections = std::env::var(MAX_CONNECTIONS_ENV)
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(10); // server expects more concurrent clients than desktop

        Self {
            path,
            max_connections,
            enable_wal: true,
        }
    }

    pub fn with_max_connections(mut self, n: u32) -> Self {
        self.max_connections = n;
        self
    }
}

pub struct Database {
    pub pool: SqlitePool,
    pub db_path: PathBuf,
}

impl Database {
    /// Open (creating if needed) the SQLite file and run migrations.
    pub async fn connect(config: DatabaseConfig) -> DbResult<Self> {
        ensure_parent_dir(&config.path)?;

        info!(path = %config.path.display(), "opening sqlite database");

        let mut options = SqliteConnectOptions::new()
            .filename(&config.path)
            .create_if_missing(true);

        if config.enable_wal {
            options = options.journal_mode(sqlx::sqlite::SqliteJournalMode::Wal);
        }

        let pool = SqlitePoolOptions::new()
            .max_connections(config.max_connections)
            .connect_with(options)
            .await
            .map_err(DbError::Sqlx)?;

        run_migrations(&pool).await?;

        Ok(Self {
            pool,
            db_path: config.path,
        })
    }

    /// Convenience constructor for tests — uses an in-memory database.
    /// Each call gets its own isolated database (sqlite `:memory:` semantics
    /// are per-connection, so we force a single connection in the pool).
    #[cfg(any(test, feature = "test-util"))]
    pub async fn connect_in_memory() -> DbResult<Self> {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .map_err(DbError::Sqlx)?;

        run_migrations(&pool).await?;

        Ok(Self {
            pool,
            db_path: PathBuf::from(":memory:"),
        })
    }
}

fn ensure_parent_dir(path: &Path) -> DbResult<()> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent).map_err(DbError::Io)?;
        }
    }
    Ok(())
}
