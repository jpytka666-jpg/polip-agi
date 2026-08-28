# Darkstar Control Room v1 — Browser System Graph and Live Run Graph

## Status

Approved architectural design for implementation planning.

## 1. Goal

Turn Darkstar's existing browser System Graph into a real human-facing Control Room: a polished n8n-inspired visual workspace for inspecting the Darkstar/AIONS system, watching live execution, and requesting authorized module actions.

The interface is a web application served by Darkstar. Chromium on the dedicated Ubuntu Darkstar host is one client, not the only client. The same interface must be reachable remotely through an HTTPS address when the deployment exposes Darkstar to the network.

## 2. User experience

The primary experience is a large visual canvas in the centre of the screen. Nodes represent modules, repositories, services, providers, workflows and other important Darkstar objects. Edges represent real relationships or execution transitions.

The visual language is inspired by n8n's workflow canvas, but the goal is a cleaner and more operationally informative experience rather than a visual copy.

The UI must be immediately understandable without reading documentation:

- healthy/ready objects are visually calm;
- running objects are visually active;
- successful steps show clear success state;
- blocked or failed objects are visibly distinct;
- the currently executing path is highlighted;
- selecting a node opens its inspector;
- dangerous or mutating actions are visibly separated from read-only inspection.

The interface is desktop-first but responsive enough for a modern browser window on a laptop or tablet.

## 3. Main layout

The first version uses four persistent regions:

1. Top command bar.
2. Central graph canvas.
3. Right inspector panel.
4. Bottom live event timeline/log.

Optional collapsed left navigation may be added for switching views without consuming significant canvas space.

### Top command bar

The command bar contains:

- Darkstar identity/status;
- current view: System Graph or Live Run Graph;
- connection/authentication state;
- search;
- refresh;
- run controls;
- fit-to-screen and reset-view controls.

The command bar must not contain secrets in visible text after authentication.

### Central graph canvas

The canvas supports:

- pan;
- zoom;
- node selection;
- edge selection where useful;
- fit-to-screen;
- minimap in a later iteration if needed;
- animated live path highlighting;
- stable layout between refreshes whenever the underlying graph has not materially changed.

Nodes are represented as compact cards rather than bare circles. Each card should show at least a name, type/icon, and current state. Additional badges may show health, provider, or capability information.

### Right inspector

Selecting a node opens a structured inspector containing:

- identity;
- kind/type;
- current state;
- health;
- version where known;
- dependencies;
- capabilities;
- related repository/resource;
- last important event;
- provenance identifiers;
- permitted actions.

Mutating controls such as Start, Stop and Restart are only shown when Darkstar reports that the current session is permitted to request them.

### Bottom timeline

The bottom panel shows a chronological stream of live events.

Each event should display enough information to answer:

- what happened;
- where it happened;
- when it happened;
- which run it belongs to;
- which node was involved;
- the resulting state;
- whether the step succeeded, failed, was blocked or is waiting.

The timeline can be collapsed to maximize canvas space.

## 4. Two related views

### System Graph

Answers: "What exists?"

This view is structural. It displays the current Darkstar/AIONS topology and relationships.

### Live Run Graph

Answers: "What is happening now?"

This view uses the same underlying graph but overlays execution state. The active node and active edges are highlighted as live events arrive.

The two views must share node identity so the operator can move from a live event directly to the corresponding system object.

## 5. Node actions

Node actions use the existing Darkstar authorization path.

A Start/Stop/Restart request from the browser is only a request. The browser never talks directly to systemd, Docker, Windows, Kali or a cloud provider.

The required path is:

Browser -> authenticated session -> capability check -> policy -> orchestrator -> provider -> external/module operation -> event/audit -> graph update.

The UI must reflect authorization state explicitly. A denied action must produce a clear reason without exposing unnecessary internal details.

## 6. Remote browser access

The WebView is served by Darkstar and should remain a normal HTTP application from the browser's perspective.

For remote use, deployment should expose it through HTTPS. The preferred production shape is:

Internet -> perimeter/reverse proxy -> Darkstar WebView/API

The browser must authenticate before reading protected graph data or requesting state-changing actions.

API credentials must not be placed in query strings or embedded in hyperlinks. The initial v1 client may accept a token in a password field and send it in the Authorization header. A later identity layer may replace this with a browser session/cookie flow without changing the graph API.

The public deployment must not expose the raw Darkstar service over an unprotected public HTTP port.

## 7. Transport and live updates

The existing authenticated JSON System Graph route remains the structural data source.

The existing authenticated Server-Sent Events stream remains the first live transport for run events.

The browser should:

1. fetch the current graph;
2. render stable node identities;
3. start or attach to a run when authorized;
4. subscribe to its live event stream;
5. update node/edge state incrementally;
6. append the event to the timeline;
7. keep the inspector synchronized with the selected object.

WebSocket is not required for v1 because the current SSE path already provides the needed one-way live execution stream.

## 8. Visual quality

The interface should feel like a premium operator console rather than a developer debug page.

Design targets:

- strong visual hierarchy;
- restrained dark workspace background;
- high-quality typography;
- clear state indicators;
- subtle animation only where it conveys live state;
- no excessive decoration that competes with the graph;
- strong contrast and readable labels;
- consistent spacing and iconography;
- clear hover/focus/selected states;
- keyboard-friendly basic operations.

The visual design should be inspired by the clarity and direct manipulation of workflow editors such as n8n, while remaining distinctly Darkstar.

## 9. Safety and operator control

The UI is an observation and control surface, not a second policy engine.

The browser must never decide whether an operation is allowed based on visual state alone. The server is authoritative.

Mutating operations must be traceable to the authenticated session and request identity.

The UI should distinguish:

- inspect;
- propose/request;
- authorized;
- executing;
- completed;
- blocked;
- failed.

This preserves the architectural distinction between human interaction, authorization and execution.

## 10. Relationship to existing code

The existing `system_graph_view.rs` already provides a self-contained browser implementation with graph loading, demo-run controls, SVG rendering, search, node inspection, zoom/pan and authenticated SSE consumption.

The implementation should evolve that existing view instead of introducing a second unrelated frontend stack.

The existing routes should remain compatible where practical. A dedicated `/control-room` route may be added as the preferred entry point, with `/system-graph` retained as a compatibility alias.

The current server already exposes:

- `/system-graph`;
- `/v1/system-graph`;
- `/v1/runs/start`;
- `/v1/runs/{run_id}/events`;
- session and memory routes.

The next implementation stage should add real module-control requests through the existing authorization/provider/orchestrator boundaries rather than wiring browser buttons directly to infrastructure.

## 11. Technology decision

Primary implementation remains Rust 2024 with the existing Axum/Tokio server.

The WebView remains embedded HTML/CSS/JavaScript in the first implementation.

No separate React/Node frontend build is required for v1. A browser-native client reduces deployment complexity and keeps the Control Room close to the Darkstar service boundary.

If the UI later becomes large enough to justify a dedicated frontend build, that decision should be made from measured complexity rather than assumed in advance.

## 12. Testing

Implementation must include:

- route-level tests for protected and public WebView/API routes;
- authorization tests for node actions;
- live-event rendering tests for state transitions;
- graph/node selection tests;
- failure/blocked-state rendering tests;
- browser smoke testing against the actual Darkstar service;
- verification that protected graph data is not exposed without authentication;
- verification that the browser never requires direct access to systemd/Docker/provider credentials.

The existing workspace test suite must remain green.

## 13. Success criteria

Control Room v1 is considered complete only when an operator can:

1. open the Darkstar WebView in Chromium on the Ubuntu host;
2. open the same interface through the configured HTTPS address from another machine;
3. authenticate successfully;
4. see the real System Graph;
5. switch to Live Run Graph;
6. observe a real run progressing through real nodes/events;
7. select a node and inspect its state and provenance;
8. request an allowed Start/Stop/Restart action from the node inspector;
9. observe authorization, execution and resulting state in the same graph;
10. see blocked and failed operations clearly;
11. retain a readable live event history for the run.

## 14. Non-goals for v1

The first implementation does not attempt to build:

- a complete n8n workflow editor;
- arbitrary workflow authoring from the browser;
- a browser-based terminal;
- direct Docker/systemd shell access;
- a second authorization system;
- a full cloud-provider dashboard;
- a permanent database replacement for CBMS.

These may be future layers or views, but they are not prerequisites for the first real Control Room.

## 15. Architectural conclusion

Darkstar Control Room is the human visual surface over the existing control plane.

It does not become the brain, the policy engine or the execution engine.

It makes the existing architecture understandable and controllable by turning the system and its live execution into a direct visual model:

Human -> Session -> Capability -> Policy -> Orchestrator -> Provider -> Module
                                           |
                                           +-> Event -> Live Run Graph -> Memory/Audit

The objective is simple: when something happens inside Darkstar, the operator should be able to see it, understand it, inspect it and, when authorized, control it from the same browser workspace.