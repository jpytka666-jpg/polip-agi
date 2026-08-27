# Dark Star MCP — Layer 04: Tentacles

## Purpose

Tentacles is the tool and external-action layer of Dark Star MCP.

Dark Star itself is not the intelligence. The agents participating in the Round Table decide what should be done; Tentacles gives those agents controlled ways to interact with the external digital world.

The layer is intentionally broad. It should eventually expose the web-facing capabilities that modern AI agents actually need, without turning Dark Star into one giant unstructured collection of privileged tools.

## Core concept

Once agents have established context, exchanged information and reached a decision through the Round Table, they can request an action through Tentacles.

Tentacles translates that intent into a controlled tool invocation against an external system.

Examples include:

- GitHub repositories, branches, commits, pull requests, issues, Actions and releases.
- Hugging Face models, datasets, spaces and related APIs.
- Web retrieval, page fetching, search and structured web interaction.
- Email and mailbox operations.
- Calendars and scheduling systems.
- Cloud storage and documents.
- Authentication and OAuth-backed services.
- APIs exposed by third-party services.
- Browser automation where an API does not exist or is insufficient.
- File transfer and controlled remote-resource access.
- Developer and infrastructure services.

The list is deliberately extensible. Tentacles should be a provider/plugin architecture, not a hard-coded list of every website on the Internet.

## MCP as the tool boundary

MCP is the natural interface between Dark Star and connected AI clients/tools. The current MCP specification defines tools as a mechanism through which models can request actions, while resources provide read-oriented context. Remote MCP deployments use Streamable HTTP; the 2026-07-28 specification also adds stronger authorization, routing and long-running task primitives.

Dark Star should therefore expose Tentacles through MCP-compatible tool definitions while keeping the actual provider implementations behind a stable internal interface.

The transport and protocol layer must remain separate from provider logic.

An agent should not need to know whether a tool is implemented through a REST API, GraphQL, OAuth flow, browser automation, a local process or another MCP server.

## Provider architecture

Each external system should be represented by a provider/adapter with a narrow contract.

For example, GitHub becomes a GitHub provider, Hugging Face becomes a Hugging Face provider, Google becomes a Google provider and Microsoft becomes a Microsoft provider.

Providers are responsible for translating Dark Star's normalized tool requests into the provider's native API operations.

The core Tentacles layer is responsible for:

- tool discovery;
- capability advertisement;
- input validation;
- authentication selection;
- credential acquisition and refresh;
- authorization checks;
- rate limiting;
- retries and backoff;
- timeout and cancellation;
- result normalization;
- provenance and tracing;
- audit events;
- error classification;
- isolation from untrusted tool output.

## Security boundary

Tentacles is a major security boundary because it is the point where agent decisions can create real-world side effects.

A tool call must never be equivalent to unrestricted execution.

Every invocation should carry enough identity and context for Dark Star to determine:

- which user initiated or authorized the action;
- which agent requested it;
- which session and Round Table decision produced it;
- which provider and resource are targeted;
- what operation is requested;
- whether the operation is read-only or mutating;
- whether explicit human approval is required;
- what credentials are permitted for the operation.

The authorization decision belongs to Dark Star, not to the model.

A model may propose an action. Tentacles may expose the capability. The policy engine decides whether that action can actually execute.

## Read, propose and execute

Tentacles should distinguish at least three classes of activity.

Read operations retrieve information and normally have the lowest risk.

Propose operations prepare a change, transaction or artifact without immediately applying the consequential side effect. Git branches, pull requests, staged files and draft changes are examples.

Execute operations create an external side effect and therefore require the strongest policy checks. Destructive operations, publishing, sending messages, modifying production resources and similar actions should be explicitly classified.

This model connects directly to Layer 02 isolation and Layer 03 Round Table decisions.

The Round Table can decide what should happen. Tentacles can prepare the action. Dark Star's authorization layer determines whether it may actually happen.

## Credentials

Credentials must never be exposed to models as ordinary tool arguments.

Tentacles should use provider-specific credential stores and short-lived credentials wherever the provider supports them.

OAuth should be preferred where appropriate. GitHub should use a dedicated GitHub App or equivalent narrowly scoped authorization rather than treating a personal long-lived token as the normal architecture.

Provider credentials should be injected into the provider execution context only for the minimum operation and duration required.

## Browser automation

Browser automation is useful but should not be the default implementation when a stable official API exists.

The preferred order is:

1. Official API.
2. Official SDK.
3. MCP server supplied by the provider or a trusted integration.
4. Browser automation as a controlled fallback.

Browser automation must remain isolated because pages are untrusted input and may contain prompt-injection content designed to manipulate an agent or tool executor.

## Tool result safety

Tentacles must treat every external response as untrusted data.

A webpage, email, issue comment, repository file, Hugging Face model card or API response can contain instructions intended for the model. Those instructions are data unless the Dark Star policy explicitly promotes them into executable intent.

This distinction is fundamental to preventing prompt injection from becoming tool execution.

## Parallel execution

Tentacles should support controlled parallel tool calls.

For example, the Round Table may decide to check GitHub, Hugging Face and an email inbox independently. Dark Star should be able to execute those independent operations concurrently, subject to provider limits, quotas, credentials and policy.

Dependent actions must remain ordered and must not be parallelized merely because they are technically possible.

## Long-running work

Some tools will not finish during one ordinary request. Downloads, repository analysis, browser workflows, data processing and external jobs may take significant time.

Tentacles should therefore support asynchronous tasks with explicit state, cancellation, timeout and resumability. The MCP Tasks direction is relevant to this architecture.

## Observability and provenance

Every Tentacles invocation should produce structured audit information sufficient to reconstruct what happened without storing unnecessary sensitive payloads.

The system should be able to answer:

- Which agent requested the operation?
- Which Round Table decision led to it?
- Which session initiated it?
- Which provider was contacted?
- Which tool was called?
- Which authorization policy allowed or denied it?
- What external operation occurred?
- What result or error came back?
- Which artifacts or state changes resulted?

This information should connect to the Git-based history where applicable and to the context/memory layer without duplicating massive external payloads unnecessarily.

## Language choice

Rust is the primary implementation language for the Tentacles core.

The reason is not simply consistency with Layers 01–03. Tentacles sits directly on a security and reliability boundary, performs high-concurrency I/O, manages credentials and cancellation, and will potentially orchestrate many simultaneous external operations. Rust is well suited to this core.

Provider adapters should not be forced into Rust when another language is materially better for a particular integration. Python or TypeScript may be used as isolated provider workers when an ecosystem advantage is significant, especially for browser automation, AI/ML-specific APIs or provider SDKs.

Those workers must communicate with the Rust core through a strictly defined interface and must not bypass Dark Star authorization.

This gives Dark Star a Rust security/control plane while allowing specialized tool implementations where they provide real value.

## Design principle

Tentacles should make Dark Star powerful without making it omnipotent by default.

The goal is not to give an agent access to everything.

The goal is to give an authorized agent the correct capability, at the correct time, against the correct resource, with the correct credentials, under the correct policy, and with a complete audit trail.

Tentacles are the hands of Dark Star. They are not the brain and they are not the authority.
