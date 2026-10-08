#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$REPO_ROOT"

COMPOSE_FILE="${SRS_COMPOSE_FILE:-$REPO_ROOT/compose.srs.yml}"

if command -v docker >/dev/null 2>&1; then
  if docker compose version >/dev/null 2>&1; then
    COMPOSE=(docker compose)
  elif command -v docker-compose >/dev/null 2>&1; then
    COMPOSE=(docker-compose)
  else
    echo "Error: neither 'docker compose' nor 'docker-compose' was found" >&2
    exit 1
  fi
else
  echo "Error: docker is required to run SRS integration tests" >&2
  exit 1
fi

SRS_HTTP_API_HOST="${SRS_HTTP_API_HOST:-127.0.0.1}"
SRS_HTTP_API_PORT="${SRS_HTTP_API_PORT:-1985}"
export SRS_HTTP_API_URL="http://${SRS_HTTP_API_HOST}:${SRS_HTTP_API_PORT}"
export SRS_RTMP_URL="rtmp://${SRS_HTTP_API_HOST}:${SRS_RTMP_PORT:-1935}/live"

if ! command -v curl >/dev/null 2>&1; then
  echo "Error: curl is required to wait for SRS readiness" >&2
  exit 1
fi

compose() {
  "${COMPOSE[@]}" -f "$COMPOSE_FILE" "$@"
}

BASE_URL="${SRS_HTTP_API_URL%/}"
VERSION_URL="${BASE_URL}/api/v1/versions"
STARTUP_TIMEOUT_SECONDS="${SRS_STARTUP_TIMEOUT_SECONDS:-60}"

if [[ "$STARTUP_TIMEOUT_SECONDS" -le 0 ]]; then
  echo "Error: SRS_STARTUP_TIMEOUT_SECONDS must be greater than 0" >&2
  exit 1
fi

cleanup() {
  if [[ "${KEEP_SRS_COMPOSE:-0}" == "1" ]]; then
    echo "Keeping SRS running due to KEEP_SRS_COMPOSE=1"
    return
  fi

  echo "Tearing down SRS..."
  compose down -v || true
}
trap cleanup EXIT

# Start dependencies
compose up -d

for ((i = 1; i <= STARTUP_TIMEOUT_SECONDS; i++)); do
  if curl -fsS "$VERSION_URL" >/dev/null; then
    echo "SRS is ready"
    break
  fi

  if [[ "$i" -eq "$STARTUP_TIMEOUT_SECONDS" ]]; then
    echo "Error: SRS did not become ready at $VERSION_URL within $STARTUP_TIMEOUT_SECONDS seconds" >&2
    compose logs
    exit 1
  fi

  sleep 1
done

if [[ "$#" -eq 0 ]]; then
  set -- --all-targets -- --include-ignored --test-threads=1
fi

cargo test "$@"
