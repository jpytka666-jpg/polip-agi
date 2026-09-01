<!--
THIS IS VERY IMPORTANT!!!
==========================================
AUTHOR: M. SZUL
AI MODEL: GPT-5 Codex
TIMESTAMP: 2026-08-29 11:19:11 Europe/London
REASON FOR CREATION: Aktualizacja architektury sieci o fazę Ghost Gate, natywną bramę Darkstar, prywatny mesh oraz nową nazwę Warlock Bridge.
==========================================
-->

# Darkstar Layered Network Architecture

## Purpose

Darkstar models the protected path into AIONS as an ordered set of network and
trust boundaries. The topology contract is descriptive first; concrete VPN,
routing, firewall and provider implementations are separate concerns.

## Current Ghost Gate phase

Until dedicated Warlock and Kali hosts are connected, Darkstar Ubuntu temporarily
owns the native gateway role:

~~~text
Internet
   -> Darkstar Ubuntu / Ghost Gate
   -> Windows and private clients
   -> AIONS
~~~

Windows must use Darkstar as its only active default route during acceptance.
Tailscale SaaS is a temporary recovery path, not the target network dependency.
The target private mesh uses self-hosted Headscale with Headplane as its
administration UI.

The detailed design and implementation sequence live in:

- docs/superpowers/specs/2026-08-29-darkstar-native-gateway-private-mesh-design.md
- docs/superpowers/plans/2026-08-29-darkstar-native-gateway-headscale-headplane-plan.md

## Initial Azure deployment

The first cloud deployment uses three isolated container roles:

```text
Internet
   -> Warlock Bridge container
   -> Kali Bridge container
   -> Darkstar container
   -> AIONS protected endpoint
```

This is the initial Azure test/development topology. It mirrors the intended
logical separation before dedicated hardware is installed.

## Physical deployment target

The network/security edge is intended to move to separate small machines:

```text
Mini PC #1  -> Warlock Bridge
Mini PC #2  -> Kali security environment / Kali Bridge
Darkstar    -> dedicated headless control-plane host
```

AIONS remains the protected system beyond the control boundary.

## Boundary responsibilities

### Warlock Bridge

Warlock Bridge is the first perimeter gateway. It is intended to provide the
outer network boundary, VPN/firewall/routing controls and the first trust
transition into the security environment.

Warlock replaces the earlier project name Warlock. At checkpoint fd8099e the
typed Rust topology still uses Warlock identifiers. They remain a legacy input
only until the backward-compatible migration in the implementation plan.

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
Warlock Bridge, Kali Bridge and Darkstar, while AIONS remains private. This
document does not claim anonymity or guarantee a particular geolocation.
Identity separation is an architectural boundary; VPN, firewall and routing
providers implement it.

## Security rule

No provider may use this topology contract as authorization. Authorization
remains the responsibility of Darkstar Policy and Capability Gate.

## Code source of truth

The typed Rust representation lives in:

`crates/darkstar-core/src/network_topology.rs`

## Nota migracyjna — Sheriff to dawna nazwa Warlocka

Perymetr nazywal sie **Sheriff Bridge** do 2026-09-01. Obowiazujaca nazwa to **Warlock Bridge**;
zmiana jest wylacznie nazewnicza i nie tworzy drugiego wezla — perymetr jest dokladnie jeden.

Stare dane pozostaja czytelne:

- `NetworkLayer` ma `#[serde(alias = "sheriff_bridge")]`, wiec zapisane wczesniej zdarzenia i
  konfiguracje wczytuja sie bez konwersji;
- `resolve_legacy_node_id("sheriff-bridge")` zwraca `"warlock-bridge"` dla odwolan trzymanych
  jako identyfikator tekstowy;
- zapis zawsze uzywa nowej nazwy — `"warlock_bridge"`.

Zrodlo: `crates/darkstar-core/src/network_topology.rs`.
