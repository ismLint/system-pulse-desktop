
use std::path::{Path, PathBuf};

use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::SqlitePool;
use tracing::info;

use crate::error::{DbError, DbResult};
use crate::migrations::run_migrations;

pub const DEFAULT_SERVER_DB_PATH: &str = "/data/system_pulse.db";

pub const DATABASE_PATH_ENV: &str = "DATABASE_PATH";

pub const MAX_CONNECTIONS_ENV: &str = "DATABASE_MAX_CONNECTIONS";

#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub path: PathBuf,
    pub max_connections: u32,
    pub enable_wal: bool,
}

impl DatabaseConfig {
    pub fn at_path(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            max_connections: 5,
            enable_wal: true,
        }
    }

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
