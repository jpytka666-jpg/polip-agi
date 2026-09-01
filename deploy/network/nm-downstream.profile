# THIS IS VERY IMPORTANT!!!
# ==========================================
# AUTHOR: M. SZUL
# AI MODEL: Claude Opus 5
# TIMESTAMP: 2026-09-01 05:44:48
# REASON FOR CREATION: Szablon profilu NetworkManager dla prywatnego downstreamu Darkstar (Task 5, Step 5.3).
# MECHANICS: Szablon, nie gotowy plik. Znaczniki __PLACEHOLDER__ podstawia darkstar-gateway-apply z kontraktu env. Brak UUID i brak sekretow - UUID generuje NetworkManager przy imporcie.
# SYSTEM PART: deploy/network - natywna brama Darkstar.
# ARCHITECTURE FUNCTION: ipv4.method=shared daje adres bramy, DHCP i DNS dla klientow prywatnej podsieci bez wlasnego dnsmasq.
# DEPENDENCIES/LINKS: nmcli/NetworkManager, darkstar-gateway-apply, darkstar-gateway.env.example.
# TECH STACK: keyfile NetworkManager - format natywny hosta Ubuntu, bez dodatkowych zaleznosci.
# LOCAL WORKSPACE: D:\codex-fresh-2026-08-28\worktrees\polip-agi-darkstar-plan
# GIT COMMIT: PENDING
# GITHUB METADATA: jpytka666-jpg/polip-agi, branch docs/darkstar-headscale-hotspot-plan
# ==========================================

[connection]
id=__DARKSTAR_CONNECTION_NAME__
type=ethernet
interface-name=__DARKSTAR_DOWNSTREAM_IFACE__
autoconnect=false

[ethernet]

[ipv4]
method=shared
address1=__DARKSTAR_DOWNSTREAM_CIDR__
never-default=true

[ipv6]
method=disabled

[proxy]
