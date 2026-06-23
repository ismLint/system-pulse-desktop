# System Pulse Desktop 🖥️

Tauri v2 desktop application for real-time remote server monitoring via SSH.  
**Stack:** Tauri 2 · Rust · SQLite (sqlx) · React 18 · Vite · TypeScript · Recharts

> This app lives inside the `system-pulse` Cargo workspace and depends on
> the shared [`system-pulse-db`](../crates/system-pulse-db) crate for all
> models, migrations, and SQL queries. The same crate also backs the
> standalone server in [`crates/system-pulse-server`](../crates/system-pulse-server) —
> see the [workspace README](../README.md) for how the two relate and how
> to deploy the server.

---

## Features

- 🔐 Local auth (Argon2 passwords, JWT stored on device)
- 📡 SSH-based metric collection — **no agent on remote server**
- 📊 Live charts: CPU, RAM, Temperature, Disk I/O, Network, Load Average
- 💾 SQLite database stored in `%APPDATA%\system-pulse-desktop\system_pulse.db`
- 🪟 Custom frameless window with native title bar controls
- ⚡ Metrics polled every 5 seconds via Tauri background tasks + events

---

## Prerequisites

| Tool | Install |
|------|---------|
| Rust (stable) | https://rustup.rs |
| Node.js 20+ | https://nodejs.org |
| Tauri CLI v2 | `cargo install tauri-cli --version "^2.0"` |
| sshpass (Windows) | `choco install sshpass` |
| WebView2 | Included in Windows 11; for Windows 10: https://developer.microsoft.com/en-us/microsoft-edge/webview2/ |

---

## Development

```powershell
# Install JS dependencies
npm install

# Start dev mode (hot-reload frontend + Tauri window)
npm run tauri:dev
```

## Build (Windows .msi / .exe)

```powershell
npm run tauri:build
# Output: src-tauri/target/release/bundle/
```

---

## Project Structure

```
system-pulse-desktop/
├── src-tauri/
│   └── src/
│       ├── commands/     # Tauri commands (auth, account, servers, metrics, window)
│       ├── db/           # SQLite init + migrations
│       ├── models/       # Rust structs (User, Server, Metric, AppError)
│       ├── services/     # JWT auth, Argon2 crypto, SSH metric collector
│       ├── lib.rs        # Tauri setup, command registration
│       └── main.rs       # Entry point
├── src/
│   ├── components/       # UI kit, MetricChart, AuthForm, AppLayout, TitleBar
│   ├── hooks/            # useMetrics (Tauri events), useServers
│   ├── pages/            # Login, Register, Dashboard, Servers, ServerDetail, Account
│   ├── store/            # Zustand auth store (persisted)
│   └── utils/            # invoke wrapper, formatters
└── tauri.conf.json
```

---

## How SSH Metrics Work

1. User adds a server with host, SSH user and password
2. Password is XOR-encrypted and stored in SQLite
3. On the **Server Detail** page, Tauri spawns a background async task
4. Every 5 seconds, the task runs a shell script on the remote host via `sshpass + ssh`
5. Script collects CPU, RAM, disk I/O, network, temperature, load average, uptime
6. Result is saved to SQLite and emitted as a Tauri event `metric:<server_id>`
7. Frontend React hook `useMetrics` listens for these events and appends to chart data

No agent, no port forwarding — just standard SSH.

---

## Security Notes

- Passwords hashed with **Argon2id**
- JWT signed with HS256, stored in Zustand (persisted to localStorage via tauri-plugin-store in prod)
- SSH passwords encrypted with XOR + hex — upgrade to AES-256-GCM for production
- SQLite file lives in the OS app data directory (not accessible to other users)

---

## Troubleshooting

### `conflicting implementations of trait From<...HourBase>` (the `time` crate)

This happens when `sqlx` and `tauri` pull in incompatible versions of the
`time` crate. Fixed in this project by pinning **sqlx 0.8** (not 0.7) —
sqlx 0.8 aligns its `time` dependency with what Tauri 2 expects.

If you still hit this after `cargo clean`, run:

```powershell
cd src-tauri
cargo update -p time
```

### Local metrics collection

Local-machine metrics use the [`sysinfo`](https://docs.rs/sysinfo) crate
instead of manually parsing `/proc` — this makes local monitoring work on
**Windows, Linux, and macOS** alike (CPU, RAM, disk, network, temperature
where sensors are exposed by the OS; `load_avg` is Unix-only and reports
`null` on Windows).

### App icons

Placeholder icons are included in `src-tauri/icons/`. Replace them with
your own before shipping — `tauri build` requires valid `.ico` (Windows)
files to exist at the paths listed in `tauri.conf.json`.
