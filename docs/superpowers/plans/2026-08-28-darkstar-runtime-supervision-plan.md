<!--
THIS IS VERY IMPORTANT!!!
==========================================
AUTHOR: M. SZUL
AI MODEL: GPT-5 Codex
TIMESTAMP: 2026-08-29 11:19:11 Europe/London
REASON FOR CREATION: Utrwalenie faktycznego stanu wdrożenia runtime oraz braku dowodu prawdziwego rebootu dla firewalla Darkstar.
==========================================
-->

# Dark Star Runtime Supervision Implementation Plan

## Handoff status at fd8099e

- Compose/systemd runtime is installed and darkstar.service is active on Ubuntu.
- The 15-minute dependency timer is active.
- Darkstar API was healthy on 127.0.0.1:18080.
- Host forwarding is enabled for IPv4 and IPv6.
- The separately installed darkstar-firewall.service is enabled but was
  inactive/dead on 2026-08-29 and had no journal entries.
- The firewall files were created after the current host boot, so boot
  persistence is not proven.
- The required real reboot test, including a changed boot_id, is now Task 3 of
  docs/superpowers/plans/2026-08-29-darkstar-native-gateway-headscale-headplane-plan.md.

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make the Dark Star runtime on the Ubuntu host start automatically at boot, expose application health, recover from container/process failure, and keep dependency retry bounded to a 15-minute cadence.

**Architecture:** Docker Compose owns the Dark Star container definition and application healthcheck. A host-level systemd unit owns boot/manual lifecycle and delegates container lifecycle to Compose. Process failure uses immediate bounded restart semantics; external dependency retries remain separate and occur every 15 minutes without a busy loop.

**Tech Stack:** Rust 2024, Axum 0.8, Tokio, Docker, Docker Compose v2, systemd, Bash/portable shell tooling, curl for health probes.

**Spec:** `docs/superpowers/specs/2026-08-28-darkstar-runtime-supervision-design.md`

## Global Constraints

- GitHub remains the source of truth for repository code and deployment configuration.
- Ubuntu is the local build/test/runtime host.
- The Dark Star application container remains non-root.
- Do not add Warlock (formerly Sheriff), Kali, Provider Registry, Power Automate, or cloud orchestration in this historical change.
- Do not add a permanent `while true` loop inside the application container.
- Supervision decisions are mechanical; no AI/model participates.
- Dependency retry cadence is exactly 15 minutes; process/container recovery is immediate and separately bounded.
- Health verification uses the existing `/health` endpoint and the existing `DARKSTAR_HOST`/`DARKSTAR_PORT` contract, defaulting to `0.0.0.0:8080`.

---

### Task 1: Define the Compose runtime contract

**Files:**
- Create: `deploy/docker-compose.yml`
- Create: `deploy/README.md`
- Test: manual Docker/Compose smoke checks on Ubuntu

**Interfaces:**
- Consumes: existing `darkstar:dev` image and server environment variables.
- Produces: a named `darkstar` Compose service with explicit port mapping, healthcheck, non-root execution inherited from the image, and bounded restart policy.

- [ ] **Step 1: Write the failing smoke check**

From `/home/owner/polip-agi`, run:

```bash
cd /home/owner/polip-agi
test -f deploy/docker-compose.yml
```

Expected: FAIL because the file does not exist yet.

- [ ] **Step 2: Add the minimal Compose definition**

Use a Compose service that maps host port `8080` to container port `8080`, passes `DARKSTAR_HOST=0.0.0.0` and `DARKSTAR_PORT=8080`, runs the existing image, declares a healthcheck against `/health`, and uses `restart: unless-stopped` so unexpected exits recover without creating an unconditional infinite restart loop in the application.

- [ ] **Step 3: Add operator documentation**

Document:

```bash
docker compose -f deploy/docker-compose.yml up -d
docker compose -f deploy/docker-compose.yml ps
docker compose -f deploy/docker-compose.yml logs --tail 50
docker compose -f deploy/docker-compose.yml down
```

State that Compose is the authoritative container lifecycle for local Dark Star.

- [ ] **Step 4: Verify Compose parses and starts**

Run:

```bash
cd /home/owner/polip-agi
docker compose -f deploy/docker-compose.yml config
docker compose -f deploy/docker-compose.yml up -d
docker compose -f deploy/docker-compose.yml ps
curl -fsS http://127.0.0.1:8080/health
```

Expected: valid config, container `Up`, and HTTP health response with `status` equal to `ok`.

- [ ] **Step 5: Commit**

```bash
git add deploy/docker-compose.yml deploy/README.md
git commit -m "feat(darkstar): define supervised compose runtime"
```

---

### Task 2: Add host-level systemd lifecycle

**Files:**
- Create: `deploy/systemd/darkstar.service`
- Modify: `deploy/README.md`
- Test: systemd installation, enablement, restart and stop/start checks on Ubuntu

**Interfaces:**
- Consumes: `deploy/docker-compose.yml`.
- Produces: `darkstar.service` that waits for Docker, starts/stops the Compose stack idempotently, and provides one authoritative boot/manual lifecycle path.

- [ ] **Step 1: Write the failing service-install check**

```bash
test -f deploy/systemd/darkstar.service
```

Expected: FAIL before the unit exists.

- [ ] **Step 2: Implement the systemd unit**

The unit must:

```ini
[Unit]
After=docker.service
Requires=docker.service

[Service]
Type=oneshot
RemainAfterExit=yes
WorkingDirectory=/home/owner/polip-agi
ExecStart=/usr/bin/docker compose -f /home/owner/polip-agi/deploy/docker-compose.yml up -d
ExecStop=/usr/bin/docker compose -f /home/owner/polip-agi/deploy/docker-compose.yml down

[Install]
WantedBy=multi-user.target
```

Use the actual host path discovered on Ubuntu. The service must not run Git operations or mutate source code.

- [ ] **Step 3: Install and enable the unit on Ubuntu**

```bash
sudo install -m 0644 deploy/systemd/darkstar.service /etc/systemd/system/darkstar.service
sudo systemctl daemon-reload
sudo systemctl enable darkstar.service
```

- [ ] **Step 4: Verify idempotent start and stop**

```bash
sudo systemctl start darkstar.service
sudo systemctl status darkstar.service --no-pager
docker compose -f /home/owner/polip-agi/deploy/docker-compose.yml ps
sudo systemctl restart darkstar.service
sudo systemctl status darkstar.service --no-pager
sudo systemctl stop darkstar.service
```

Expected: repeated start/restart does not create duplicate services, and stop removes the Compose stack cleanly.

- [ ] **Step 5: Re-enable for boot**

```bash
sudo systemctl enable darkstar.service
```

- [ ] **Step 6: Commit**

```bash
git add deploy/systemd/darkstar.service deploy/README.md
git commit -m "feat(darkstar): supervise compose at host boot"
```

---

### Task 3: Add application-level health supervision

**Files:**
- Create: `deploy/healthcheck.sh`
- Modify: `deploy/docker-compose.yml`
- Test: healthcheck success/failure behavior

**Interfaces:**
- Consumes: HTTP `GET /health` on the configured Dark Star port.
- Produces: deterministic zero/non-zero health result for Compose.

- [ ] **Step 1: Write the failing healthcheck contract test**

Run against a stopped/unavailable service:

```bash
DARKSTAR_HEALTH_URL=http://127.0.0.1:8080/health ./deploy/healthcheck.sh
```

Expected: non-zero exit code when the endpoint is unavailable.

- [ ] **Step 2: Implement the minimal healthcheck script**

The script must fail closed and use a short timeout. It should request `/health` and require a successful HTTP status. It must not restart services itself.

- [ ] **Step 3: Wire the script into Compose**

The Compose healthcheck should run the script with an interval of 30 seconds, timeout 5 seconds, and a small start grace period so startup is observable without creating a tight loop.

- [ ] **Step 4: Verify healthy behavior**

```bash
cd /home/owner/polip-agi
docker compose -f deploy/docker-compose.yml up -d
sleep 3
./deploy/healthcheck.sh
docker inspect --format '{{.State.Health.Status}}' "$(docker compose -f deploy/docker-compose.yml ps -q darkstar)"
```

Expected: script exits 0 and container health is `healthy`.

- [ ] **Step 5: Verify unhealthy behavior**

Stop the container, run the script, and confirm a non-zero result without any restart side effect from the script itself.

- [ ] **Step 6: Commit**

```bash
git add deploy/healthcheck.sh deploy/docker-compose.yml
git commit -m "feat(darkstar): add application health supervision"
```

---

### Task 4: Implement bounded 15-minute dependency retry logic

**Files:**
- Create: `deploy/dependency-retry.sh`
- Create: `deploy/systemd/darkstar-dependency-retry.service`
- Create: `deploy/systemd/darkstar-dependency-retry.timer`
- Modify: `deploy/README.md`
- Test: deterministic retry scheduling with a test endpoint/command and journal inspection

**Interfaces:**
- Consumes: a configured dependency probe command/URL.
- Produces: one retry attempt per timer firing; no in-container permanent loop.

- [ ] **Step 1: Write the failing scheduling assertion**

```bash
test -f deploy/systemd/darkstar-dependency-retry.timer
```

Expected: FAIL before the timer exists.

- [ ] **Step 2: Implement a single-attempt dependency probe**

`dependency-retry.sh` must perform exactly one probe/attempt and exit. It must log success/failure and never sleep for 15 minutes itself.

- [ ] **Step 3: Implement the systemd service**

The service runs one attempt of `dependency-retry.sh`. It must not run continuously and must not restart the application merely because a dependency probe failed.

- [ ] **Step 4: Implement the 15-minute timer**

Use systemd timer semantics to execute the retry service every 15 minutes. Configure it to start after boot and persist schedule state when appropriate.

- [ ] **Step 5: Verify without waiting 15 minutes**

Run the service manually:

```bash
sudo systemctl start darkstar-dependency-retry.service
sudo journalctl -u darkstar-dependency-retry.service -n 20 --no-pager
```

Then inspect the timer:

```bash
systemctl cat darkstar-dependency-retry.timer
systemctl list-timers --all | grep darkstar-dependency
```

Expected: exactly one service attempt per activation and a timer interval of 15 minutes.

- [ ] **Step 6: Commit**

```bash
git add deploy/dependency-retry.sh deploy/systemd/darkstar-dependency-retry.service deploy/systemd/darkstar-dependency-retry.timer deploy/README.md
git commit -m "feat(darkstar): add bounded dependency retry supervisor"
```

---

### Task 5: Final integration verification and documentation

**Files:**
- Modify: `deploy/README.md`
- Modify: `Dockerfile` only if the final runtime contract requires it
- Test: full repository and runtime verification on Ubuntu

**Interfaces:**
- Consumes: all supervision artifacts from Tasks 1–4.
- Produces: reproducible operator runbook and verified host deployment.

- [ ] **Step 1: Verify repository tests and formatting**

```bash
cd /home/owner/polip-agi
source "$HOME/.cargo/env"
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo build --release -p darkstar-server
```

- [ ] **Step 2: Build the image**

```bash
docker build -t darkstar:dev .
```

- [ ] **Step 3: Exercise Compose lifecycle**

```bash
docker compose -f deploy/docker-compose.yml up -d
docker compose -f deploy/docker-compose.yml ps
curl -fsS http://127.0.0.1:8080/health
docker compose -f deploy/docker-compose.yml restart
docker compose -f deploy/docker-compose.yml ps
```

- [ ] **Step 4: Exercise failure recovery**

Terminate the container, wait for the configured restart mechanism, and verify that the service returns to `healthy`.

- [ ] **Step 5: Verify systemd boot contract**

```bash
sudo systemctl enable darkstar.service
sudo systemctl is-enabled darkstar.service
sudo systemctl start darkstar.service
sudo systemctl is-active darkstar.service
```

- [ ] **Step 6: Verify dependency timer contract**

```bash
sudo systemctl enable --now darkstar-dependency-retry.timer
systemctl is-enabled darkstar-dependency-retry.timer
systemctl list-timers --all | grep darkstar-dependency
```

- [ ] **Step 7: Document operational commands**

Document one-command checks for service, container, health, logs, restart and retry state.

- [ ] **Step 8: Commit final integration**

```bash
git add deploy/README.md Dockerfile deploy/
git commit -m "docs(darkstar): document supervised runtime operations"
```
