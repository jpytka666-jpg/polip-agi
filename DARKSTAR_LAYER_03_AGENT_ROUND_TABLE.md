# Dark Star MCP — Layer 03: Agent Execution / Round Table

## Status

Architectural draft. This document records the intended role of Layer 03 before implementation.

## 1. Purpose

Layer 03 is the execution and coordination layer of Dark Star MCP.

Its purpose is not to create another AI model. It is the controlled environment in which one or more existing AI agents/models can participate in the same task, exchange context, propose actions, inspect results, divide work, challenge one another, and converge on a decision or execution plan.

The working name for this layer is **Round Table**.

The Round Table is conceptually a shared meeting space for agents. Dark Star remains the host and authority around the meeting; individual models remain replaceable participants.

## 2. Core concept

Multiple agents may participate simultaneously.

Possible participants include:

- a cloud model such as Claude;
- a local Ionis agent running on the user's computer;
- an agent operating from a phone or another remote client;
- ChatGPT or another external model/client when an integration exists;
- future local or cloud models.

The system must not assume that all participants use the same model, provider, context window, tool set, or execution environment.

The Round Table therefore needs a common protocol and a common task/session representation rather than a model-specific implementation.

## 3. What agents do at the Round Table

Agents may:

- present observations;
- provide reasoning or proposed solutions;
- request information from another participant;
- assign or delegate a subtask;
- return a result from a local or remote environment;
- challenge or validate another proposal;
- propose a tool call;
- propose a change to a repository or other resource;
- vote, rank, or otherwise contribute to a decision when the task policy requires it;
- converge on a final plan;
- request execution through Dark Star's controlled execution layer.

The Round Table must preserve attribution: Dark Star must know which participant produced each message, proposal, result, decision, and action request.

## 4. Agent-to-agent communication

The system should support direct logical communication between participants without requiring the human owner to relay every message manually.

For example, a remote Claude session may ask the local Ionis agent to inspect a repository or execute a diagnostic operation. The local agent can return the result to the Round Table, where the originating agent can use it to continue the task.

Conversely, a local agent may request assistance from a cloud model for analysis, planning, interpretation, or another capability that is more suitable for the remote model.

The human should therefore be able to initiate a task while Dark Star coordinates the participating agents automatically.

## 5. Human authority

Autonomous agent collaboration does not imply unrestricted autonomous execution.

The Round Table may reach a decision, but execution must still pass through Dark Star's authorization and safety mechanisms from Layer 02 and any later execution layer.

The system must distinguish between:

1. discussion;
2. proposal;
3. decision;
4. authorization;
5. execution;
6. verification.

A consensus between agents is not, by itself, equivalent to authorization by the owner.

A group of compromised or mistaken agents must not be able to manufacture authority merely by agreeing with one another.

## 6. Task delegation

The Round Table should support hierarchical task decomposition without requiring a permanent hierarchy of agents.

A main agent may create a task and delegate subtasks to other participants. Results return to the common task context and can be evaluated by the main agent or by the Round Table policy.

Example:

- Claude identifies a suspected build problem.
- Claude asks the local Ionis agent to inspect the repository and run the relevant diagnostics.
- Local Ionis performs only the operations permitted to its session.
- The result is returned to Dark Star.
- Claude analyses the result.
- Another model may independently review the proposed fix.
- Dark Star records the complete chain of observations, proposals, approvals and results.

## 7. Session and context integration

Layer 03 must use Layer 01 rather than creating an independent memory system.

Every Round Table should have a persistent identity and session context.

Messages, decisions, task assignments, tool results and important conclusions should be represented as context events that can be stored by the Dark Star memory layer using the project's CBMS-compatible representation.

The Round Table therefore becomes another consumer and producer of the shared Dark Star context system.

## 8. Tool access

Agents should not receive arbitrary unrestricted access to every tool available to Dark Star.

The Round Table should expose capabilities according to the participant's identity, session, task and authorization policy.

An agent may be able to request an operation without being able to execute it directly.

This preserves the distinction between intelligence and authority.

## 9. Failure and disagreement

The Round Table must explicitly support disagreement.

Agents may produce conflicting conclusions. Dark Star must preserve both positions rather than silently overwriting one with the other.

Possible future policies include:

- primary-agent decision;
- independent verification;
- majority decision;
- weighted confidence;
- specialist review;
- human approval;
- automatic escalation when agents disagree beyond a defined threshold.

These are policy mechanisms, not assumptions built into the individual model adapters.

## 10. Security boundary

The Round Table is not itself a security boundary.

Layer 02 remains responsible for authentication, session security, isolation and authorization. Layer 03 consumes those decisions.

This prevents a malicious participant from obtaining additional authority merely by joining a Round Table or persuading another model to request an operation.

All execution requests must remain attributable to the originating participant and session.

## 11. Network and protocol direction

The external entry point should be MCP over a modern HTTP transport where appropriate, with HTTPS providing transport security.

For agent-to-agent collaboration, Dark Star should not invent a proprietary wire protocol unless the requirements prove that existing standards are insufficient.

The architecture should investigate interoperability with current agent communication standards, including MCP and agent-to-agent protocols, while keeping Dark Star's internal task/session model independent of any one external protocol.

The exact protocol split is intentionally left open for engineering validation before implementation.

## 12. Isolation of execution environments

A participant may be associated with an isolated execution environment when the task requires tools or code execution.

The Round Table must treat such an environment as an execution resource, not as a permanent agent identity.

The environment may be ephemeral and destroyed after the task, while the important context, results and audit trail remain in Layer 01.

This allows Dark Star to provide agents with controlled working spaces without turning every agent connection into a permanently running virtual machine.

## 13. Recommended implementation language by layer

### Layer 01 — Context and Memory

**Primary language: Rust.**

Reason: this layer is infrastructure-heavy, performance-sensitive and expected to process very large volumes of structured context. It also needs to integrate tightly with the existing CBMS/WPC ecosystem. Rust provides a strong foundation for streaming, zero-copy or low-copy processing, concurrency and memory safety.

The CBMS-compatible storage format should remain the canonical representation rather than introducing a second memory format merely for convenience.

### Layer 02 — Connection, Security and Isolation

**Primary language: Rust.**

Reason: this is the security-critical network boundary. It needs robust asynchronous networking, explicit state management, strong typing, careful handling of credentials, session management and controlled interaction with sandbox/container/VM infrastructure.

Rust should own the security-sensitive control plane. External infrastructure such as containers, VMs or Azure services should be controlled by the Rust service rather than becoming the application's implicit security model.

### Layer 03 — Agent Execution / Round Table

**Primary language: Rust for the orchestration core.**

Reason: the Round Table is fundamentally a concurrent orchestration engine. It must manage sessions, event streams, participant state, task dependencies, deadlines, cancellation, authorization decisions and durable event recording.

Model-specific adapters may use other languages where an external SDK requires it, but the Dark Star orchestration core should remain Rust. This prevents the central execution model from becoming tied to one model provider or SDK ecosystem.

Python may be used later for optional model adapters or experimentation, but it should not become the trusted core of Dark Star.

## 14. Architectural language rule

The project should avoid choosing a different primary language merely because each layer has a different purpose.

The first three layers form one security- and context-sensitive system. Keeping their core in Rust reduces cross-language serialization boundaries, simplifies deployment, and makes the system easier to audit.

Other languages may exist at the edges where they provide a concrete advantage, but they should communicate through explicit, versioned interfaces.

## 15. Important architectural principle

**Dark Star is not the intelligence. Dark Star is the environment in which intelligence can safely cooperate.**

The Round Table therefore must remain model-agnostic, provider-agnostic and execution-policy-driven.

The models can change.

The agents can change.

The cloud provider can change.

The local hardware can change.

The Dark Star task, session, context, authorization and audit model should remain stable.

## 16. Next layer

The next planned architectural area is the execution/tool layer: the controlled mechanism through which an approved agent decision becomes an actual operation in a repository, local system, cloud environment or external service.

That layer must be designed separately from the Round Table so that agent discussion cannot silently become execution.