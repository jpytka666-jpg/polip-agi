# Dark Star supervised runtime

## Host firewall

The versioned host firewall is in [firewall/README.md](firewall/README.md).
It owns only `inet darkstar_host_guard`, keeps the API closed to public input,
and has separate install, verification and rollback instructions. Applying it
to a live host is a later manual operation that requires physical access and
timestamped root-owned backups; this repository task does not change live
rules.

This directory defines the first Ubuntu runtime deployment for the existing `darkstar-server` image.

## Port Manager

Host ports are not hard-coded in the Compose service. The tracked `deploy/port-manager/registry.yaml` defines the host-port pool and service contracts. The operator allocates a free host port, checks live listeners, and writes local runtime state to the ignored `deploy/.env` file.

Allocate and inspect the Dark Star host port:

```bash
./deploy/port-manager/port-manager.sh allocate darkstar
./deploy/port-manager/port-manager.sh list
./deploy/port-manager/port-manager.sh audit
```

The container keeps its application port at `8080`; only the host-side port is variable. The current deployment binds the host port to `127.0.0.1` by design so the service is not exposed on every host interface.

## Runtime

Build the image from the repository root:

```bash
docker build -t darkstar:dev .
```

Allocate a host port and start the Compose service:

```bash
./deploy/port-manager/port-manager.sh allocate darkstar
docker compose --env-file deploy/.env -f deploy/docker-compose.yml up -d
```

Inspect the service and health:

```bash
docker compose --env-file deploy/.env -f deploy/docker-compose.yml ps
curl -fsS http://127.0.0.1:$(awk -F= '/^DARKSTAR_HOST_PORT=/{print $2}' deploy/.env)/health
```

View recent logs:

```bash
docker compose --env-file deploy/.env -f deploy/docker-compose.yml logs --tail 50
```

Stop the service:

```bash
docker compose --env-file deploy/.env -f deploy/docker-compose.yml down
```

## Contract

The service binds explicitly to `DARKSTAR_HOST` and keeps `DARKSTAR_PORT` in sync with the allocated host port. The default bind address is `192.168.2.1` - the DARKSTAR-WiFi gateway - so the `/world/` landing is reachable from that private network. `0.0.0.0` is forbidden: with `network_mode: host` it would also expose the service on `wlp2s0`, the upstream Vodafone segment. The host port is allocated centrally by Port Manager from the registered pool `18080-18999`.

Static files under the built frontend (including `/world/`) are served by the fallback service and need no token. Every `/v1/*` route checks the `Authorization: Bearer` header inside its own handler, so opening the landing does not open the API. Copy `deploy/.env.example` to `deploy/.env`, fill in the real values and `chmod 0600` it.

The image remains non-root. Compose provides the container lifecycle contract and application healthcheck. Host boot supervision is added by the systemd unit in the next stage.
