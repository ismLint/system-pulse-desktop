use sqlx::SqlitePool;

use crate::error::{DbError, DbResult};
use crate::models::metric::{Metric, MetricInput};

pub async fn insert(pool: &SqlitePool, m: &MetricInput) -> DbResult<Metric> {
    sqlx::query_as(
        "INSERT INTO metrics (
            server_id, collected_at, cpu_usage, cpu_cores,
            ram_used, ram_total, ram_usage_pct,
            temperature_cpu, temperature_gpu,
            disk_used, disk_total, disk_read_bytes_sec, disk_write_bytes_sec,
            net_rx_bytes_sec, net_tx_bytes_sec,
            load_avg_1, load_avg_5, load_avg_15, uptime_seconds
        ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19)
        RETURNING *",
    )
    .bind(&m.server_id)
    .bind(&m.collected_at)
    .bind(m.cpu_usage)
    .bind(m.cpu_cores)
    .bind(m.ram_used)
    .bind(m.ram_total)
    .bind(m.ram_usage_pct)
    .bind(m.temperature_cpu)
    .bind(m.temperature_gpu)
    .bind(m.disk_used)
    .bind(m.disk_total)
    .bind(m.disk_read_bytes_sec)
    .bind(m.disk_write_bytes_sec)
    .bind(m.net_rx_bytes_sec)
    .bind(m.net_tx_bytes_sec)
    .bind(m.load_avg_1)
    .bind(m.load_avg_5)
    .bind(m.load_avg_15)
    .bind(m.uptime_seconds)
    .fetch_one(pool)
    .await
    .map_err(DbError::Sqlx)
}

/// Returns metrics oldest-first (caller already gets them in chart order).
pub async fn recent_for_server(pool: &SqlitePool, server_id: &str, limit: i64) -> DbResult<Vec<Metric>> {
    let mut rows: Vec<Metric> = sqlx::query_as(
        "SELECT * FROM metrics WHERE server_id = ?1 ORDER BY collected_at DESC LIMIT ?2",
    )
    .bind(server_id)
    .bind(limit)
    .fetch_all(pool)
    .await
    .map_err(DbError::Sqlx)?;
    rows.reverse();
    Ok(rows)
}

pub async fn latest_for_server(pool: &SqlitePool, server_id: &str) -> DbResult<Option<Metric>> {
    sqlx::query_as("SELECT * FROM metrics WHERE server_id = ?1 ORDER BY collected_at DESC LIMIT 1")
        .bind(server_id)
        .fetch_optional(pool)
        .await
        .map_err(DbError::Sqlx)
}

/// Manual cleanup hook — the schema also has an `AFTER INSERT` trigger that
/// does this automatically, but the server exposes this as an explicit
/// admin/cron-callable function too (e.g. for servers with low metric volume
/// where the trigger rarely fires).
pub async fn delete_older_than_days(pool: &SqlitePool, days: i64) -> DbResult<u64> {
    let result = sqlx::query("DELETE FROM metrics WHERE collected_at < datetime('now', ?1)")
        .bind(format!("-{days} days"))
        .execute(pool)
        .await
        .map_err(DbError::Sqlx)?;
    Ok(result.rows_affected())
}
