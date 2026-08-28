# Darkstar Layered Network Architecture

## Purpose

Darkstar models the protected path into AIONS as an ordered set of network and
trust boundaries. The topology contract is descriptive first; concrete VPN,
routing, firewall and provider implementations are separate concerns.

## Initial Azure deployment

The first cloud deployment uses three isolated container roles:

```text
Internet
   -> Sheriff Bridge container
   -> Kali Bridge container
   -> Darkstar container
   -> AIONS protected endpoint
```

This is the initial Azure test/development topology. It mirrors the intended
logical separation before dedicated hardware is installed.

## Physical deployment target

The network/security edge is intended to move to separate small machines:

```text
Mini PC #1  -> Sheriff Bridge
Mini PC #2  -> Kali security environment / Kali Bridge
Darkstar    -> dedicated headless control-plane host
```

AIONS remains the protected system beyond the control boundary.

## Boundary responsibilities

### Sheriff Bridge

Sheriff Bridge is the first perimeter gateway. It is intended to provide the
outer network boundary, VPN/firewall/routing controls and the first trust
transition into the security environment.

### Kali Bridge

Kali Bridge is an isolated security-testing environment. Web and network
security tooling must be treated as execution capabilities inside a sandbox,
not as part of the trusted AIONS endpoint.

### Darkstar

Darkstar is the control plane and Ghost Gate. Requests entering the protected
side should cross Session, Capability, Policy and Orchestrator controls before
a provider is allowed to act.

### AIONS

AIONS is the protected system endpoint. It remains outside the three-container
edge and is not modelled as another public network identity in this contract.

## Identity separation

The architecture permits separate externally visible network identities for
Sheriff Bridge, Kali Bridge and Darkstar, while AIONS remains private. This
document does not claim anonymity or guarantee a particular geolocation.
Identity separation is an architectural boundary; VPN, firewall and routing
providers implement it.

## Security rule

No provider may use this topology contract as authorization. Authorization
remains the responsibility of Darkstar Policy and Capability Gate.

## Code source of truth

The typed Rust representation lives in:

`crates/darkstar-core/src/network_topology.rs`
