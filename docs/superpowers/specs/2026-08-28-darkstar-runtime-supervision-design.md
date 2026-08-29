<!--
THIS IS VERY IMPORTANT!!!
==========================================
AUTHOR: M. SZUL
AI MODEL: GPT-5 Codex
TIMESTAMP: 2026-08-29 11:19:11 Europe/London
REASON FOR CREATION: Aktualizacja specyfikacji nadzoru runtime o stan checkpointu i nazwę Warlock.
==========================================
-->

# Dark Star Runtime Supervision Design

## Goal

Make the Dark Star runtime on the Ubuntu host start automatically with the host, remain continuously observable, recover quickly from process/container failure, and retry unavailable external dependencies on a bounded 15-minute cadence without turning recovery into an uncontrolled loop.

## Architecture

Docker Compose owns the Dark Star container definition and application-level healthcheck. A host-level systemd unit owns lifecycle at boot and delegates container lifecycle to Compose. Process/container failure is recovered immediately by systemd/Docker restart semantics; dependency recovery is bounded separately by a watchdog/health supervisor with a 15-minute retry interval. No model or agent participates in supervision decisions.

## Scope

This design covers the first Ubuntu host deployment of the existing `darkstar-server` container. It does not add Warlock (formerly Sheriff), Kali, provider registry behavior, external cloud deployment, or Power Automate integration.

## Runtime contract

The existing Dark Star server listens on `DARKSTAR_HOST` and `DARKSTAR_PORT`, defaulting to `0.0.0.0:8080`. The container image runs as the non-root `darkstar` user. The existing `/health` endpoint returns a JSON health response containing the service name, API version, and `status: ok`.

## Startup

At Ubuntu boot, systemd must start the Dark Star Compose stack automatically after Docker is available. The startup operation must be idempotent: starting an already-running stack must not create duplicate containers or fail because the desired state already exists.

## Recovery

Container/process failure must trigger a bounded immediate restart through the container/runtime supervisor. Restart policy must have an explicit maximum retry window or equivalent guard so a permanently broken image/configuration does not create an unbounded crash loop hidden from operators.

Application health must be checked independently of process liveness. A running process that no longer answers `/health` is unhealthy and must be restarted or marked for recovery according to the supervisor policy.

## Dependency retry

A temporary failure to reach an external dependency must not cause a tight retry loop. Dependency retries use a 15-minute interval. The dependency state must be observable in logs, and recovery attempts must be idempotent.

The 15-minute interval does not replace immediate process/container recovery. It applies only to unavailable external dependencies or connections whose failure does not require restarting a healthy Dark Star process.

## Shutdown and restart

System shutdown must stop the Compose stack cleanly. Manual restart must use the same systemd/Compose lifecycle path as boot so there is one authoritative operational mechanism.

## Observability

The operator must be able to inspect, without modifying the deployment:

- whether the systemd unit is enabled and active;
- whether the Dark Star container is running;
- whether the application healthcheck is passing;
- recent restart events;
- dependency retry events and outcomes;
- container logs.

## Security boundaries

The supervisor must not grant additional application privileges. The Dark Star container remains non-root. Supervisor credentials, if any are later required for a provider, must not be passed as plain environment values merely to make recovery work. Recovery remains mechanical and policy-independent.

## Failure semantics

1. Host boots -> Docker available -> Compose stack starts.
2. Container exits -> Docker/systemd recovery attempts restart immediately.
3. Container remains running but `/health` fails -> health supervisor marks it unhealthy and invokes the defined recovery action.
4. External dependency is unavailable -> dependency status is recorded and retried every 15 minutes with no busy loop.
5. Recovery remains unsuccessful -> the system exposes a persistent failure state in logs/status instead of silently looping forever.

## Testing requirements

The implementation must provide tests or reproducible checks for:

- boot/start idempotency;
- healthy container startup;
- process/container restart after termination;
- unhealthy application detection;
- 15-minute dependency retry scheduling logic without waiting 15 minutes in unit tests;
- clean stop/restart lifecycle;
- non-root container execution;
- final `cargo test`, formatting, Clippy, release build, image build, container start, and health verification on Ubuntu.

## Non-goals

Do not introduce a second workflow engine. Do not put a permanent `while true` loop inside the Dark Star application container. Do not use an AI model for supervision. Do not add cloud-specific orchestration just for local recovery. Do not merge this deployment work with Warlock/Kali implementation.

## Relationship to existing architecture

The runtime supervisor is infrastructure around the existing control plane. It does not replace `Policy`, `Orchestrator`, `AuthorizedModuleCommand`, or `ModuleProvider`. It only keeps the already-authorized Dark Star runtime alive and observable.
