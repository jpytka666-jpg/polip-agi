#!/usr/bin/env bash
set -euo pipefail

# ==========================================
# AUTHOR: M. SZUL
# AI MODEL: GPT-5.6 Luna
# TIMESTAMP: 2026-08-28 05:55:00
# REASON FOR CREATION: Central host-port allocation for Darkstar and future services.
# MECHANICS: Reads the tracked registry, checks live TCP listeners, reserves a unique host port under a file lock, and writes local runtime environment state.
# SYSTEM PART: Port Manager / deployment infrastructure
# ARCHITECTURE FUNCTION: Single source of truth for host-port allocation without hard-coded host ports in Compose.
# DEPENDENCIES/LINKS: registry.yaml, deploy/.env, Docker Compose, Linux ss/flock, Bash.
# TECH STACK: Bash; selected for direct Linux host integration and minimal dependencies.
# LOCAL WORKSPACE: /home/owner/polip-agi/deploy/port-manager/port-manager.sh
# GIT COMMIT: PENDING
# GITHUB METADATA: jpytka666-jpg/polip-agi, feat/darkstar-module-control
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
  local service="$1" key="$2"
  awk -v service="$service" -v key="$key" '
    $0 == "  " service ":" { in_service=1; next }
    in_service && $0 ~ /^  [^[:space:]].*:/ { exit }
    in_service && $0 ~ "^    " key ":[[:space:]]*" {
      line=$0
      sub(/^[[:space:]]+/, "", line)
      sub("^" key ":[[:space:]]*", "", line)
      print line
      exit
    }
  ' "$REGISTRY"
}

range_value() {
  local key="$1"
  awk -v k="$key" '$0 ~ "^  " k ":[[:space:]]*" {sub(/^[^:]*:[[:space:]]*/, ""); print; exit}' "$REGISTRY"
}

require_value() {
  local label="$1" value="$2"
  if [ -z "$value" ]; then
    echo "Port Manager error: missing $label in registry" >&2
    return 1
  fi
}

port_is_free() {
  local port="$1"
  ! ss -H -ltn "sport = :$port" 2>/dev/null | grep -q .
}

port_is_reserved() {
  local port="$1"
  awk -F '\t' -v p="$port" 'NF && $2 == p {found=1; exit 0} END {if (found) exit 0; exit 1}' "$STATE"
}

get_allocated() {
  local service="$1"
  awk -F '\t' -v s="$service" '$1 == s {print; exit}' "$STATE"
}

write_runtime_env() {
  local tmp="$RUNTIME_ENV.tmp"
  : > "$tmp"
  while IFS=$'\t' read -r service host_port container_port protocol timestamp; do
    [ -n "${service:-}" ] || continue
    case "$service" in
      darkstar) printf 'DARKSTAR_HOST_PORT=%s\n' "$host_port" >> "$tmp" ;;
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

format_allocation() {
  local service="$1" host_port="$2" container_port="$3" protocol="$4" exposure="$5"
  printf '%s -> host:%s:%s container:%s/%s\n' "$service" "$exposure" "$host_port" "$container_port" "$protocol"
}

allocate_one() {
  local service="$1"
  local existing
  existing="$(get_allocated "$service" || true)"
  if [ -n "$existing" ]; then
    local e_service e_host e_container e_protocol _timestamp
    IFS=$'\t' read -r e_service e_host e_container e_protocol _timestamp <<< "$existing"
    require_value "allocated service" "$e_service"
    require_value "allocated host port" "$e_host"
    require_value "allocated container port" "$e_container"
    require_value "allocated protocol" "$e_protocol"
    format_allocation "$e_service" "$e_host" "$e_container" "$e_protocol" "127.0.0.1"
    return 0
  fi

  local start end container_port protocol exposure
  start="$(range_value start)"
  end="$(range_value end)"
  container_port="$(service_value "$service" container_port)"
  protocol="$(service_value "$service" protocol)"
  exposure="$(service_value "$service" exposure)"

  require_value "registry start" "$start"
  require_value "registry end" "$end"
  require_value "container port for $service" "$container_port"
  require_value "protocol for $service" "$protocol"
  require_value "exposure for $service" "$exposure"

  [[ "$start" =~ ^[0-9]+$ ]] || { echo "Invalid registry start port: $start" >&2; return 1; }
  [[ "$end" =~ ^[0-9]+$ ]] || { echo "Invalid registry end port: $end" >&2; return 1; }
  [[ "$container_port" =~ ^[0-9]+$ ]] || { echo "Invalid container port for $service: $container_port" >&2; return 1; }
  [ "$protocol" = "tcp" ] || { echo "Unsupported protocol for $service: $protocol" >&2; return 1; }
  [ "$exposure" = "localhost" ] || { echo "Unsupported exposure for $service: $exposure" >&2; return 1; }
  (( start <= end )) || { echo "Invalid registry range: $start-$end" >&2; return 1; }

  for ((port=start; port<=end; port++)); do
    if port_is_free "$port" && ! port_is_reserved "$port"; then
      printf '%s\t%s\t%s\t%s\t%s\n' "$service" "$port" "$container_port" "$protocol" "$(date -u +%Y-%m-%dT%H:%M:%SZ)" >> "$STATE"
      write_runtime_env
      format_allocation "$service" "$port" "$container_port" "$protocol" "$exposure"
      return 0
    fi
  done

  echo "No free port in registry range ${start}-${end}" >&2
  return 1
}

release_one() {
  local service="$1"
  local tmp="$STATE.tmp"
  awk -F '\t' -v s="$service" '$1 != s' "$STATE" > "$tmp"
  mv "$tmp" "$STATE"
  write_runtime_env
  echo "released: $service"
}

list_all() {
  printf 'SERVICE\tHOST_PORT\tCONTAINER_PORT\tPROTOCOL\tSTATE\n'
  while IFS=$'\t' read -r service host_port container_port protocol timestamp; do
    [ -n "${service:-}" ] || continue
    if port_is_free "$host_port"; then state="reserved/free"; else state="reserved/in-use"; fi
    printf '%s\t%s\t%s\t%s\t%s\n' "$service" "$host_port" "$container_port" "$protocol" "$state"
  done < "$STATE"
}

audit_all() {
  local rc=0 start end
  start="$(range_value start)"; end="$(range_value end)"
  require_value "registry start" "$start" || rc=1
  require_value "registry end" "$end" || rc=1
  echo "registry: $start-$end"
  if awk -F '\t' 'NF && seen[$2]++ {print "DUPLICATE_HOST_PORT " $2; bad=1} END{exit bad}' "$STATE"; then
    echo "allocation uniqueness: OK"
  else rc=1; fi
  while IFS=$'\t' read -r service host_port container_port protocol timestamp; do
    [ -n "${service:-}" ] || continue
    if [ -z "$host_port" ] || [ -z "$container_port" ] || [ -z "$protocol" ]; then
      echo "$service -> INVALID_ALLOCATION"
      rc=1
      continue
    fi
    if port_is_free "$host_port"; then
      echo "$service host:$host_port -> free"
    else
      echo "$service host:$host_port -> IN USE"
    fi
  done < "$STATE"
  if [ -f "$RUNTIME_ENV" ]; then echo "runtime env: present"; else echo "runtime env: missing"; rc=1; fi
  return "$rc"
}

main() {
  local command="${1:-}" service="${2:-}"
  case "$command" in
    list) list_all ;;
    allocate)
      [ -n "$service" ] || { usage >&2; exit 2; }
      lock_run allocate_one "$service"
      ;;
    release)
      [ -n "$service" ] || { usage >&2; exit 2; }
      lock_run release_one "$service"
      ;;
    audit|check) audit_all ;;
    *) usage >&2; exit 2 ;;
  esac
}

main "$@"
