# Darkstar Layered Network Architecture

## Purpose

Darkstar models the protected path into AIONS as an ordered set of network and
trust boundaries. The topology contract is descriptive first; concrete VPN,
routing, firewall and provider implementations are separate concerns.

## Reference path

```text
Internet
   -> Watcher (perimeter / mini PC)
   -> Kali Security Zone (sandbox)
   -> Darkstar (control plane / policy gate)
   -> AIONS (protected endpoint)
```

## Boundary responsibilities

### Watcher

The Watcher is the first perimeter gateway. It is intended to live on a small
network appliance or mini PC and provide the outer network boundary for the
system.

### Kali Security Zone

Kali is an isolated security-testing environment. Web and network testing tools
must be treated as execution capabilities inside a sandbox rather than as part
of the trusted AIONS endpoint.

### Darkstar

Darkstar is the control plane. Requests entering the protected side should
cross Session, Capability, Policy and Orchestrator controls before a provider
is allowed to act.

### AIONS

AIONS is the protected system endpoint. It is not modelled as another public
network identity in this contract.

## Identity separation

The architecture permits separate externally visible network identities for
Watcher, Kali and Darkstar, while AIONS remains private. This document does not
claim anonymity or guarantee a particular geolocation. Identity separation is
an architectural boundary; VPN, firewall and routing providers implement it.

## Security rule

No provider may use this topology contract as authorization. Authorization
remains the responsibility of Darkstar Policy and Capability Gate.

## Code source of truth

The typed Rust representation lives in:

`crates/darkstar-core/src/network_topology.rs`
