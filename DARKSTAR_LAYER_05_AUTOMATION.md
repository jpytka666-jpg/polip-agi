# Dark Star MCP — Layer 05: Automation

## Purpose

The Automation layer is the long-running, event-driven and scheduled execution layer of Dark Star MCP.

It exists so that agents do not have to remain continuously connected or actively supervise every process. An agent can define an automation, schedule it, register event triggers, monitor it, pause it, resume it, or terminate it. Dark Star remains responsible for policy, authorization, isolation, auditability and state.

Automation is therefore not another intelligence layer. It is the mechanism that turns approved decisions and workflows into controlled processes that can continue operating over time.

## Scope

Automation is intended to cover:

- Scheduled jobs and recurring tasks.
- One-shot delayed tasks.
- Event-driven workflows.
- Long-running workflows.
- Retry and backoff policies.
- Timeouts and cancellation.
- Dependencies between tasks.
- Conditional branches.
- Parallel task execution.
- Human approval gates.
- Agent approval gates explicitly delegated by the owner.
- Monitoring and health checks.
- Notifications and escalation.
- Periodic mailbox, calendar, repository and web checks.
- CI/CD and development automation.
- Cloud resource automation.
- Infrastructure automation.
- Cross-provider workflows.
- Azure automation and orchestration.
- AWS automation and orchestration.
- Other cloud or SaaS automation where a supported provider exists.
- Local-machine automation through the trusted local Dark Star/Ionis connection.

## Core principle

Tentacles perform actions.

The Round Table decides what should happen.

Automation decides when, under which trigger, and according to which workflow an already-authorized process should continue.

Automation must never become an unrestricted back door around the security model of Layers 1–4.

A scheduled workflow does not automatically gain more privileges merely because it is scheduled. Every execution remains bound to an identity, session/workflow identity, policy, scope and authorization state.

## Event-driven architecture

Dark Star Automation should be event-driven rather than implemented as a collection of permanent polling loops.

An event may originate from:

- GitHub.
- An incoming MCP/API request.
- A webhook.
- A mailbox event.
- A calendar event.
- A cloud resource event.
- A CI/CD event.
- A local trusted peer.
- A timer or scheduler.
- Completion or failure of another workflow.
- A state transition inside Dark Star itself.

Events enter an internal event bus/router. Rules determine whether an event starts, advances, pauses, cancels or otherwise affects an automation.

This follows established cloud patterns. Azure Event Grid can route events to Functions, Logic Apps, Automation runbooks and webhooks, while AWS provides EventBridge and EventBridge Scheduler for event routing and scheduled execution. AWS Step Functions provides durable workflow orchestration. These services should be treated as external execution capabilities/providers that Dark Star can control rather than as replacements for Dark Star's own security and policy model.

## Workflow model

An automation should be represented as a durable state machine/workflow rather than as an opaque script.

A workflow should have at minimum:

- Unique workflow ID.
- Owner identity.
- Creation timestamp.
- Version.
- Trigger definition.
- Input schema.
- Allowed tools/providers.
- Execution policy.
- Current state.
- Retry policy.
- Timeout policy.
- Approval requirements.
- Audit trail.
- Result and error state.
- Cancellation state.

The workflow definition should be versioned. A running workflow must not silently change behaviour because its source definition was modified later.

## Loop and recurring execution

Recurring automation is a first-class capability.

For example, an automation may periodically inspect a mailbox, check a GitHub repository, monitor a cloud resource, collect information, or trigger an agent workflow.

A recurring workflow should have explicit limits and lifecycle controls. It must be possible to pause, disable, terminate or revoke it without destroying historical records.

The system should also prevent accidental runaway loops, including:

- Maximum execution frequency.
- Maximum concurrent instances.
- Maximum total runtime.
- Maximum retry count.
- Maximum resource consumption.
- Circuit breakers.
- Duplicate-event suppression/idempotency.

## Human approval

Automation must support approval gates.

A workflow can reach a state where it has prepared an action but cannot continue until the owner approves it.

Examples include:

- Publishing code.
- Merging a Pull Request.
- Deleting resources.
- Sending sensitive communications.
- Spending money.
- Changing infrastructure.
- Granting permissions.
- Executing destructive operations.

Approval should be represented as an explicit state transition and recorded in the audit history.

## Delegated agent approval

The owner may explicitly delegate approval authority to a trusted agent for narrowly defined operations.

This delegation must be scoped and revocable. An agent cannot delegate its own authority further unless the owner explicitly permits delegation.

A workflow must never interpret general agent participation in the Round Table as blanket permission to execute arbitrary automation.

## Cloud providers

Dark Star should provide a provider abstraction for cloud automation.

Azure is the first-class environment for the initial deployment because Dark Star is being developed around GitHub and the available Azure student resources provide a practical execution environment.

Azure capabilities may include Functions, Durable Functions, Event Grid, Logic Apps, Automation and other suitable services.

AWS should be supported through a separate provider layer. Relevant AWS capabilities include Lambda, Step Functions, EventBridge and EventBridge Scheduler.

The architecture should remain cloud-agnostic at the Dark Star policy layer. A workflow should express intent and policy independently from the provider-specific implementation whenever practical.

## Execution strategy

Dark Star should support several execution classes:

1. In-process lightweight jobs for safe, short operations.
2. Isolated workers for untrusted or heavier operations.
3. Cloud-native execution through provider services.
4. Local execution through the authenticated trusted local peer.
5. Long-running durable workflows.

The execution class is selected by policy and risk, not simply by convenience.

## Security boundary

Automation is a high-risk layer because it introduces persistence.

A one-time mistake is dangerous. A mistake scheduled every five minutes can become catastrophic.

Therefore every automation must be:

- Authenticated.
- Authorized.
- Scoped.
- Audited.
- Rate-limited where appropriate.
- Cancellable.
- Observable.
- Idempotent where possible.
- Protected against replay and duplicate events.
- Separated from untrusted inputs.

Credentials should not be embedded in workflow definitions. Provider credentials should be referenced through the Dark Star credential/identity subsystem.

Where the cloud provider supports managed identities or equivalent workload identities, Dark Star should prefer them over long-lived static secrets.

## Observability

Every automation execution should produce structured lifecycle information.

The system should be able to answer:

- Who created the automation?
- Who authorized it?
- What triggered it?
- Which version of the workflow ran?
- Which tools were called?
- Which provider executed each action?
- What data entered the workflow?
- What decisions were made?
- Where did it fail?
- What changed?
- What remains pending?
- Who or what approved the next step?

Automation history should connect naturally to the Context/Memory layer and Git-based project history without treating Git as the runtime database.

## Git and automation

GitHub remains the source-control and development control plane.

Automation may create branches, commits, Pull Requests, run CI, inspect Actions and prepare changes. Production or protected changes remain subject to the authorization and approval model.

Git is therefore used for provenance and controlled change management, not as a substitute for the runtime workflow engine.

## Language strategy

The Dark Star control plane remains Rust-first.

Rust should implement the core automation scheduler/orchestrator, workflow state model, policy enforcement, event routing, concurrency control, cancellation, persistence interfaces, audit interfaces and provider boundaries.

Provider-specific workers may use another language when that provider's SDK or ecosystem makes it materially better. Such workers remain subordinate to Dark Star's Rust control plane and cannot bypass its authorization model.

Infrastructure definitions and deployment automation may naturally use provider-native declarative formats, Azure CLI, PowerShell, Terraform/OpenTofu or other appropriate tooling. These are deployment artifacts, not replacements for the Dark Star core language.

## Relationship to the other layers

Layer 1 — Context & Memory stores durable context, knowledge, session history and workflow history.

Layer 2 — Connection & Isolation authenticates clients, establishes sessions and isolates execution environments.

Layer 3 — Agent Round Table provides the collaborative intelligence layer where agents exchange information, divide work and form decisions.

Layer 4 — Tentacles provides controlled tools and external-system capabilities.

Layer 5 — Automation turns approved workflows into persistent, scheduled, event-driven and independently progressing processes.

The layers must remain separated. Automation can invoke Tentacles, but it cannot bypass Layer 2 security. Automation can receive decisions from the Round Table, but it does not become an unrestricted agent. Automation can write execution history to Context/Memory, but it does not own the memory layer.

## Architectural goal

The final Automation layer should make Dark Star capable of continuing useful work after an agent disconnects, while remaining completely observable and controllable.

An agent should be able to say, in effect:

"Monitor this repository. If condition X occurs, investigate it. If the investigation produces a safe result, perform action Y. If the action requires approval, stop and wait. If it fails, retry according to policy. If it remains unresolved, notify me. Keep the complete history."

Dark Star Automation is the machinery that makes such a workflow persistent, durable and controlled.

It is the layer that turns Dark Star from a server that responds to requests into an infrastructure platform that can continue operating over time.