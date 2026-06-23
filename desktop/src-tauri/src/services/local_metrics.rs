//! Collect metrics from the local machine using `sysinfo`.
//! Cross-platform: works on Linux, Windows, and macOS — no /proc parsing needed.

use std::sync::Mutex;
use once_cell::sync::Lazy;
use sysinfo::{Components, Disks, Networks, System};

use system_pulse_db::models::metric::MetricInput;

/// Shared, lazily-initialized sysinfo state so successive calls compute
/// accurate deltas (CPU %, disk I/O/s, network I/O/s) instead of resetting
/// every call.
static STATE: Lazy<Mutex<CollectorState>> = Lazy::new(|| Mutex::new(CollectorState::new()));

struct CollectorState {
    sys: System,
    disks: Disks,
    networks: Networks,
    components: Components,
    prev_rx: u64,
    prev_tx: u64,
    prev_read: u64,
    prev_write: u64,
    prev_time: std::time::Instant,
}

impl CollectorState {
    fn new() -> Self {
        Self {
            sys: System::new_all(),
            disks: Disks::new_with_refreshed_list(),
            networks: Networks::new_with_refreshed_list(),
            components: Components::new_with_refreshed_list(),
            prev_rx: 0,
            prev_tx: 0,
            prev_read: 0,
            prev_write: 0,
            prev_time: std::time::Instant::now(),
        }
    }
}

/// Collect a single metrics snapshot for the local machine.
/// Safe to call repeatedly (e.g. every 5s from a polling loop) — deltas
/// (CPU usage, disk/net throughput) are computed against the previous call.
pub fn collect_local(server_id: &str) -> MetricInput {
    let mut state = STATE.lock().unwrap_or_else(|e| e.into_inner());

    let now = std::time::Instant::now();
    let elapsed = now.duration_since(state.prev_time).as_secs_f64().max(0.001);
    state.prev_time = now;

    state.sys.refresh_cpu_all();
    state.sys.refresh_memory();
    state.sys.refresh_processes(sysinfo::ProcessesToUpdate::All);
    state.disks.refresh();
    state.networks.refresh();
    state.components.refresh();

    // ── CPU ──────────────────────────────────────────────────────────────────
    let cpu_usage = state.sys.global_cpu_usage() as f64;
    let cpu_cores = Some(state.sys.cpus().len() as i64);

    // ── RAM ──────────────────────────────────────────────────────────────────
    let ram_total = state.sys.total_memory() as i64;
    let ram_used = state.sys.used_memory() as i64;
    let ram_usage_pct = if ram_total > 0 {
        (ram_used as f64 / ram_total as f64) * 100.0
    } else {
        0.0
    };

    // ── Temperature ──────────────────────────────────────────────────────────
    let temperature_cpu = state
        .components
        .iter()
        .find(|c| {
            let label = c.label().to_lowercase();
            label.contains("cpu") || label.contains("core") || label.contains("package")
        })
        .and_then(|c| Some(c.temperature()))
        .map(|t| t as f64);

    let temperature_gpu = state
        .components
        .iter()
        .find(|c| c.label().to_lowercase().contains("gpu"))
        .and_then(|c| Some(c.temperature()))
        .map(|t| t as f64);

    // ── Disk usage + I/O ─────────────────────────────────────────────────────
    let mut disk_total = 0i64;
    let mut disk_used = 0i64;
    for disk in state.disks.list() {
        disk_total += disk.total_space() as i64;
        disk_used += (disk.total_space() - disk.available_space()) as i64;
    }

    let mut read_bytes = 0u64;
    let mut write_bytes = 0u64;
    for (_pid, process) in state.sys.processes() {
        let io = process.disk_usage();
        read_bytes += io.read_bytes;
        write_bytes += io.written_bytes;
    }
    let read_delta = read_bytes.saturating_sub(state.prev_read);
    let write_delta = write_bytes.saturating_sub(state.prev_write);
    state.prev_read = read_bytes;
    state.prev_write = write_bytes;

    let disk_read_bytes_sec = Some((read_delta as f64 / elapsed) as i64);
    let disk_write_bytes_sec = Some((write_delta as f64 / elapsed) as i64);

    // ── Network ──────────────────────────────────────────────────────────────
    let mut rx = 0u64;
    let mut tx = 0u64;
    for (_name, data) in state.networks.iter() {
        rx += data.total_received();
        tx += data.total_transmitted();
    }
    let rx_delta = rx.saturating_sub(state.prev_rx);
    let tx_delta = tx.saturating_sub(state.prev_tx);
    state.prev_rx = rx;
    state.prev_tx = tx;

    let net_rx_bytes_sec = Some((rx_delta as f64 / elapsed) as i64);
    let net_tx_bytes_sec = Some((tx_delta as f64 / elapsed) as i64);

    // ── Load average (Unix only — None on Windows) ───────────────────────────
    #[cfg(unix)]
    let (load_avg_1, load_avg_5, load_avg_15) = {
        let load = System::load_average();
        (Some(load.one), Some(load.five), Some(load.fifteen))
    };
    #[cfg(not(unix))]
    let (load_avg_1, load_avg_5, load_avg_15): (Option<f64>, Option<f64>, Option<f64>) =
        (None, None, None);

    // ── Uptime ───────────────────────────────────────────────────────────────
    let uptime_seconds = Some(System::uptime() as i64);

    MetricInput {
        server_id: server_id.to_string(),
        collected_at: chrono::Utc::now().to_rfc3339(),
        cpu_usage,
        cpu_cores,
        ram_used,
        ram_total,
        ram_usage_pct,
        temperature_cpu,
        temperature_gpu,
        disk_used: Some(disk_used),
        disk_total: Some(disk_total),
        disk_read_bytes_sec,
        disk_write_bytes_sec,
        net_rx_bytes_sec,
        net_tx_bytes_sec,
        load_avg_1,
        load_avg_5,
        load_avg_15,
        uptime_seconds,
    }
}
