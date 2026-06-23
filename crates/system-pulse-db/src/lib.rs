//! Shared SQLite data layer for System Pulse.
//!
//! This crate is consumed by:
//! - `system-pulse-desktop` (Tauri app) — opens a local file under the OS app-data dir
//! - `system-pulse-server` (standalone Axum server) — opens a file path from config,
//!   typically a mounted Docker volume, for multi-client deployments
//!
//! Both consumers get the same models, the same migrations, and the same
//! query functions, so behavior never drifts between the two binaries.

pub mod connection;
pub mod error;
pub mod migrations;
pub mod models;
pub mod queries;

pub use connection::{Database, DatabaseConfig};
pub use error::{DbError, DbResult};
