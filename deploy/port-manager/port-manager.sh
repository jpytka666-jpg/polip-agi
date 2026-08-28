#!/usr/bin/env bash
set -euo pipefail

# THIS IS VERY IMPORTANT!!!
# ==========================================
# AUTHOR: M. SZUL
# AI MODEL: GPT-5.6 Luna
# TIMESTAMP: 2026-08-28 06:10:00
# REASON FOR CREATION: Central dynamic host-port allocation for Darkstar and future services without hard-coded host-port collisions.
# MECHANICS: Reads the tracked registry, checks live TCP listeners, reserves one free host port under an exclusive file lock, and writes local runtime environment state.
# SYSTEM PART: Deployment infrastructure / Port Manager
# ARCHITECTURE FUNCTION: Single local authority for host-port allocation used by Docker Compose.
# DEPENDENCIES/LINKS: deploy/port-manager/registry.yaml, deploy/.env, docker-compose.yml, ss, awk, flock.
# TECH STACK: Bash with standard Linux utilities; selected for direct host networking visibility and deployment portability.
# LOCAL WORKSPACE: /home/owner/polip-agi/deploy/port-manager/port-manager.sh
# GIT COMMIT: PENDING
# GITHUB METADATA: Repository jpytka666-jpg/polip-agi, branch feat/darkstar-module-control
# ==========================================

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
DEPLOY="$ROOT/deploy"
REGISTRY="$DEPLOY/port-manager/registry.yaml"
STATE_DIR="$DEPLOY/.port-manager"
STATE="$STATE_DIR/allocations.tsv"
RUNTIME_ENV="$DEPLOY/.env"
LOCK="$STATE_DIR/.lock"

mkdir -p "$STATE_DIR"
touch "$STATE"

usage() {
  cat <<EOF
Usage: $(basename "$0") <list|allocate|release|audit|check> [service]
EOF
}

service_value() {
  local requested_service="$1" requested_key="$2" value
  value="$(awk -v wanted_service="$requested_service" -v wanted_key="$requested_key" '
    $0 == "  " wanted_service ":" { in_service=1; next }
    in_service && $0 ~ /^  [^[:space:]].*:/ { exit }
    in_service && $0 ~ /^    / {
      line=$0
      sub(/^[[:space:]]+/, "", line)
      split(line, parts, ":")
      if (parts[1] == wanted_key) {
        value=substr(line, index(line, ":") + 1)
        gsub(/^[[:space:]]+|[[:space:]]+$/, "", value)
        print value
        found=1
        exit 0
      }
    }
    END { if (!found) exit 1 }
  ' "$REGISTRY")"
  [ -n "$value" ] || return 1
  printf '%s\n' "$value"
}

range_value() {
  local key="$1" value
  value="$(awk -v wanted_key="$key" '$0 ~ "^  " wanted_key ":[[:space:]]*" {sub(/^[^:]*:[[:space:]]*/, ""); print; exit}' "$REGISTRY")"
  [ -n "$value" ] || return 1
  printf '%s\n' "$value"
}

port_is_free() {
  local port="$1"
  ! ss -H -ltn "sport = :$port" 2>/dev/null | grep -q .
}

port_is_reserved() {
  local port="$1"
  awk -F '\t' -v wanted_port="$port" '$2 == wanted_port {found=1} END {exit(found ? 0 : 1)}' "$STATE"
}

get_allocated() {
  local requested_service="$1"
  awk -F '\t' -v wanted_service="$requested_service" '$1 == wanted_service {print; exit}' "$STATE"
}

write_runtime_env() {
  local tmp="$RUNTIME_ENV.tmp"
  : > "$tmp"
  while IFS=$'\t' read -r state_service state_host_port state_container_port state_protocol state_timestamp; do
    [ -n "${state_service:-}" ] || continue
    case "$state_service" in
      darkstar) printf 'DARKSTAR_HOST_PORT=%s\n' "$state_host_port" >> "$tmp" ;;
    esac
  done < "$STATE"
  chmod 600 "$tmp"
  mv "$tmp" "$RUNTIME_ENV"
}

lock_run() {
  exec 9>"$LOCK"
  flock -x 9
  "$@"
}

allocate_one() {
  local requested_service="$1"
  local existing
  existing="$(get_allocated "$requested_service" || true)"
  if [ -n "$existing" ]; then
    echo "$existing" | awk -F '\t' '{printf "%s -> host:127.0.0.1:%s container:%s/%s\n", $1,$2,$3,$4}'
    return 0
  fi

  local start end container_port protocol exposure
  start="$(range_value start)"
  end="$(range_value end)"
  container_port="$(service_value "$requested_service" container_port)"
  protocol="$(service_value "$requested_service" protocol)"
  exposure="$(service_value "$requested_service" exposure)"

  [[ "$start" =~ ^[0-9]+$ ]] || { echo "Invalid registry start port: $start" >&2; return 1; }
  [[ "$end" =~ ^[0-9]+$ ]] || { echo "Invalid registry end port: $end" >&2; return 1; }
  [[ "$container_port" =~ ^[0-9]+$ ]] || { echo "Invalid container port for $requested_service: $container_port" >&2; return 1; }
  [ "$protocol" = "tcp" ] || { echo "Unsupported protocol for $requested_service: $protocol" >&2; return 1; }
  [ "$exposure" = "localhost" ] || { echo "Unsupported exposure for $requested_service: $exposure" >&2; return 1; }

  for ((port=start; port<=end; port++)); do
    if port_is_free "$port" && ! port_is_reserved "$port"; then
      printf '%s\t%s\t%s\t%s\t%s\n' "$requested_service" "$port" "$container_port" "$protocol" "$(date -u +%Y-%m-%dT%H:%M:%SZ)" >> "$STATE"
      write_runtime_env
      echo "$requested_service -> host:${exposure}:${port} container:${container_port}/${protocol}"
      return 0
    fi
  done

  echo "No free port in registry range ${start}-${end}" >&2
  return 1
}

release_one() {
  local requested_service="$1"
  local tmp="$STATE.tmp"
  awk -F '\t' -v wanted_service="$requested_service" '$1 != wanted_service' "$STATE" > "$tmp"
  mv "$tmp" "$STATE"
  write_runtime_env
  echo "released: $requested_service"
}

list_all() {
  printf 'SERVICE\tHOST_PORT\tCONTAINER_PORT\tPROTOCOL\tSTATE\n'
  while IFS=$'\t' read -r list_service list_host_port list_container_port list_protocol list_timestamp; do
    [ -n "${list_service:-}" ] || continue
    if port_is_free "$list_host_port"; then list_state="reserved/free"; else list_state="reserved/in-use"; fi
    printf '%s\t%s\t%s\t%s\t%s\n' "$list_service" "$list_host_port" "$list_container_port" "$list_protocol" "$list_state"
  done < "$STATE"
}

audit_all() {
  local rc=0 start end
  start="$(range_value start)"; end="$(range_value end)"
  echo "registry: $start-$end"
  if awk -F '\t' 'seen[$2]++ {print "DUPLICATE_HOST_PORT " $2; bad=1} END{exit bad}' "$STATE"; then
    echo "allocation uniqueness: OK"
  else rc=1; fi
  while IFS=$'\t' read -r audit_service audit_host_port audit_container_port audit_protocol audit_timestamp; do
    [ -n "${audit_service:-}" ] || continue
    if [ -z "$audit_host_port" ] || [ -z "$audit_container_port" ] || [ -z "$audit_protocol" ]; then
      echo "$audit_service -> INVALID_ALLOCATION"
      rc=1
      continue
    fi
    if port_is_free "$audit_host_port"; then
      echo "$audit_service host:$audit_host_port -> free"
    else
      echo "$audit_service host:$audit_host_port -> IN USE"
    fi
  done < "$STATE"
  if [ -f "$RUNTIME_ENV" ]; then echo "runtime env: present"; else echo "runtime env: missing"; rc=1; fi
  return "$rc"
}

main() {
  local command="${1:-}" requested_service="${2:-}"
  case "$command" in
    list) list_all ;;
    allocate)
      [ -n "$requested_service" ] || { usage >&2; exit 2; }
      lock_run allocate_one "$requested_service"
      ;;
    release)
      [ -n "$requested_service" ] || { usage >&2; exit 2; }
      lock_run release_one "$requested_service"
      ;;
    audit|check) audit_all ;;
    *) usage >&2; exit 2 ;;
  esac
}

main "$@"
