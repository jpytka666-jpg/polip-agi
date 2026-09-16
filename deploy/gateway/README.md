# THIS IS VERY IMPORTANT!!!
# ==========================================
# AUTHOR: M. SZUL
# AI MODEL: GPT-5.6 Sol
# TIMESTAMP: 2026-08-29 03:25:00 Europe/London
# REASON FOR CREATION: Make Darkstar the mandatory network gateway for the current Windows host and the future iOnis OS host.
# MECHANICS: Defines an immediate WireGuard full-tunnel gateway and the later two-interface physical gateway without bypassing Darkstar policy boundaries.
# SYSTEM PART: Darkstar network perimeter / gateway
# ARCHITECTURE FUNCTION: Ensure protected hosts have no ordinary Internet path that bypasses Darkstar.
# DEPENDENCIES/LINKS: WireGuard, nftables, Linux IP forwarding, Darkstar network architecture, future Sheriff/Kali perimeter layers.
# TECH STACK: Ubuntu Linux networking + WireGuard + nftables.
# LOCAL WORKSPACE: /home/owner/polip-agi
# GIT COMMIT: PENDING
# GITHUB METADATA: jpytka666-jpg/polip-agi / feat/darkstar-gateway
# ==========================================

# Darkstar mandatory gateway

## Goal

The current Windows machine, and later the iOnis OS server, must not use a normal direct Internet route. Darkstar is the mandatory gateway between the protected host and external networks.

Immediate topology:

```text
Internet / upstream LAN
        |
        v
Darkstar Ubuntu
  - WireGuard gateway
  - nftables forwarding/NAT
  - DNS boundary
        |
        v
WireGuard full tunnel
        |
        v
Windows now / iOnis OS later
```

Long-term physical topology:

```text
Internet
   |
Sheriff Bridge
   |
Kali Bridge
   |
Darkstar WAN NIC
   |
Darkstar policy/gateway
   |
Darkstar LAN NIC / protected switch
   |
Windows now / iOnis OS later
```

The immediate WireGuard phase enforces the same logical boundary before the dedicated physical NIC/switch topology exists.

## Separation of responsibilities

Serveo is management access only. It is not the protected host's Internet transport.

WireGuard is the immediate protected transport between Windows/iOnis and Darkstar.

nftables on Darkstar performs forwarding controls and source NAT for the protected WireGuard subnet.

Darkstar application Policy remains the authority for application/module operations. Kernel routing is infrastructure and must not be treated as application authorization.

## Immediate addressing contract

Use a dedicated private subnet that does not overlap the local LAN:

```text
Darkstar WireGuard: 10.77.0.1/24
Protected host:     10.77.0.2/32
WireGuard UDP:      51820
```

If 10.77.0.0/24 overlaps an existing network, choose another RFC1918 subnet before deployment.

## Windows/iOnis full-tunnel contract

The protected client must use one Darkstar WireGuard peer with:

```text
AllowedIPs = 0.0.0.0/0, ::/0
DNS = 10.77.0.1
```

On WireGuard for Windows, a single peer containing a /0 allowed route activates the built-in block-untunneled-traffic firewall semantics. This is required; do not replace /0 with two /1 routes because that intentionally disables the kill-switch behavior.

The Windows tunnel should be installed as a WireGuard tunnel service so it can start automatically at boot. Secrets/private keys are local machine state and must never be committed to GitHub.

## Darkstar gateway contract

Darkstar must:

1. enable IPv4 forwarding;
2. accept forwarded traffic from the protected WireGuard interface only toward the configured WAN/uplink;
3. accept established/related return traffic;
4. masquerade the protected WireGuard subnet on the WAN/uplink;
5. reject unexpected forwarding arriving from the protected WireGuard interface;
6. provide or explicitly select DNS reachable through the tunnel;
7. keep WireGuard keys outside the repository;
8. start WireGuard automatically through systemd/wg-quick after validation.

The first deployment must not flush or replace the host's complete nftables ruleset because Docker may maintain networking state of its own. Darkstar gateway rules should be isolated in dedicated tables/chains and validated with `nft -c` before being loaded.

## Inbound Internet rule

Darkstar does not automatically forward unsolicited Internet traffic to Windows/iOnis.

Default inbound model:

```text
Internet -> Darkstar -> DROP
```

A service is exposed inward only through an explicit reverse-proxy/DNAT/provider contract that has been deliberately configured. Generic WAN-to-protected-host forwarding is forbidden.

## Deployment sequence

1. Keep the Serveo SSH management channel working first so gateway changes do not remove administration access.
2. Discover the real Darkstar uplink interface and current routes; do not guess interface names.
3. Install/configure WireGuard on Darkstar and generate its key locally.
4. Generate the Windows/iOnis key locally and add only its public key to Darkstar.
5. Validate the WireGuard handshake on the dedicated 10.77.0.0/24 subnet before changing the client's default route.
6. Enable IP forwarding and the isolated nftables NAT/forwarding rules.
7. Enable the Windows full-tunnel /0 routes and verify that its public egress address is Darkstar's egress address.
8. Stop the tunnel deliberately and verify the protected host loses Internet access rather than falling back to its normal adapter.
9. Verify DNS also stops/leaks correctly according to the full-tunnel contract.
10. Only after these tests pass, enable both sides to start automatically at boot.

## Physical hardening stage

The final architecture should remove the protected host's physical ability to bypass Darkstar:

- Darkstar gets a dedicated WAN-facing interface and a dedicated protected-LAN interface (or equivalent VLAN separation).
- Windows/iOnis connects only to the protected interface/switch.
- Wi-Fi and any second direct path to the upstream router are disabled/removed on the protected host.
- The protected host's default gateway is Darkstar.
- Darkstar remains the only device with an upstream route.

At that point the isolation is enforced by topology as well as software.
