# System Pulse

Server monitoring, in two shapes that share one database layer:

- **Desktop app** (Tauri) — local-first, SQLite file under the OS app-data dir
- **Standalone server** (Axum) — same SQLite schema, deployed via Docker with a persistent volume

```
system-pulse/
├── Cargo.toml                      ← workspace root
├── crates/
│   ├── system-pulse-db/            ← shared: models, migrations, queries (SQLite)
│   └── system-pulse-server/        ← Axum HTTP API, built on system-pulse-db
├── desktop/                         ← Tauri app (src-tauri/ depends on system-pulse-db too)
└── docker/                          ← server Dockerfile, compose, backup/restore scripts
```

## Why a separate `system-pulse-db` crate

Both binaries need identical behavior for users, servers, and metrics —
same columns, same migrations, same uniqueness rules. Before this split,
that logic was duplicated (and could silently drift) between the desktop
app and any future server. Now:

- `system-pulse-db` owns the schema, migrations, and every SQL query
- The desktop app calls `Database::connect(DatabaseConfig::at_path(app_data_dir))`
- The server calls `Database::connect(DatabaseConfig::from_env())`, reading
  `DATABASE_PATH` (defaults to `/data/system_pulse.db`, the Docker volume mount)

A `.db` file produced by one binary opens correctly in the other — same
tables, same indexes, same triggers.

---

## Running the shared db crate's tests

No external database needed — tests run against `sqlite::memory:`:

```bash
cargo test -p system-pulse-db
```

## Running the server locally (no Docker)

```bash
cd crates/system-pulse-server
export JWT_SECRET=$(openssl rand -hex 32)
export SERVER_ENC_KEY=$(openssl rand -hex 32)
export DATABASE_PATH=./dev.db
cargo run
# -> listening on 0.0.0.0:8090
```

## Deploying the server with Docker

```bash
cd system-pulse/
cp docker/.env.example docker/.env
# edit docker/.env — set JWT_SECRET and SERVER_ENC_KEY
#   openssl rand -hex 32   (run twice, once per secret)

docker compose -f docker/docker-compose.yml --env-file docker/.env up -d --build
```

The SQLite file lives in the `sqlite_data` named volume, mounted at
`/data/system_pulse.db` inside the container. It survives container
restarts and `docker compose down` (without `-v`).

### Backup / restore

```bash
# Snapshot the live database to ./backups/system_pulse_<timestamp>.db
./docker/backup.sh

# Restore a snapshot (stops + restarts the container)
./docker/restore.sh ./backups/system_pulse_20260620_120000.db
```

### Health check

```bash
curl http://localhost:8090/health
# {"status":"ok","db":"ok"}
```

---

## Running the desktop app

```bash
cd desktop
npm install
npm run tauri:dev      # dev mode
npm run tauri:build    # Windows .msi / .exe — see desktop/README.md
```

---

## API surface (server)

| Method | Path                              | Auth         | Notes |
|--------|------------------------------------|--------------|-------|
| POST   | `/api/auth/register`              | —            | |
| POST   | `/api/auth/login`                 | —            | |
| POST   | `/api/auth/logout`                | Bearer       | revokes current session |
| GET    | `/api/auth/me`                    | Bearer       | |
| POST   | `/api/account/changepassword`     | Bearer       | revokes all sessions |
| POST   | `/api/account/changeemail`        | Bearer       | |
| POST   | `/api/account/changelogin`        | Bearer       | |
| GET    | `/api/servers`                    | Bearer       | |
| POST   | `/api/servers`                    | Bearer       | |
| GET    | `/api/servers/:id`                | Bearer       | |
| PUT    | `/api/servers/:id`                | Bearer       | |
| DELETE | `/api/servers/:id`                | Bearer       | |
| GET    | `/api/metrics/:server_id`         | Bearer       | `?limit=120` |
| GET    | `/api/metrics/:server_id/latest`  | Bearer       | |
| POST   | `/api/metrics/:server_id/collect` | Bearer       | SSH-collects once, only for `server_type = "remote"` |
| GET    | `/health`                         | —            | |

Sessions are tracked server-side (the `sessions` table) so logout and
password changes can revoke tokens immediately — something the desktop
app doesn't need, since it never sends its JWT anywhere over a network.

---

## Security notes

- Passwords hashed with Argon2id
- JWTs signed HS256; the server additionally stores a SHA-256 hash of each
  issued token so it can revoke sessions without needing a JWT blocklist
  service
- SSH passwords are XOR+hex "encrypted" at rest — replace with AES-256-GCM
  before using this for anything beyond a homelab; set `SERVER_ENC_KEY` /
  the desktop's equivalent constant to a real secret either way
- The server container runs as a non-root user and only ships
  `sshpass` + `openssh-client` — no SSH server, no other attack surface
