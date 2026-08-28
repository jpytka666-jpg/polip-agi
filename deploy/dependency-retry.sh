#!/usr/bin/env bash
# THIS IS VERY IMPORTANT!!!
# ==========================================
# AUTHOR: M. SZUL
# AI MODEL: GPT-5.6 Luna
# TIMESTAMP: 2026-08-28 07:05:00
# REASON FOR CREATION: Perform one bounded Darkstar connectivity recovery attempt.
# MECHANICS: Probe the published health endpoint once; if unhealthy, reconcile the Compose stack once and verify health again.
# SYSTEM PART: Darkstar runtime supervision
# ARCHITECTURE FUNCTION: Fifteen-minute recovery attempt for runtime connectivity without a permanent loop.
# DEPENDENCIES/LINKS: deploy/.env, deploy/docker-compose.yml, Docker Engine, curl, systemd timer.
# TECH STACK: Bash; selected because the host already uses Docker Compose and systemd and this task needs only deterministic shell orchestration.
# LOCAL WORKSPACE: /home/owner/polip-agi
# GIT COMMIT: PENDING
# GITHUB METADATA: jpytka666-jpg/polip-agi / feat/darkstar-module-control
# ==========================================

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DEPLOY="$ROOT/deploy"
COMPOSE_FILE="$DEPLOY/docker-compose.yml"
ENV_FILE="$DEPLOY/.env"

log() {
  printf '[darkstar-dependency-retry] %s\n' "$*"
}

require_file() {
  local path="$1"
  [ -f "$path" ] || {
    log "missing required file: $path"
    return 1
  }
}

health_url() {
  local port
  port="$(awk -F= '/^DARKSTAR_HOST_PORT=/{print $2; exit}' "$ENV_FILE")"
  [ -n "$port" ] || {
    log "DARKSTAR_HOST_PORT is missing from $ENV_FILE"
    return 1
  }
  printf 'http://127.0.0.1:%s/health\n' "$port"
}

main() {
  require_file "$COMPOSE_FILE"
  require_file "$ENV_FILE"

  local url
  url="${DARKSTAR_HEALTH_URL:-$(health_url)}"

  if curl -fsS --max-time 5 "$url" >/dev/null; then
    log "health check OK: $url"
    return 0
  fi

  log "health check failed; attempting one Compose reconciliation"
  /usr/bin/docker compose --env-file "$ENV_FILE" -f "$COMPOSE_FILE" up -d --remove-orphans

  if curl -fsS --max-time 5 "$url" >/dev/null; then
    log "recovery OK: $url"
    return 0
  fi

  log "recovery failed: $url"
  return 1
}

main "$@"
