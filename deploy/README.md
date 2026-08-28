# Dark Star supervised runtime

This directory defines the first Ubuntu runtime deployment for the existing `darkstar-server` image.

## Runtime

Build the image from the repository root:

```bash
docker build -t darkstar:dev .
```

Start the Compose service:

```bash
docker compose -f deploy/docker-compose.yml up -d
```

Inspect the service and health:

```bash
docker compose -f deploy/docker-compose.yml ps
curl -fsS http://127.0.0.1:8080/health
```

View recent logs:

```bash
docker compose -f deploy/docker-compose.yml logs --tail 50
```

Stop the service:

```bash
docker compose -f deploy/docker-compose.yml down
```

## Contract

The service uses `DARKSTAR_HOST=0.0.0.0` and `DARKSTAR_PORT=8080` and publishes host port `8080` to container port `8080`.

The image remains non-root. Compose provides the container lifecycle contract and application healthcheck. Host boot supervision is added by the systemd unit in the next stage.
