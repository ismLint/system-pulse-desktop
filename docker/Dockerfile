FROM rust:1.81-slim-bookworm AS builder

WORKDIR /build

RUN apt-get update && apt-get install -y --no-install-recommends \
        pkg-config \
        libssl-dev \
    && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml ./Cargo.toml
COPY crates/system-pulse-db        ./crates/system-pulse-db
COPY crates/system-pulse-server    ./crates/system-pulse-server

RUN printf '[workspace]\nresolver = "2"\nmembers = ["crates/system-pulse-db", "crates/system-pulse-server"]\n' \
    > Cargo.toml

RUN cargo build --release -p system-pulse-server

FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y --no-install-recommends \
        ca-certificates \
        sshpass \
        openssh-client \
        curl \
    && rm -rf /var/lib/apt/lists/*

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
