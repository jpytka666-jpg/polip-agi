# Darkstar Reference Architecture

## Canonical hierarchy

AIONS is the complete ecosystem. Darkstar is a component of AIONS: its online security/control plane, Ghost Gate, policy boundary and orchestrator.

```text
AIONS
  |
  +-- DARKSTAR
       |
       +-- Sheriff Bridge
       +-- Kali Bridge
       +-- Module Registry
       +-- Provider / Tentacle layer
       +-- Capability Selector / Gate
       +-- Policy
       +-- Session
       +-- Orchestrator
       +-- Round Table
       +-- System Graph
       +-- Live Run Graph
```

## Network path

The reference path into the protected ecosystem is:

```text
Internet
  -> Sheriff Bridge
  -> Kali Bridge
  -> Darkstar
  -> AIONS
```

### Sheriff Bridge

Sheriff Bridge is the perimeter/network gateway. It is intended for a small dedicated mini-PC in the physical deployment. It owns the outer network boundary, VPN/routing/firewall responsibilities and the first ingress/egress decision boundary.

### Kali Bridge

Kali Bridge is an isolated security/testing zone. It is intended for a separate mini-PC in the physical deployment. Web and network testing capabilities execute here and are treated as sandboxed capabilities, not as trusted AIONS components.

### Darkstar

Darkstar is the control/security plane. Browser, CLI and agent requests cross Session, Capability, Policy and Orchestrator boundaries before a Provider/Tentacle may act.

### AIONS

AIONS is the protected endpoint and the complete ecosystem behind the Darkstar boundary. AIONS must not be treated as another public perimeter layer.

## Initial Azure deployment

The first Azure representation uses three separate container roles:

```text
Container 1: Sheriff Bridge
Container 2: Kali Bridge
Container 3: Darkstar
```

The Azure representation is a deployment form, not a change to the logical hierarchy. Physical deployment can later move Sheriff Bridge and Kali Bridge to separate mini-PCs without changing Darkstar's control-plane contracts.

## Network identity separation

The architecture supports separate externally visible network identities for Sheriff Bridge, Kali Bridge and Darkstar, while AIONS remains private behind the protected boundary. Identity separation is not an anonymity guarantee and does not itself establish a specific geolocation.

## Automation and tentacles

Power Automate, PowerShell, GitHub, Azure, Docker, Linux, Kali tools and other external mechanisms are Providers/Tentacles of Darkstar when integrated. They are not peers of AIONS and they are not a second control plane.

The intended execution chain is:

```text
Browser / Round Table
  -> Session
  -> Capability Selector / Gate
  -> Policy
  -> Orchestrator
  -> Provider / Tentacle
  -> external system or module
  -> Event
  -> Live Run Graph
  -> Audit
```

## Non-duplication rule

Darkstar must adapt and orchestrate existing AIONS, WPC, CBMS, Ghost Gate, GitHub and other mechanisms rather than recreating their internal engines.
