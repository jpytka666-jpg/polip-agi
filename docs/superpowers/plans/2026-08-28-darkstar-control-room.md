# Darkstar Control Room Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Turn the existing Darkstar browser graph into a real, polished Control Room that shows the system and live runs as n8n-inspired nodes and routes authorized Start/Stop/Restart requests through Darkstar policy rather than directly to infrastructure.

**Architecture:** Evolve the existing embedded `system_graph_view.rs` instead of adding a second frontend stack. Keep the server authoritative: browser -> authenticated session -> capability check -> policy -> orchestrator -> provider -> event/audit -> graph update. Use the current JSON System Graph route and authenticated SSE stream as the base transport, adding a typed module-control HTTP boundary for real actions.

**Tech Stack:** Rust 2024, Axum 0.8, Tokio, serde/serde_json, UUID, embedded HTML/CSS/vanilla JavaScript, SVG, existing Darkstar core module/session/policy/provider/orchestrator contracts.

**Spec:** `docs/superpowers/specs/2026-08-28-darkstar-control-room-design.md`

## Global Constraints

- Rust remains the trusted control-plane language; browser JavaScript is presentation only.
- Do not introduce React/Node build tooling for v1.
- Browser actions are requests only; the browser never calls systemd, Docker, Windows, Kali or cloud APIs directly.
- Server-side authorization remains authoritative for every state-changing module action.
- Protected graph/control endpoints require authentication; the public WebView shell may be served without exposing protected data.
- Remote access is HTTPS-only at deployment time; never publish the raw Darkstar service as an unprotected public port.
- Keep the existing `/system-graph`, `/v1/system-graph`, `/v1/runs/start`, and `/v1/runs/{run_id}/events` routes compatible where practical.
- Preserve the existing workspace test suite as a required regression gate.
- Keep permanent changes on GitHub; no destructive local actions.

---

### Task 1: Baseline and action contract

**Files:**
- Modify: `crates/darkstar-core/src/module_state.rs`
- Modify: `crates/darkstar-core/src/module_provider.rs`
- Modify: `crates/darkstar-core/src/module_execution.rs`
- Modify: `crates/darkstar-core/src/orchestrator.rs`
- Test: `crates/darkstar-core/src/module_state.rs`
- Test: `crates/darkstar-core/src/module_provider.rs`
- Test: `crates/darkstar-core/src/module_execution.rs`
- Test: `crates/darkstar-core/src/orchestrator.rs`

**Interfaces:**
- Consumes existing `ModuleCommandRequest`, `ModuleState`, `AuthorizedModuleCommand`, `ModuleProvider`, and `OrchestrationPlan` contracts.
- Produces a stable server-facing control contract for `Start`, `Stop`, and `Restart`, including explicit result/error states that the HTTP layer can serialize.

- [ ] **Step 1: Establish current baseline**

Run from Ubuntu:

```bash
cd /home/owner/polip-agi && \
DARKSTAR_API_TOKEN=secret cargo test --workspace
```

Expected: all existing tests pass before the Control Room changes begin.

- [ ] **Step 2: Write failing core tests for control-result semantics**

Add tests covering:

```rust
assert_eq!(ModuleCommand::Start.capability(), "module.start");
assert_eq!(ModuleCommand::Stop.capability(), "module.stop");
assert_eq!(ModuleCommand::Restart.capability(), "module.restart");
```

and provider outcomes for authorized Start/Stop/Restart requests plus denied and approval-required paths.

- [ ] **Step 3: Implement the smallest typed contract**

Keep all authorization in the existing policy path. Expose only the typed request/result data needed by the server. Do not add infrastructure calls here.

- [ ] **Step 4: Run focused core tests**

```bash
cd /home/owner/polip-agi && cargo test -p darkstar-core
```

Expected: PASS.

- [ ] **Step 5: Run workspace regression**

```bash
cd /home/owner/polip-agi && DARKSTAR_API_TOKEN=secret cargo test --workspace
```

Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add crates/darkstar-core/src/module_state.rs crates/darkstar-core/src/module_provider.rs crates/darkstar-core/src/module_execution.rs crates/darkstar-core/src/orchestrator.rs && \
git commit -m "feat(darkstar): define module control contract"
```

---

### Task 2: Authenticated module-control HTTP boundary

**Files:**
- Modify: `crates/darkstar-server/src/http.rs`
- Create: `crates/darkstar-server/src/module_control_http.rs`
- Test: `crates/darkstar-server/tests/memory_http.rs`

**Interfaces:**
- Consumes `AppState`, session authentication, core module command contracts and the existing router.
- Produces `POST /v1/modules/{module_id}/actions` with a JSON action request and a JSON action result, while preserving the existing auth style.

- [ ] **Step 1: Write route tests first**

Tests must cover:

```text
POST without Authorization -> 401
POST with invalid token -> 401
POST with valid token but missing module capability -> 403/deny
POST with valid token and capability -> accepted request
```

Use an explicit request body such as:

```json
{"command":"start","reason":"operator requested start"}
```

- [ ] **Step 2: Add the dedicated HTTP module**

Keep request parsing and response serialization in `module_control_http.rs` so `http.rs` does not grow another large responsibility.

- [ ] **Step 3: Wire the route**

Register:

```text
POST /v1/modules/{module_id}/actions
```

and reuse the existing `authenticated(&state, &headers)` behavior rather than creating a second authentication mechanism.

- [ ] **Step 4: Map authorization outcomes cleanly**

Use these semantics:

```text
missing/invalid auth -> 401
missing session capability / policy deny -> 403
approval required -> 409 with explicit approval state
provider success -> 200
provider unavailable -> 503
execution failure -> 502
```

Do not leak provider credentials or internal command details into the browser response.

- [ ] **Step 5: Run focused HTTP tests**

```bash
cd /home/owner/polip-agi && DARKSTAR_API_TOKEN=secret cargo test -p darkstar-server --test memory_http
```

Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add crates/darkstar-server/src/http.rs crates/darkstar-server/src/module_control_http.rs crates/darkstar-server/tests/memory_http.rs && \
git commit -m "feat(darkstar): expose authorized module control API"
```

---

### Task 3: Replace debug-style graph rendering with Control Room visual system

**Files:**
- Modify: `crates/darkstar-server/src/system_graph_view.rs`
- Test: `crates/darkstar-server/tests/memory_http.rs`

**Interfaces:**
- Consumes the existing `/v1/system-graph` snapshot and `/v1/runs/{run_id}/events` SSE stream.
- Produces a single browser workspace with System Graph and Live Run Graph, compact node cards, inspector, bottom timeline, search, zoom/pan, fit/reset, and explicit action controls.

- [ ] **Step 1: Add browser-level structural tests**

Verify the WebView source contains stable UI anchors:

```text
Darkstar Control Room
System Graph
Live Run Graph
Inspector
Live Timeline
Start
Stop
Restart
```

- [ ] **Step 2: Build the layout**

Use four regions from the spec:

```text
Top command bar
Central graph canvas
Right inspector
Bottom event timeline
```

Use CSS grid/flex only; no new frontend build system.

- [ ] **Step 3: Rework node cards**

Each node card must show:

```text
icon/type
name
state
optional health/provider badges
```

State styling must distinguish Ready, Running, Success, Blocked, Failed and waiting states without depending on color alone.

- [ ] **Step 4: Improve graph manipulation**

Implement:

```text
pan
zoom
fit to view
reset view
stable node positions
search filter
node selection
```

- [ ] **Step 5: Build the inspector**

Selecting a node must show identity, type, state, health, version, dependencies, capabilities, related resource, provenance, and last important event.

- [ ] **Step 6: Build the bottom timeline**

Render events in chronological order with run ID, node ID, event type, state, message, and timestamp where available. Allow collapsing the timeline.

- [ ] **Step 7: Keep actions visibly separated**

Show Start/Stop/Restart only when the authenticated server response says the session may request them. Never infer permission from graph appearance.

- [ ] **Step 8: Run focused tests**

```bash
cd /home/owner/polip-agi && DARKSTAR_API_TOKEN=secret cargo test -p darkstar-server --test memory_http
```

Expected: PASS.

- [ ] **Step 9: Commit**

```bash
git add crates/darkstar-server/src/system_graph_view.rs crates/darkstar-server/tests/memory_http.rs && \
git commit -m "feat(darkstar): build Control Room graph interface"
```

---

### Task 4: Connect browser actions to real module execution

**Files:**
- Modify: `crates/darkstar-server/src/module_control_http.rs`
- Modify: `crates/darkstar-server/src/http.rs`
- Modify: `crates/darkstar-server/src/run_stream.rs`
- Modify: `crates/darkstar-core/src/orchestrator.rs`
- Modify: `crates/darkstar-core/src/module_execution.rs`
- Test: `crates/darkstar-server/tests/memory_http.rs`

**Interfaces:**
- Consumes authenticated browser action requests and existing module/provider contracts.
- Produces one traceable execution sequence: request -> authorization -> plan -> provider -> result -> event.

- [ ] **Step 1: Write the end-to-end action test**

The test must submit:

```http
POST /v1/modules/wpc-engine/actions
Authorization: Bearer secret
Content-Type: application/json
```

with:

```json
{"command":"start","reason":"operator test"}
```

and assert the response identifies the request, module, command and resulting state.

- [ ] **Step 2: Wire the authenticated request into the existing execution boundary**

Call the existing core authorization/execution path rather than duplicating capability checks inside the browser handler.

- [ ] **Step 3: Emit run events for state transitions**

Emit at least:

```text
Requested
Authorized or Denied
Started
Completed or Failed
```

with request ID, run ID, session ID, module ID, provider ID where known, and resulting state.

- [ ] **Step 4: Make the browser subscribe to the real run ID**

After a successful action request, use the returned run ID to subscribe to the existing authenticated SSE stream and drive the Live Run Graph.

- [ ] **Step 5: Test deny and approval paths**

Verify the browser sees an explicit blocked/approval state and that no provider call occurs when authorization fails.

- [ ] **Step 6: Run all server tests**

```bash
cd /home/owner/polip-agi && DARKSTAR_API_TOKEN=secret cargo test -p darkstar-server
```

Expected: PASS.

- [ ] **Step 7: Commit**

```bash
git add crates/darkstar-server/src/module_control_http.rs crates/darkstar-server/src/http.rs crates/darkstar-server/src/run_stream.rs crates/darkstar-core/src/orchestrator.rs crates/darkstar-core/src/module_execution.rs crates/darkstar-server/tests/memory_http.rs && \
git commit -m "feat(darkstar): stream real module execution into Control Room"
```

---

### Task 5: Browser smoke test on the real Ubuntu Darkstar service

**Files:**
- Modify: `deploy/README.md` only if operational launch instructions are missing
- Test: real running service at `http://127.0.0.1:18080` on Ubuntu

**Interfaces:**
- Consumes the deployed Darkstar service and Chromium.
- Produces verified evidence that the Control Room opens, authenticates, displays the graph and observes a real run.

- [ ] **Step 1: Verify service health**

```bash
curl -fsS http://127.0.0.1:18080/health && echo
```

Expected: Darkstar status `ok`.

- [ ] **Step 2: Verify route availability**

```bash
curl -I http://127.0.0.1:18080/system-graph
```

Expected: HTTP success for the WebView shell.

- [ ] **Step 3: Open Chromium on Ubuntu**

Open the local Control Room address and authenticate with the configured token through the UI.

- [ ] **Step 4: Exercise the graph**

Verify:

```text
System Graph visible
node click opens inspector
search filters nodes
zoom/pan work
Live Run Graph switches correctly
```

- [ ] **Step 5: Exercise one real action**

Use a safe module/provider path first. Start or restart a module only when its capability and policy permit the operation, then verify the timeline and graph reflect the actual result.

- [ ] **Step 6: Commit any operational documentation change**

```bash
git add deploy/README.md && git commit -m "docs(darkstar): document Control Room launch"
```

Skip this commit when no documentation change is required.

---

### Task 6: Remote HTTPS access boundary

**Files:**
- Modify: `deploy/docker-compose.yml`
- Modify: `deploy/README.md`
- Modify: `deploy/systemd/darkstar.service`
- Create only if justified by existing deployment patterns: `deploy/reverse-proxy/` configuration

**Interfaces:**
- Consumes the existing Darkstar HTTP service on the internal port.
- Produces an HTTPS entry path where a remote browser can load the same Control Room without exposing an unauthenticated raw Darkstar port.

- [ ] **Step 1: Inspect current deployment boundary**

Verify which component is responsible for network ingress before adding a reverse proxy. Do not duplicate an existing perimeter service.

- [ ] **Step 2: Write the deployment verification checks**

Required checks:

```text
raw Darkstar port is not publicly exposed
HTTPS endpoint exists
WebView loads remotely
protected graph call without auth is rejected
protected graph call with auth succeeds
```

- [ ] **Step 3: Add the smallest compatible HTTPS routing configuration**

Use the existing perimeter architecture when available. Do not put API tokens in URLs or shell scripts.

- [ ] **Step 4: Run deployment checks**

```bash
cd /home/owner/polip-agi && \
cargo fmt --all -- --check && \
DARKSTAR_API_TOKEN=secret cargo test --workspace
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add deploy/docker-compose.yml deploy/README.md deploy/systemd/darkstar.service deploy/reverse-proxy && \
git commit -m "feat(darkstar): expose Control Room through HTTPS"
```

Only include files that actually changed.

---

### Task 7: Final visual, security and regression gate

**Files:**
- Modify only files required by failing checks from Tasks 1-6
- Test: all workspace tests and real browser smoke test

**Interfaces:**
- Consumes the completed Control Room and deployment configuration.
- Produces the final evidence package in Git history: tests green, remote browser access verified, module action traceable, no direct infrastructure access from browser.

- [ ] **Step 1: Run formatting and static checks**

```bash
cd /home/owner/polip-agi && cargo fmt --all -- --check && cargo check --workspace
```

Expected: PASS.

- [ ] **Step 2: Run complete tests**

```bash
cd /home/owner/polip-agi && DARKSTAR_API_TOKEN=secret cargo test --workspace
```

Expected: PASS with zero failed tests.

- [ ] **Step 3: Verify protected graph behavior**

```bash
curl -s -o /dev/null -w '%{http_code}\n' http://127.0.0.1:18080/v1/system-graph
```

Expected: `401` without authorization.

- [ ] **Step 4: Verify authenticated graph behavior**

```bash
curl -fsS -H 'Authorization: Bearer secret' http://127.0.0.1:18080/v1/system-graph >/dev/null && echo GRAPH_OK
```

Expected: `GRAPH_OK`.

- [ ] **Step 5: Verify Git cleanliness**

```bash
cd /home/owner/polip-agi && git status --short --branch && git diff --check
```

Expected: no unintended changes.

- [ ] **Step 6: Push the completed branch**

```bash
git push origin feat/darkstar-module-control
```

- [ ] **Step 7: Final acceptance check**

The implementation is accepted only when the operator can open the WebView locally and remotely, authenticate, see the real graph, watch real execution, inspect node provenance, request an authorized Start/Stop/Restart action, and see the resulting state in the same workspace.
