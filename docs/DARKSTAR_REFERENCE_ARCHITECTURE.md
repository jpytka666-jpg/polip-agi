<!--
THIS IS VERY IMPORTANT!!!
==========================================
AUTHOR: M. SZUL
AI MODEL: GPT-5 Codex
TIMESTAMP: 2026-08-29 11:19:11 Europe/London
REASON FOR CREATION: Ujednolicenie architektury referencyjnej z fazą Ghost Gate i zmianą nazwy z Sheriff Bridge na Warlock Bridge.
==========================================
-->

# Dark Star Reference Architecture v1.1

## 1. System position

AIONS is the complete ecosystem. Dark Star is a component of AIONS: its security and control plane for identity, sessions, capabilities, policy, coordination, orchestration, providers, automation, events, audit and system visibility.

Dark Star does not replace AIONS, WPC, CBMS, Ghost Gate or other existing systems. It adapts and controls them through explicit boundaries.

```text
AIONS
  |
  +-- DARK STAR
       |
       +-- Network / perimeter boundaries
       +-- Identity and Session
       +-- Capability Selector / Gate
       +-- Policy
       +-- Round Table
       +-- Module Registry
       +-- Orchestrator
       +-- Provider / Tentacle layer
       +-- Automation
       +-- Event system
       +-- Audit
       +-- System Graph
       +-- Live Run Graph
```

## 2. Network boundary

The canonical protected path is:

```text
Internet
  -> Warlock Bridge
  -> Kali Bridge
  -> Dark Star
  -> AIONS
```

### Warlock Bridge

Warlock Bridge is the outer perimeter gateway. It owns the external network boundary, VPN, routing, firewall responsibilities and the first ingress/egress decision boundary.

The physical target is a dedicated mini-PC. The initial Azure deployment may represent it as a separate container role.

Warlock replaces the earlier name Sheriff. Existing Sheriff identifiers remain
legacy inputs until the typed Rust migration is completed with backward-compatible
deserialization.

### Kali Bridge

Kali Bridge is an isolated security and testing zone. Security and network testing capabilities run here and are treated as sandboxed capabilities, not as trusted AIONS internals.

The physical target is a separate mini-PC. The initial Azure deployment may represent it as a separate container role.

### Dark Star

Dark Star is the security and control plane. Requests cross Session, Capability, Policy and Orchestrator boundaries before a Provider/Tentacle can act.

### AIONS

AIONS is the protected ecosystem behind the Dark Star boundary. It is not another public perimeter layer.

## 3. Initial deployment model

The first Azure representation uses three isolated roles:

```text
Container 1 -> Warlock Bridge
Container 2 -> Kali Bridge
Container 3 -> Dark Star
```

The logical model is independent of this deployment form. A later physical deployment may use:

```text
Mini-PC #1 -> Warlock Bridge
Mini-PC #2 -> Kali Bridge
Dedicated host -> Dark Star
Protected environment -> AIONS
```

Moving from containers to physical hosts does not change the control-plane contracts.

### Current transitional deployment

Before the dedicated Warlock and Kali hosts exist, Darkstar Ubuntu temporarily
acts as Ghost Gate and native Internet gateway for Windows/private clients:

~~~text
Internet
  -> Darkstar Ubuntu / Ghost Gate
  -> Windows and private clients
  -> AIONS
~~~

NetworkManager, nftables and systemd implement the host networking mechanics.
Darkstar policy and a constrained provider authorize control operations.
Headscale/Headplane provide the planned private mesh/control UI. Tailscale SaaS
is only a temporary recovery route and must not be treated as the final control
plane.

## 4. Layer 1 — Context and Memory

Layer 1 stores durable context required to understand and reconstruct system activity.

It covers sessions, tasks, actions, results, decisions, artifacts, relationships and history.

The layer must reuse existing memory systems such as CBMS rather than recreate their internal engines. Dark Star should request only the context required for the current decision or workflow rather than loading the full history into an agent.

## 5. Layer 2 — Connection and Isolation

Layer 2 establishes authenticated connections, identity, sessions and controlled execution environments.

The intended path is:

```text
Client
  -> Authentication
  -> Identity
  -> Session
  -> Allowed capabilities
  -> Isolated execution boundary
```

Git is a change-control and provenance mechanism. A commit or branch does not by itself grant execution authority.

## 6. Layer 3 — Agent Round Table

Round Table provides controlled collaboration between humans and agents.

Participants have explicit identities and capability scopes. Messages and proposals keep their source information.

The following distinctions are mandatory:

```text
Discussion
  != Proposal
  != Decision
  != Authorization
  != Execution
```

Agent agreement cannot by itself create execution authority. Authorization remains a Dark Star responsibility.

## 7. Layer 4 — Capability and Policy

Dark Star already contains the capability and policy boundary.

The canonical decision path is:

```text
Capability Index
  -> Capability Selector
  -> Capability Gate
  -> Policy
  -> Allow / Deny / NeedsApproval
```

Policy distinguishes at least:

```text
Read
Propose
Execute
Destructive
```

Capability selection identifies a possible target. Policy decides whether that target may be used for the requested operation.

Providers must never use their own judgment to bypass this boundary.

## 8. Module Registry

Plugin and Module are different concepts.

A Plugin describes a mechanism that exposes capabilities through a runtime or integration.

A Module represents an element of the wider AIONS ecosystem that Dark Star may observe, describe and control.

A module descriptor may include:

```text
module_id
name
system
language
version
capabilities
dependencies
state
health
```

Example modules may include WPC, CBMS, Ghost Gate, AIONS Server Wiedzy, Warlock Bridge and Kali Bridge.

## 9. Module lifecycle

The common module lifecycle is:

```text
Offline
  -> Starting
  -> Ready
  -> Running
  -> Stopping
  -> Offline
```

The lifecycle also includes explicit failure and block states:

```text
Failed
Blocked
```

A Blocked module must not be activated merely because an agent or workflow requested it.

## 10. Orchestrator

The Orchestrator plans state changes. It does not become a second security authority and it does not directly spawn arbitrary system processes.

The basic operation is:

```text
Current state + Desired state
           -> Plan
```

Example:

```text
WPC = Offline
Desired = Ready
        -> Start WPC
```

The plan is then passed through the existing policy and provider boundaries.

## 11. Providers and Tentacles

Providers are the controlled integration layer between Dark Star and external systems.

Examples include:

```text
GitHub
GitHub Actions
Azure
Windows
PowerShell
Power Automate
Linux
Docker
Kali
Web
Email
Calendar
Hugging Face
WPC
AIONS
```

A Provider answers the question:

> How do we perform an operation already authorized by Dark Star?

It does not answer:

> Are we allowed to perform it?

The second question belongs to Dark Star Policy and authorization.

## 12. Power Automate position

Power Automate is a Provider/Tentacle of Dark Star, not a separate control plane.

The intended relationship is:

```text
AIONS
  -> Dark Star
       -> Policy
       -> Orchestrator
       -> Power Automate Provider
       -> Power Automate Desktop
       -> Windows action
       -> Event
       -> Audit / Live Run Graph
```

The same pattern applies to PowerShell, GitHub, Azure and other external mechanisms.

## 13. Layer 5 — Automation

Automation makes approved Dark Star workflows persistent in time.

It answers:

```text
When?
How often?
After which event?
For how long?
Under which condition?
```

It does not replace identity, capability or policy.

Automation must support:

- One-shot delayed work.
- Scheduled work.
- Recurring work.
- Event-driven work.
- Long-running workflows.
- Retry and backoff.
- Timeouts and cancellation.
- Pause and resume.
- Conditional branches.
- Parallel tasks.
- Human approval gates.
- Health monitoring and escalation.

Recurring automation must also provide limits such as execution frequency, retry count, concurrency, total runtime and resource use, plus duplicate-event protection and a circuit breaker.

A scheduled workflow never receives extra authority merely because it is scheduled.

## 14. Event system

Dark Star is event-oriented.

Important lifecycle events include:

```text
Requested
Authorized
Started
Running
Completed
Failed
Cancelled
Blocked
ApprovalRequired
Approved
Denied
```

Events should carry enough identity to reconstruct the operation, including event ID, timestamp, principal, session, workflow, module, provider, action and result.

## 15. Provenance

Every meaningful automated operation should be traceable through a stable envelope.

At minimum:

```text
request_id
run_id
principal_id
session_id
workflow_id
repo
branch
commit_sha
module_id
provider_id
action
risk
started_at
finished_at
result
```

The objective is to answer:

```text
Who?
What?
Why?
Which module?
Which session?
Which provider?
Which commit?
Which run?
What happened?
```

## 16. System Graph and Live Run Graph

These graphs serve different purposes.

### System Graph

System Graph answers:

> What exists?

It represents repositories, files, modules, runtimes, providers, relationships and provenance.

### Live Run Graph

Live Run Graph answers:

> What is happening now?

Example:

```text
Human
  -> Request
  -> Capability Gate
  -> Policy
  -> Orchestrator
  -> Azure Provider
  -> Azure VM
  -> Test
  -> Result
```

System Graph is structural. Live Run Graph is operational.

## 17. Audit

Audit records enough information to reconstruct control decisions and external actions without unnecessarily copying every large external payload.

Audit should preserve provenance, authorization results, important state transitions, provider identity and outcomes.

Large payloads may remain in their source system with a reference stored by Dark Star.

## 18. Error model

Failures must remain distinguishable by layer.

```text
Connection failure
  -> Session failure
  -> Capability failure
  -> Policy denial
  -> Orchestration failure
  -> Provider unavailable
  -> External execution failure
```

A generic `failed=true` result is insufficient because the recovery action depends on where the failure occurred.

## 19. Core execution path

The canonical Dark Star path is:

```text
Human / Agent
      |
      v
Connection
      |
      v
Session
      |
      v
Round Table / Request
      |
      v
Capability Selector
      |
      v
Capability Gate
      |
      v
Policy
   |       |
 Deny   NeedsApproval
   |       |
   |    Approval
   |       |
   +--- Allow
          |
          v
     Orchestrator
          |
          v
       Provider
          |
          v
 External System / Module
          |
          v
        Result
       /      \
    Event     Audit
      |
      v
Live Run Graph
      |
      v
Context / Memory
```

This sequence keeps planning, authorization and execution as separate responsibilities.

## 20. Security boundary

The following rule is fundamental:

```text
Agent proposes.
Round Table discusses.
Policy authorizes.
Orchestrator plans.
Provider executes.
Event records.
Audit reconstructs.
```

No individual external adapter may become a hidden bypass around Dark Star policy.

## 21. What Dark Star is not

Dark Star is not:

```text
AIONS itself
WPC
CBMS
Ghost Gate
Kali
VPN appliance
GitHub
Azure
PowerShell
Power Automate
Operating system
AI model
```

Dark Star is the controlled security and coordination layer between intelligent requests and the systems that perform real work.

## 22. Technology boundary

Dark Star core remains Rust-first.

Rust is used for the trusted control plane, including sessions, capabilities, policy, registries, orchestration, event handling, audit interfaces and automation control.

External workers may use PowerShell, Python, TypeScript or provider-native tooling where that is the appropriate integration technology. Those workers remain subordinate to Dark Star policy and cannot bypass it.

## 23. Non-duplication rule

Dark Star must adapt and orchestrate existing systems rather than recreate their internal engines.

In particular:

```text
AIONS -> existing ecosystem
WPC   -> existing computational engine
CBMS  -> existing memory mechanism
Ghost Gate -> existing network security boundary
GitHub -> existing source-control system
Azure -> existing cloud infrastructure
```

Dark Star provides the control boundary between them.

## 24. Architectural goal

Dark Star should become the place where the complete execution chain remains understandable and controllable.

A request should be traceable from its origin, through session, capability, policy and orchestration, to the provider and external action, and back through result, event, audit and live graph.

The target is not a giant autonomous process.

The target is a small number of explicit, testable and observable boundaries that together allow AIONS to act safely across many systems.
