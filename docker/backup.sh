#!/usr/bin/env bash
# docker/backup.sh — snapshot the live SQLite database out of the running
# container into a timestamped file on the host.
#
# SQLite's `.backup` command is safe to run against a live database (it
# uses the same locking the WAL mode already relies on), so this does not
# require stopping the server.
set -euo pipefail

CONTAINER="system_pulse_server"
OUT_DIR="${1:-./backups}"
TIMESTAMP="$(date +%Y%m%d_%H%M%S)"
OUT_FILE="${OUT_DIR}/system_pulse_${TIMESTAMP}.db"

mkdir -p "$OUT_DIR"

echo "Backing up ${CONTAINER}:/data/system_pulse.db -> ${OUT_FILE}"

docker exec "$CONTAINER" sh -c \
  "command -v sqlite3 >/dev/null 2>&1 && sqlite3 /data/system_pulse.db '.backup /data/_backup_tmp.db' || cp /data/system_pulse.db /data/_backup_tmp.db"

docker cp "${CONTAINER}:/data/_backup_tmp.db" "$OUT_FILE"
docker exec "$CONTAINER" rm -f /data/_backup_tmp.db

echo "Done: $OUT_FILE ($(du -h "$OUT_FILE" | cut -f1))"
