#!/usr/bin/env bash
set -euo pipefail

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
  python3 - "$REGISTRY" "$service" "$key" <<'PY'
import sys
from pathlib import Path

path, service, key = sys.argv[1:]
lines = Path(path).read_text(encoding="utf-8").splitlines()
in_service = False
for raw in lines:
    line = raw.rstrip()
    if line.startswith("  ") and not line.startswith("    "):
        in_service = False
    if line == f"  {service}:":
        in_service = True
        continue
    if in_service and line.startswith("    "):
        stripped = line.strip()
        if stripped.startswith(f"{key}:"):
            print(stripped.split(":", 1)[1].strip())
            raise SystemExit(0)
raise SystemExit(1)
PY
}

range_value() {
  local key="$1"
  awk -F': *' -v k="$key" '$1 ~ "^  " k "$" {print $2; exit}' "$REGISTRY"
}

port_is_free() {
  local port="$1"
  ! ss -H -ltn "sport = :$port" 2>/dev/null | grep -q .
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

allocate_one() {
  local service="$1"
  local existing
  existing="$(get_allocated "$service" || true)"
  if [ -n "$existing" ]; then
    echo "$existing" | awk -F '\t' '{printf "%s -> host:%s container:%s/%s\n", $1,$2,$3,$4}'
    return 0
  fi

  local start end container_port protocol exposure
  start="$(range_value start)"
  end="$(range_value end)"
  container_port="$(service_value "$service" container_port)"
  protocol="$(service_value "$service" protocol)"
  exposure="$(service_value "$service" exposure)"

  for ((port=start; port<=end; port++)); do
    if port_is_free "$port" && ! awk -F '\t' -v p="$port" '$2 == p {found=1} END{exit found}' "$STATE"; then
      printf '%s\t%s\t%s\t%s\t%s\n' "$service" "$port" "$container_port" "$protocol" "$(date -u +%Y-%m-%dT%H:%M:%SZ)" >> "$STATE"
      write_runtime_env
      echo "$service -> host:${exposure}:${port} container:${container_port}/${protocol}"
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
  echo "registry: $start-$end"
  if awk -F '\t' 'seen[$2]++ {print "DUPLICATE_HOST_PORT " $2; bad=1} END{exit bad}' "$STATE"; then
    echo "allocation uniqueness: OK"
  else rc=1; fi
  while IFS=$'\t' read -r service host_port container_port protocol timestamp; do
    [ -n "${service:-}" ] || continue
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
