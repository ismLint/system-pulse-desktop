#!/usr/bin/env bash
set -euo pipefail

CONTAINER="system_pulse_server"
BACKUP_FILE="${1:?Usage: restore.sh <path-to-backup.db>}"

if [ ! -f "$BACKUP_FILE" ]; then
  echo "Backup file not found: $BACKUP_FILE" >&2
  exit 1
fi

echo "Stopping ${CONTAINER}..."
docker stop "$CONTAINER"

echo "Copying ${BACKUP_FILE} into the volume..."
docker cp "$BACKUP_FILE" "${CONTAINER}:/data/system_pulse.db"

echo "Starting ${CONTAINER}..."
docker start "$CONTAINER"

echo "Done. Tail logs with: docker logs -f ${CONTAINER}"
