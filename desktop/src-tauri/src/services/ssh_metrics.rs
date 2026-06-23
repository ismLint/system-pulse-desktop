//! Collect metrics from a remote Linux server via SSH (sshpass + ssh).
//! One SSH session per collection cycle — runs a single multi-line shell script.

use anyhow::{Context, Result};
use std::process::{Command, Stdio};

use system_pulse_db::models::metric::MetricInput;

// ─────────────────────────────────────────────────────────────────────────────
// Shared shell script — works on any standard Linux
// ─────────────────────────────────────────────────────────────────────────────
const COLLECT_SCRIPT: &str = r#"
#!/bin/sh
# CPU usage
CPU=$(top -bn1 | grep -E '^(%Cpu|Cpu)' | awk '{print 100 - $8}' 2>/dev/null)
[ -z "$CPU" ] && CPU=$(top -bn2 | grep -E '^(%Cpu|Cpu)' | tail -1 | awk '{print 100-$8}')
echo "CPU_USAGE:${CPU:-0}"
echo "CPU_CORES:$(nproc 2>/dev/null || echo 1)"

# RAM
RAM_LINE=$(free -b 2>/dev/null | awk '/^Mem:/{print $2,$3}')
RAM_TOTAL=$(echo $RAM_LINE | awk '{print $1}')
RAM_USED=$(echo  $RAM_LINE | awk '{print $2}')
RAM_PCT=$(awk "BEGIN{printf \"%.2f\", ${RAM_USED:-0}/${RAM_TOTAL:-1}*100}")
echo "RAM_TOTAL:${RAM_TOTAL:-0}"
echo "RAM_USED:${RAM_USED:-0}"
echo "RAM_PCT:${RAM_PCT:-0}"

# Disk usage (root)
DISK_LINE=$(df -B1 / 2>/dev/null | awk 'NR==2{print $2,$3}')
echo "DISK_TOTAL:$(echo $DISK_LINE | awk '{print $1}')"
echo "DISK_USED:$(echo  $DISK_LINE | awk '{print $2}')"

# Uptime
echo "UPTIME:$(awk '{print int($1)}' /proc/uptime 2>/dev/null || echo 0)"

# Load average
LOAD=$(cat /proc/loadavg 2>/dev/null)
echo "LOAD_1:$(echo $LOAD  | awk '{print $1}')"
echo "LOAD_5:$(echo $LOAD  | awk '{print $2}')"
echo "LOAD_15:$(echo $LOAD | awk '{print $3}')"

# CPU temperature (thermal_zone0 or coretemp hwmon)
TEMP=""
if [ -f /sys/class/thermal/thermal_zone0/temp ]; then
  TEMP=$(awk '{printf "%.1f",$1/1000}' /sys/class/thermal/thermal_zone0/temp 2>/dev/null)
fi
if [ -z "$TEMP" ]; then
  TEMP=$(cat /sys/class/hwmon/hwmon*/temp1_input 2>/dev/null | head -1 | awk '{printf "%.1f",$1/1000}')
fi
echo "TEMP_CPU:${TEMP}"

# Disk IO (two snapshots with 1s sleep)
DISK_DEV=$(awk '$3~/^(sd[a-z]|vd[a-z]|nvme[0-9]n[0-9])$/{print $3;exit}' /proc/diskstats 2>/dev/null)
if [ -n "$DISK_DEV" ]; then
  R1=$(awk -v d="$DISK_DEV" '$3==d{print $6}' /proc/diskstats)
  W1=$(awk -v d="$DISK_DEV" '$3==d{print $10}' /proc/diskstats)
  sleep 1
  R2=$(awk -v d="$DISK_DEV" '$3==d{print $6}' /proc/diskstats)
  W2=$(awk -v d="$DISK_DEV" '$3==d{print $10}' /proc/diskstats)
  echo "DISK_R:$(( (${R2:-0}-${R1:-0})*512 ))"
  echo "DISK_W:$(( (${W2:-0}-${W1:-0})*512 ))"
else
  sleep 1
  echo "DISK_R:0"
  echo "DISK_W:0"
fi

# Network IO
NET_IF=$(ip route get 1.1.1.1 2>/dev/null | awk '/dev/{for(i=1;i<=NF;i++)if($i=="dev")print $(i+1)}')
[ -z "$NET_IF" ] && NET_IF=$(ls /sys/class/net | grep -v lo | head -1)
if [ -n "$NET_IF" ]; then
  RX1=$(cat /sys/class/net/$NET_IF/statistics/rx_bytes 2>/dev/null || echo 0)
  TX1=$(cat /sys/class/net/$NET_IF/statistics/tx_bytes 2>/dev/null || echo 0)
  sleep 1
  RX2=$(cat /sys/class/net/$NET_IF/statistics/rx_bytes 2>/dev/null || echo 0)
  TX2=$(cat /sys/class/net/$NET_IF/statistics/tx_bytes 2>/dev/null || echo 0)
  echo "NET_RX:$(( ${RX2:-0}-${RX1:-0} ))"
  echo "NET_TX:$(( ${TX2:-0}-${TX1:-0} ))"
else
  echo "NET_RX:0"
  echo "NET_TX:0"
fi
"#;

// ─────────────────────────────────────────────────────────────────────────────
// SSH execution
// ─────────────────────────────────────────────────────────────────────────────

fn run_ssh(host: &str, user: &str, password: &str, script: &str) -> Result<String> {
    // Try sshpass first (password auth)
    let result = Command::new("sshpass")
        .args([
            "-p", password,
            "ssh",
            "-o", "StrictHostKeyChecking=no",
            "-o", "ConnectTimeout=10",
            "-o", "ServerAliveInterval=5",
            "-o", "BatchMode=no",
            &format!("{}@{}", user, host),
            script,
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output();

    match result {
        Ok(out) if out.status.success() => {
            Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
        }
        Ok(out) => {
            let stderr = String::from_utf8_lossy(&out.stderr);
            anyhow::bail!("SSH error: {}", stderr.trim())
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            // sshpass not found — try ssh with key auth
            let out = Command::new("ssh")
                .args([
                    "-o", "StrictHostKeyChecking=no",
                    "-o", "ConnectTimeout=10",
                    "-o", "PasswordAuthentication=no",
                    &format!("{}@{}", user, host),
                    script,
                ])
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output()
                .context("ssh not found — install OpenSSH")?;

            if out.status.success() {
                Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
            } else {
                anyhow::bail!(
                    "sshpass not found and key auth failed. Install sshpass: choco install sshpass\n{}",
                    String::from_utf8_lossy(&out.stderr).trim()
                )
            }
        }
        Err(e) => Err(anyhow::Error::from(e)),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Parse output
// ─────────────────────────────────────────────────────────────────────────────

fn parse_f64(s: &str) -> Option<f64> { s.trim().parse().ok() }
fn parse_i64(s: &str) -> Option<i64> { s.trim().parse().ok() }

fn parse_output(raw: &str, server_id: &str) -> MetricInput {
    let mut cpu_usage = 0.0f64;
    let mut cpu_cores: Option<i64> = None;
    let mut ram_total = 0i64;
    let mut ram_used  = 0i64;
    let mut ram_pct   = 0.0f64;
    let mut disk_total: Option<i64> = None;
    let mut disk_used:  Option<i64> = None;
    let mut disk_r: Option<i64> = None;
    let mut disk_w: Option<i64> = None;
    let mut net_rx: Option<i64> = None;
    let mut net_tx: Option<i64> = None;
    let mut uptime: Option<i64> = None;
    let mut load_1:  Option<f64> = None;
    let mut load_5:  Option<f64> = None;
    let mut load_15: Option<f64> = None;
    let mut temp_cpu: Option<f64> = None;

    for line in raw.lines() {
        if let Some((k, v)) = line.split_once(':') {
            match k.trim() {
                "CPU_USAGE"  => cpu_usage  = parse_f64(v).unwrap_or(0.0),
                "CPU_CORES"  => cpu_cores  = parse_i64(v),
                "RAM_TOTAL"  => ram_total  = parse_i64(v).unwrap_or(0),
                "RAM_USED"   => ram_used   = parse_i64(v).unwrap_or(0),
                "RAM_PCT"    => ram_pct    = parse_f64(v).unwrap_or(0.0),
                "DISK_TOTAL" => disk_total = parse_i64(v),
                "DISK_USED"  => disk_used  = parse_i64(v),
                "DISK_R"     => disk_r     = parse_i64(v),
                "DISK_W"     => disk_w     = parse_i64(v),
                "NET_RX"     => net_rx     = parse_i64(v),
                "NET_TX"     => net_tx     = parse_i64(v),
                "UPTIME"     => uptime     = parse_i64(v),
                "LOAD_1"     => load_1     = parse_f64(v),
                "LOAD_5"     => load_5     = parse_f64(v),
                "LOAD_15"    => load_15    = parse_f64(v),
                "TEMP_CPU"   => { if !v.trim().is_empty() { temp_cpu = parse_f64(v); } }
                _ => {}
            }
        }
    }

    MetricInput {
        server_id: server_id.to_string(),
        collected_at: chrono::Utc::now().to_rfc3339(),
        cpu_usage,
        cpu_cores,
        ram_used,
        ram_total,
        ram_usage_pct: ram_pct,
        temperature_cpu: temp_cpu,
        temperature_gpu: None,
        disk_used,
        disk_total,
        disk_read_bytes_sec: disk_r,
        disk_write_bytes_sec: disk_w,
        net_rx_bytes_sec: net_rx,
        net_tx_bytes_sec: net_tx,
        load_avg_1: load_1,
        load_avg_5: load_5,
        load_avg_15: load_15,
        uptime_seconds: uptime,
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Public API
// ─────────────────────────────────────────────────────────────────────────────

/// Collect metrics from a remote Linux host via SSH
pub fn collect_remote(server_id: &str, host: &str, user: &str, password: &str) -> Result<MetricInput> {
    let raw = run_ssh(host, user, password, COLLECT_SCRIPT)?;
    Ok(parse_output(&raw, server_id))
}

/// Test SSH connectivity — return OS info string on success
pub fn test_connection(host: &str, user: &str, password: &str) -> Result<String> {
    run_ssh(host, user, password, "uname -sr && hostname && echo 'OK'")
}
