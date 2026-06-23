# ── Build stage ──────────────────────────────────────────────────────────────
FROM rust:1.81-slim-bookworm AS builder

WORKDIR /build

# System deps needed to compile sqlx (sqlite, openssl for rustls deps, etc.)
RUN apt-get update && apt-get install -y --no-install-recommends \
        pkg-config \
        libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy the whole workspace — system-pulse-server depends on system-pulse-db
# via a path dependency, so both crates must be present to build either one.
COPY Cargo.toml ./Cargo.toml
COPY crates/system-pulse-db        ./crates/system-pulse-db
COPY crates/system-pulse-server    ./crates/system-pulse-server

# The desktop crate is intentionally NOT copied — it depends on Tauri and
# system libraries (webkit2gtk etc.) that have no business being in a
# server image. We build a workspace-less Cargo.toml just for this image
# so `cargo build` only ever sees the two server-relevant crates.
RUN printf '[workspace]\nresolver = "2"\nmembers = ["crates/system-pulse-db", "crates/system-pulse-server"]\n' \
    > Cargo.toml

RUN cargo build --release -p system-pulse-server

# ── Runtime stage ────────────────────────────────────────────────────────────
FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y --no-install-recommends \
        ca-certificates \
        sshpass \
        openssh-client \
        curl \
    && rm -rf /var/lib/apt/lists/*

# Non-root user; the SQLite file lives in /data, owned by this user.
RUN useradd -r -s /bin/false -u 10001 sppulse \
    && mkdir -p /data \
    && chown sppulse:sppulse /data

COPY --from=builder /build/target/release/system-pulse-server /usr/local/bin/system-pulse-server

USER sppulse
WORKDIR /data

EXPOSE 8090

ENV DATABASE_PATH=/data/system_pulse.db \
    HOST=0.0.0.0 \
    PORT=8090 \
    RUST_LOG=info

HEALTHCHECK --interval=15s --timeout=5s --start-period=10s --retries=3 \
    CMD curl -fsS http://localhost:8090/health || exit 1

ENTRYPOINT ["system-pulse-server"]
