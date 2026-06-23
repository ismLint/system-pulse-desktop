use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Metric {
    pub id: i64,
    pub server_id: String,
    pub collected_at: String,
    pub cpu_usage: f64,
    pub cpu_cores: Option<i64>,
    pub ram_used: i64,
    pub ram_total: i64,
    pub ram_usage_pct: f64,
    pub temperature_cpu: Option<f64>,
    pub temperature_gpu: Option<f64>,
    pub disk_used: Option<i64>,
    pub disk_total: Option<i64>,
    pub disk_read_bytes_sec: Option<i64>,
    pub disk_write_bytes_sec: Option<i64>,
    pub net_rx_bytes_sec: Option<i64>,
    pub net_tx_bytes_sec: Option<i64>,
    pub load_avg_1: Option<f64>,
    pub load_avg_5: Option<f64>,
    pub load_avg_15: Option<f64>,
    pub uptime_seconds: Option<i64>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MetricInput {
    pub server_id: String,
    pub collected_at: String,
    pub cpu_usage: f64,
    pub cpu_cores: Option<i64>,
    pub ram_used: i64,
    pub ram_total: i64,
    pub ram_usage_pct: f64,
    pub temperature_cpu: Option<f64>,
    pub temperature_gpu: Option<f64>,
    pub disk_used: Option<i64>,
    pub disk_total: Option<i64>,
    pub disk_read_bytes_sec: Option<i64>,
    pub disk_write_bytes_sec: Option<i64>,
    pub net_rx_bytes_sec: Option<i64>,
    pub net_tx_bytes_sec: Option<i64>,
    pub load_avg_1: Option<f64>,
    pub load_avg_5: Option<f64>,
    pub load_avg_15: Option<f64>,
    pub uptime_seconds: Option<i64>,
}
