<!-- darkstar-header-v1 -->
<!-- po co: 2026-08-29-darkstar-native-gateway-headscale-headplane-plan.md -->
<!-- nie wolno: hotspot, ruszac wlp2s0, wracac do 10.44, gasic DARKSTAR-WiFi, haslo w gicie -->
<!-- autor: Marcin -->
<!-- powstal: 2026-09-01 -->
<!--
THIS IS VERY IMPORTANT!!!
==========================================
AUTHOR: M. SZUL
AI MODEL: GPT-5 Codex
TIMESTAMP: 2026-08-29 11:19:11 Europe/London
REASON FOR CREATION: Szczegółowy plan wykonawczy dalszego rozwoju Darkstar od checkpointu fd8099e, obejmujący firewall, prawdziwy reboot, natywną bramę, hotspot, Headscale, Headplane i Control Room.
==========================================
-->

# Darkstar Native Gateway, Headscale and Headplane Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans or superpowers:subagent-driven-development to implement this plan task-by-task. Use superpowers:test-driven-development for code tasks and superpowers:verification-before-completion before every milestone is marked complete.

**Goal:** Przekształcić obecny host Darkstar w sprawdzoną, natywną bramę Ghost Gate dla Windows i prywatnych urządzeń, uruchomić własny Headscale z panelem Headplane, połączyć wszystko z autoryzowanym Control Room oraz dopiero po udowodnieniu nowej drogi wycofać zależność od Tailscale SaaS.

**Architecture:** Rust/Axum pozostaje zaufanym control plane. React/TypeScript/Vite jest interaktywnym UI. NetworkManager zarządza interfejsami i hotspotem, nftables filtrowaniem/NAT, systemd startem po bootowaniu, Headscale prywatnym control serverem mesh, a Headplane panelem administracyjnym. Żadna przeglądarka nie uruchamia poleceń systemowych bezpośrednio.

**Tech Stack:** Rust 2024, Axum 0.8, Tokio, React 19, TypeScript 6, Vite 8, React Flow 12, NetworkManager/nmcli, nftables, systemd, Docker Compose v2, Headscale, Headplane, Bash/POSIX shell na Ubuntu oraz PowerShell na Windows.

**Spec:** docs/superpowers/specs/2026-08-29-darkstar-native-gateway-private-mesh-design.md

**Roadmap:** docs/DARKSTAR_ROADMAP.md

## Global Constraints

- Exact handoff is fd8099e4e3c3399ae69a885fa2ed32ab0c57d2df.
- Work on feat/darkstar-module-control through a clean isolated worktree.
- Preserve all existing upstream licenses and author notices.
- Every new project file starts with the AUTHOR: M. SZUL metadata block in a valid comment syntax.
- Never commit passwords, Wi-Fi PSKs, API tokens, private keys, machine keys or database contents.
- Never use Serveo.
- Tailscale SaaS is a temporary recovery route until explicit cutover acceptance.
- A status of enabled is not reboot proof; boot_id must change.
- Prefer Ethernet downstream first. Enable same-radio Wi-Fi client + AP only after capability and stability tests.
- Use existing module policy/provider boundaries; do not add arbitrary shell execution.
- Keep Darkstar API on localhost/private interfaces until the private route and authentication are proven.
- Each task ends with focused tests and a small commit.
- Do not begin a task if its stated entry gate is not satisfied.

## Evidence baseline

Record these facts in the first execution journal:

~~~text
Windows worktree HEAD: fd8099e4e3c3399ae69a885fa2ed32ab0c57d2df
Ubuntu repo HEAD:      fd8099e4e3c3399ae69a885fa2ed32ab0c57d2df
Ubuntu repo status:    clean
Darkstar host:         CBMS / darkstar-cbms
Temporary mesh IP:     100.71.8.70
Upstream Wi-Fi:        wlp2s0 / Intel Wireless-AC 8260 / iwlwifi
Downstream Ethernet:   enp1s0, currently down/unavailable
Firewall unit:         enabled, inactive/dead
Firewall journal:      no entries
Current boot:          2026-08-28 04:12:48
Firewall files saved:  2026-08-29 05:36
IPv4 forwarding:       1
IPv6 forwarding:       1
Headscale command:     absent
Tailscale:             active temporary fallback
SSH key authentication: rejected; password login works temporarily
React frontend:        Vite starter, not a finished Control Room
Rust baseline:         fmt passes; 3 integration tests fail with unexpected 401
~~~

---

## Task 0: Preserve checkpoint and establish deterministic green tests

**Entry gate:** Isolated worktree points at fd8099e and is clean.

**Files:**

- Modify: crates/darkstar-server/tests/memory_http.rs
- Verify: crates/darkstar-server/src/http.rs
- Verify: frontend/package.json
- Create only if the test API is reused: crates/darkstar-server/src/test_support.rs

**Interfaces:**

- Consumes AppState with a known API token.
- Produces deterministic integration tests independent of DARKSTAR_API_TOKEN in the developer shell.

- [x] **Step 0.1: Reconfirm the exact checkpoint**

Run:

~~~bash
git rev-parse HEAD
git status --short --branch
~~~

Expected: HEAD equals fd8099e4e3c3399ae69a885fa2ed32ab0c57d2df and no changed paths.

- [x] **Step 0.2: Reproduce the three failures**

Run:

~~~bash
cargo test -p darkstar-server --test memory_http
~~~

Expected before the fix: session_can_write_and_read_memory, module_action_denies_missing_capability and module_action_accepts_authorized_start_request fail because session creation returns 401.

- [x] **Step 0.3: Write a test that proves fail-closed production behavior**

Add an integration test constructing AppState with api_token=None and assert
that an authenticated route returns 401. Do not mutate the process environment
inside parallel tests. AppState::from_env remains the production constructor.

- [x] **Step 0.4: Replace the fixture with an explicit token**

Change test_state so it constructs AppState directly with:

- api_token equal to Arc&lt;str&gt; containing secret;
- an empty session map;
- a fresh MemoryStore;
- a default RunStreamHub.

Prefer a crate-private AppState constructor only if at least two test modules need it. Do not weaken AppState::from_env.

- [x] **Step 0.5: Run the focused tests**

~~~bash
cargo test -p darkstar-server --test memory_http
~~~

Expected: all tests pass.

- [x] **Step 0.6: Run the complete Rust quality gate**

~~~bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
~~~

Expected: all commands exit 0.

- [ ] **Step 0.7: Verify the frontend starter without changing product UI**

From frontend:

~~~bash
npm ci
npm run lint
npm run build
~~~

Expected: dependency lock is honored and lint/build pass. Record any checkpoint failure before changing dependencies.

- [x] **Step 0.8: Confirm metadata and workspace hygiene**

Check changed/new source files for AUTHOR: M. SZUL. Confirm that frontend/dist, target and dependency caches are ignored and are not staged.

- [x] **Step 0.9: Commit the baseline repair**

~~~bash
git add crates/darkstar-server/tests/memory_http.rs
git commit -m "test(darkstar): make HTTP fixtures deterministic"
~~~

**Exit gate:** Rust tests are green and the failure is not hidden by setting a global shell variable.

---

## Task 0A: Restore key-based SSH before network mutation

**Entry gate:** Task 0 is green. Password login still works and no firewall
mutation is in progress.

**Files:**

- Inspect locally: C:\Users\User\.ssh\darkstar_codex
- Inspect locally: C:\Users\User\.ssh\darkstar_codex.pub
- Inspect on Ubuntu: /home/owner/.ssh/authorized_keys
- No repository file is created because private access material does not belong in Git.

- [ ] **Step 0A.1: Record the local public-key fingerprint**

On Windows:

~~~powershell
ssh-keygen -lf C:\Users\User\.ssh\darkstar_codex.pub
~~~

Never print or copy the private key.

- [ ] **Step 0A.2: Inspect remote ownership and modes**

Through the temporary password session:

~~~bash
stat -c '%U %G %a %n' /home/owner
stat -c '%U %G %a %n' /home/owner/.ssh
stat -c '%U %G %a %n' /home/owner/.ssh/authorized_keys
~~~

Expected: owner owns .ssh/authorized_keys, directory mode 700 and file mode 600.

- [ ] **Step 0A.3: Compare the exact public key**

Read only darkstar_codex.pub locally and search for that exact public-key body
in authorized_keys. Do not compare only the comment.

- [ ] **Step 0A.4: Repair without overwriting existing keys**

If missing, append exactly one public-key line. If ownership/modes are wrong,
repair only /home/owner/.ssh and authorized_keys. Do not replace the whole file.

- [ ] **Step 0A.5: Check sshd policy**

~~~bash
sshd -T | grep -E 'pubkeyauthentication|authorizedkeysfile|passwordauthentication'
~~~

Confirm public-key authentication and the resolved authorized_keys path.

- [ ] **Step 0A.6: Test a second BatchMode session**

Keep the password session open and run from Windows:

~~~powershell
ssh -o BatchMode=yes -o ConnectTimeout=15 -i C:\Users\User\.ssh\darkstar_codex owner@100.71.8.70 hostnamectl --static
~~~

Expected: CBMS with exit code 0 and no password prompt.

- [ ] **Step 0A.7: Test reconnection**

Close only the key-test session, reconnect twice with BatchMode and verify
repository status. Do not disable password login in this task.

**Exit gate:** Two fresh key-only sessions succeed. Firewall and reboot work may
not begin while key authentication is unproven.

---

## Task 1: Bring the installed host firewall under source control

**Entry gate:** Task 0 is green. The current Ubuntu files have been read before modification.

**Files:**

- Create: deploy/firewall/host-guard.nft
- Create: deploy/firewall/darkstar-firewall-apply
- Create: deploy/firewall/darkstar-firewall-verify
- Create: deploy/firewall/README.md
- Create: deploy/systemd/darkstar-firewall.service
- Modify: deploy/README.md

**Interfaces:**

- Consumes explicit interface names and private subnets.
- Produces one idempotent nftables table named inet darkstar_host_guard and one systemd oneshot unit.

- [x] **Step 1.1: Capture checksums of installed files**

On Ubuntu:

~~~bash
sudo sha256sum /etc/darkstar/host-guard.nft
sudo sha256sum /usr/local/sbin/darkstar-firewall-apply
sudo sha256sum /etc/systemd/system/darkstar-firewall.service
~~~

Store checksums in the implementation journal, not as claims in source templates.

- [x] **Step 1.2: Add a failing repository presence check**

~~~bash
test -f deploy/firewall/host-guard.nft
test -x deploy/firewall/darkstar-firewall-apply
test -f deploy/systemd/darkstar-firewall.service
~~~

Expected before creation: at least one command fails.

- [x] **Step 1.3: Create the versioned nftables ruleset**

Start from the inspected host-guard.nft, then add explicit named sets/chains for:

- loopback;
- established/related;
- ICMP and ICMPv6;
- temporary management through tailscale0;
- direct WireGuard transport UDP 41641 on wlp2s0 while fallback exists;
- downstream management on enp1s0 after Task 5;
- forward policy that defaults to drop;
- forwarding only from the Darkstar downstream subnet to wlp2s0;
- masquerade only for the selected private subnet;
- no public input to the Darkstar API.

Do not encode a Wi-Fi password or a temporary IP assigned by a client.

- [x] **Step 1.4: Implement the apply script**

The POSIX script must:

1. use set -eu;
2. verify required files;
3. run nft -c before mutation;
4. load rules atomically when possible;
5. return non-zero on validation/load failure;
6. log a concise result to journald/stdout;
7. never flush unrelated nftables tables.

- [x] **Step 1.5: Implement the verify script**

The script checks:

- table inet darkstar_host_guard exists;
- input policy is drop;
- forward policy is drop;
- loopback and established rules exist;
- current management route exists;
- IPv4 and IPv6 forwarding have expected values.

It must only inspect and must not repair.

- [x] **Step 1.6: Version the systemd unit**

Use Type=oneshot and RemainAfterExit=yes. Order it after local filesystems and before services that require the protected network. ExecStart points to /usr/local/sbin/darkstar-firewall-apply. Add ExecStartPost for the verify script only after manual verification proves it safe.

- [x] **Step 1.7: Add install and rollback instructions**

Document:

- exact source and destination paths;
- chmod/mode;
- daemon-reload;
- start/status/journal commands;
- rollback that restores the previous three files from timestamped root-owned backups;
- physical access requirement.

- [x] **Step 1.8: Validate syntax without applying**

On Ubuntu:

~~~bash
sudo nft -c -f deploy/firewall/host-guard.nft
systemd-analyze verify deploy/systemd/darkstar-firewall.service
shellcheck deploy/firewall/darkstar-firewall-apply deploy/firewall/darkstar-firewall-verify
~~~

If shellcheck is absent, record that fact; do not silently skip syntax checking. Use sh -n as the minimum:

~~~bash
sh -n deploy/firewall/darkstar-firewall-apply
sh -n deploy/firewall/darkstar-firewall-verify
~~~

- [x] **Step 1.9: Commit firewall-as-code**

~~~bash
git add deploy/firewall deploy/systemd/darkstar-firewall.service deploy/README.md
git commit -m "feat(darkstar): version host firewall controls"
~~~

**Exit gate:** Repository copies are reviewed, syntax-valid and install/rollback steps exist. No live rules changed yet.

---

## Task 2: Install and manually validate darkstar-firewall.service

**Entry gate:** Task 1 syntax checks pass. Operator has physical access or the current Tailscale recovery route is tested.

**Files:**

- Use: deploy/firewall/*
- Use: deploy/systemd/darkstar-firewall.service
- Create after success: docs/operations/evidence/2026-08-29-firewall-manual-activation.md

- [x] **Step 2.1: Record pre-change network evidence**

On Ubuntu record:

~~~bash
ip -brief address
ip route
ip -6 route
sudo nft list ruleset
systemctl status darkstar-firewall.service --no-pager --full
curl -fsS http://127.0.0.1:18080/health
~~~

Sanitize public tokens and Wi-Fi secrets before saving evidence.

- [x] **Step 2.2: Create root-owned backups**

Create a timestamped directory under /var/backups/darkstar-firewall and copy only:

- /etc/darkstar/host-guard.nft;
- /usr/local/sbin/darkstar-firewall-apply;
- /etc/systemd/system/darkstar-firewall.service.

List and checksum the backup before continuing.

- [x] **Step 2.3: Install versioned files**

Use sudo install with:

- 0644 for nft rules and systemd unit;
- 0755 for apply/verify scripts;
- root:root ownership.

- [x] **Step 2.4: Reload systemd and verify unit resolution**

~~~bash
sudo systemctl daemon-reload
systemctl cat darkstar-firewall.service
systemctl show darkstar-firewall.service -p FragmentPath -p UnitFileState
~~~

- [x] **Step 2.5: Start the unit manually**

~~~bash
sudo systemctl start darkstar-firewall.service
systemctl is-active darkstar-firewall.service
systemctl status darkstar-firewall.service --no-pager --full
~~~

Expected: active with SubState=exited because RemainAfterExit=yes.

- [x] **Step 2.6: Run post-apply verification**

~~~bash
sudo /usr/local/sbin/darkstar-firewall-verify
sudo nft list table inet darkstar_host_guard
curl -fsS http://127.0.0.1:18080/health
~~~

- [x] **Step 2.7: Verify management from Windows**

From Windows:

~~~powershell
Test-NetConnection 100.71.8.70 -Port 22
ssh owner@100.71.8.70 hostnamectl --static
~~~

Expected: SSH remains reachable through the temporary fallback.

- [x] **Step 2.8: Exercise idempotency**

Start the unit a second time, restart it once, and confirm there is still exactly one inet darkstar_host_guard table.

- [ ] **Step 2.9: Exercise rollback without losing the session**

Review the rollback command from a second local terminal. Do not intentionally cut the only working route. Confirm backups are readable and paths are exact.

- [x] **Step 2.10: Write evidence and commit**

The evidence file includes commands, exit codes, service status and sanitized checksums.

~~~bash
git add docs/operations/evidence/2026-08-29-firewall-manual-activation.md
git commit -m "docs(darkstar): record firewall activation evidence"
~~~

**Exit gate:** Service is active/exited, nft rules are present, API is healthy and Windows management still works.

---

## Task 3: Perform a real reboot persistence test

**Entry gate:** Task 2 passes. Physical access is available. No package installation or unrelated service change is in progress.

**Files:**

- Create: docs/operations/evidence/2026-08-29-darkstar-real-reboot.md
- Modify only if failure is found: deploy/systemd/darkstar-firewall.service
- Modify only if failure is found: deploy/firewall/*

- [x] **Step 3.1: Record the old boot identity**

~~~bash
cat /proc/sys/kernel/random/boot_id
uptime -s
who -b
~~~

Store old_boot_id in the evidence draft.

- [x] **Step 3.2: Verify all pre-reboot gates**

~~~bash
systemctl is-enabled darkstar-firewall.service
systemctl is-active darkstar-firewall.service
systemctl is-enabled darkstar.service
systemctl is-active darkstar.service
systemctl is-enabled darkstar-dependency-retry.timer
systemctl is-active darkstar-dependency-retry.timer
curl -fsS http://127.0.0.1:18080/health
~~~

Every command must match its expected state.

- [ ] **Step 3.3: Confirm Windows has a recovery route**

Keep one Windows terminal ready for SSH and one local/physical console available. Record the current route and successful SSH.

- [x] **Step 3.4: Issue the real reboot**

Run directly on Ubuntu or through the authenticated SSH session:

~~~bash
sudo systemctl reboot
~~~

Do not substitute a helper that merely closes the connection.

- [ ] **Step 3.5: Prove the host went down**

From Windows poll port 22 at a bounded interval and record at least one failed connection after the reboot command.

- [x] **Step 3.6: Wait for the host to return**

Poll for a maximum of ten minutes. Stop and use physical recovery if the host does not return.

- [x] **Step 3.7: Prove a new boot occurred**

After reconnect:

~~~bash
cat /proc/sys/kernel/random/boot_id
uptime -s
who -b
~~~

The new boot_id must differ from old_boot_id. If it is equal, the test fails even if SSH reconnected.

- [x] **Step 3.8: Verify persistence**

~~~bash
systemctl is-active darkstar-firewall.service
systemctl status darkstar-firewall.service --no-pager --full
journalctl -b -u darkstar-firewall.service --no-pager
sudo nft list table inet darkstar_host_guard
sysctl net.ipv4.ip_forward
sysctl net.ipv6.conf.all.forwarding
systemctl is-active darkstar.service
systemctl is-active darkstar-dependency-retry.timer
curl -fsS http://127.0.0.1:18080/health
~~~

- [ ] **Step 3.9: Verify Windows egress**

Confirm Windows can reach Darkstar and a public HTTPS endpoint through the current Darkstar exit route. Record route table and public egress IP without exposing credentials.

- [ ] **Step 3.10: Handle failure by layer**

If firewall failed, inspect only its journal/unit. If Darkstar failed, inspect darkstar.service/Compose. If routing failed, inspect sysctl/routes/nft. Do not make multiple speculative changes at once.

- [x] **Step 3.11: Commit the reboot evidence**

~~~bash
git add docs/operations/evidence/2026-08-29-darkstar-real-reboot.md
git commit -m "docs(darkstar): prove firewall reboot persistence"
~~~

**Exit gate:** A changed boot_id and green post-boot checks are present in the committed evidence.

---

## Task 4: Run hardware preflight and select the downstream topology

**Entry gate:** Real reboot persistence from Task 3 is proven.

**Files:**

- Create: deploy/network/darkstar-network-preflight
- Create: deploy/network/README.md
- Create: docs/operations/evidence/2026-08-29-network-preflight.md

- [ ] **Step 4.1: Add a failing preflight presence check**

~~~bash
test -x deploy/network/darkstar-network-preflight
~~~

Expected before creation: fail.

- [ ] **Step 4.2: Implement read-only interface discovery**

The script prints machine-readable and human-readable data for:

- NetworkManager version;
- wlp2s0 driver and firmware;
- AP capability;
- enp1s0 carrier/state;
- default IPv4/IPv6 routes;
- active NetworkManager profiles;
- current forwarding values;
- installed nft and systemd versions.

It must not activate, disconnect or modify a connection.

- [ ] **Step 4.3: Add iw capability inspection**

If iw is absent, install only the Ubuntu package iw after recording the package source and version. Run:

~~~bash
iw list
~~~

Capture Supported interface modes and valid interface combinations.

- [ ] **Step 4.4: Decide whether same-radio AP+STA is permitted**

Permit it only if iw reports a combination containing managed and AP with enough total interfaces/channels. Otherwise mark it unsupported for this host.

- [ ] **Step 4.5: Test Ethernet carrier**

Connect Darkstar enp1s0 directly to Windows Ethernet. Confirm carrier and link without changing default routes:

~~~bash
cat /sys/class/net/enp1s0/carrier
ethtool enp1s0
~~~

- [ ] **Step 4.6: Select the baseline**

Selection order:

1. wlp2s0 upstream + enp1s0 downstream;
2. wlp2s0 upstream + second supported USB Wi-Fi AP;
3. same-radio wlp2s0 AP+STA only after capability and soak tests.

Write the selected result and reason in the evidence file.

- [ ] **Step 4.7: Run script quality checks**

~~~bash
sh -n deploy/network/darkstar-network-preflight
shellcheck deploy/network/darkstar-network-preflight
~~~

- [ ] **Step 4.8: Commit preflight tooling and evidence**

~~~bash
git add deploy/network docs/operations/evidence/2026-08-29-network-preflight.md
git commit -m "feat(darkstar): add native gateway preflight"
~~~

**Exit gate:** One supported downstream topology is selected with evidence. No hotspot has been started yet.

---

## Task 5: Implement the native Darkstar gateway and hotspot module

**Entry gate:** Task 4 selected a supported topology and Task 3 rollback remains available.

**Live status 2026-09-01 — measured, not assumed.** An equivalent gateway is already running and
was reached by a different route than this task describes, so most steps below stay unticked even
though their outcome exists:

- Gateway live: NetworkManager profile `DARKSTAR-WiFi` (`95a89b38-ee71-484e-98c2-a6d02e8b92b3`)
  on `enp1s0`, `ipv4.method=shared`, `192.168.2.1/24`, `ipv6.method=disabled`, autoconnect on.
  It predates this plan; `darkstar-gateway-apply` has never been run with `DARKSTAR_DRY_RUN=0`.
- Host guard live: `darkstar_downstream_ipv4 = { 192.168.2.0/24 }` installed and loaded,
  `darkstar-firewall.service` enabled and active, reboot persistence proven.
- Downstream path live: EE hub at `192.168.2.2` acting as an access point with its DHCP server
  off, SSID `EE-57GMTG`, cable in a LAN port.
- Client live: Windows at `192.168.2.50/24`, gateway `192.168.2.1`, DNS `8.8.8.8`, reaching the
  public Internet through Darkstar NAT.
- **Known gap:** DHCP does not traverse the hub Wi-Fi — clients fell back to `169.254.x.x`, so
  every wireless client currently needs a static address. Step 5.12 therefore stays unticked.
- Deferred: the EE hub management GUI is unusable (`AppInitScreen` never renders); hotspot on
  `wlp2s0` is out of scope by explicit instruction.

Evidence: `docs/operations/evidence/2026-09-01-ee-hub-ap-static-windows-online.md`,
`2026-09-01-host-guard-192-168-2-apply.md`, `2026-09-01-existing-downstream-gateway.md`,
`2026-09-01-ee-hub-appinitscreen-dead.md`.

**Files:**

- Create: deploy/network/darkstar-gateway.env.example
- Create: deploy/network/darkstar-gateway-apply
- Create: deploy/network/darkstar-gateway-stop
- Create: deploy/network/darkstar-gateway-verify
- Create: deploy/network/connections/darkstar-downstream.nmconnection.example
- Create: deploy/systemd/darkstar-gateway.service
- Modify: deploy/firewall/host-guard.nft
- Modify: deploy/network/README.md
- Modify: deploy/README.md

**Interfaces:**

- Upstream: wlp2s0.
- Preferred downstream: enp1s0.
- Default private subnet: 192.168.2.0/24.
- Darkstar downstream address: 192.168.2.1.
- Hotspot SSID and PSK: local environment/NetworkManager secret storage, never Git.

- [ ] **Step 5.1: Write failing file and dry-run checks**

~~~bash
test -x deploy/network/darkstar-gateway-apply
test -x deploy/network/darkstar-gateway-verify
test -f deploy/systemd/darkstar-gateway.service
~~~

- [x] **Step 5.2: Define the environment contract**

The example file contains non-secret values:

~~~text
DARKSTAR_UPSTREAM_IFACE=wlp2s0
DARKSTAR_DOWNSTREAM_IFACE=enp1s0
DARKSTAR_DOWNSTREAM_CIDR=192.168.2.1/24
DARKSTAR_DOWNSTREAM_SUBNET=192.168.2.0/24
DARKSTAR_CONNECTION_NAME=darkstar-downstream
DARKSTAR_MODE=ethernet
~~~

For hotspot mode, the actual SSID/PSK live in root-readable local configuration outside Git.

- [x] **Step 5.3: Create a NetworkManager profile template**

Use ipv4.method=shared for the private downstream and ipv6.method=disabled initially unless an explicitly tested IPv6 prefix delegation design is selected. Do not copy a host-generated UUID or secret into Git.

- [ ] **Step 5.4: Implement apply as an idempotent operation**

The script:

1. validates interface names against /sys/class/net;
2. validates the private CIDR;
3. refuses to use the same interface twice unless preflight explicitly allowed concurrent mode;
4. imports/updates only DARKSTAR_CONNECTION_NAME;
5. activates the downstream profile;
6. applies the versioned firewall;
7. verifies gateway address, forwarding, DHCP/DNS listener and rules;
8. returns non-zero on partial activation.

- [ ] **Step 5.5: Implement stop without deleting profiles**

Stop deactivates only DARKSTAR_CONNECTION_NAME and leaves the profile available for recovery. It must not disconnect the upstream.

- [ ] **Step 5.6: Implement verify as read-only**

Verify:

- upstream has a default route;
- downstream has 192.168.2.1/24;
- DHCP/DNS for shared mode is present;
- nft forward/NAT rules reference only expected interfaces/subnet;
- Darkstar API remains localhost/private;
- no unexpected public listening ports appeared.

- [ ] **Step 5.7: Implement hotspot mode**

When DARKSTAR_MODE=hotspot:

- use the preflight-approved AP interface;
- configure WPA2/WPA3 according to driver support;
- set a local secret through nmcli without printing it;
- choose a tested band/channel;
- avoid automatic fallback to an open network;
- keep Ethernet mode available.

- [ ] **Step 5.8: Add systemd ordering**

darkstar-gateway.service:

- runs after NetworkManager and darkstar-firewall;
- requires the firewall unit;
- remains active after successful apply;
- has ExecStop using the stop script;
- does not restart forever;
- is not enabled until manual testing passes.

- [ ] **Step 5.9: Validate files offline**

~~~bash
sh -n deploy/network/darkstar-gateway-apply
sh -n deploy/network/darkstar-gateway-stop
sh -n deploy/network/darkstar-gateway-verify
systemd-analyze verify deploy/systemd/darkstar-gateway.service
~~~

- [ ] **Step 5.10: Install without enabling**

Install scripts/config/unit with root ownership. Reload systemd. Confirm the unit is disabled before the first manual start.

- [ ] **Step 5.11: Start the gateway manually**

Run apply manually, then verify. Connect a test client to enp1s0 or the private SSID.

- [ ] **Step 5.12: Test DHCP, DNS and egress**

From the client verify:

- address in 192.168.2.0/24;
- default gateway 192.168.2.1;
- DNS resolution;
- public HTTPS;
- public egress IP equals Darkstar upstream egress;
- client cannot reach Darkstar ports not explicitly allowed.

- [ ] **Step 5.13: Run a 60-minute stability test**

Record packet loss, reconnects and NetworkManager journal. For same-radio AP+STA, stress both upstream and downstream; any repeated disconnect forces the Ethernet/second-adapter fallback.

- [ ] **Step 5.14: Test stop and rollback**

Stop the gateway, confirm upstream remains connected, then restore the gateway. Verify the temporary management route survives.

- [ ] **Step 5.15: Enable and reboot-test gateway persistence**

Enable only after the manual test. Repeat the Task 3 boot_id procedure and verify both firewall and gateway after reboot.

- [ ] **Step 5.16: Commit the gateway module**

~~~bash
git add deploy/network deploy/firewall/host-guard.nft deploy/systemd/darkstar-gateway.service deploy/README.md
git commit -m "feat(darkstar): add native gateway and hotspot module"
~~~

**Exit gate:** A client receives a private address and reaches the Internet only through Darkstar; the gateway returns after a proven reboot.

---

## Task 6: Cut Windows over to Darkstar-only networking with rollback

**Entry gate:** Task 5 gateway survived a real reboot. Windows has both a tested Darkstar route and a documented local recovery method.

**Files:**

- Create: deploy/network/windows/Enable-DarkstarOnly.ps1
- Create: deploy/network/windows/Test-DarkstarOnly.ps1
- Create: deploy/network/windows/Restore-PreviousNetwork.ps1
- Create: deploy/network/windows/README.md
- Create after test: docs/operations/evidence/2026-08-29-windows-darkstar-only.md

- [ ] **Step 6.1: Inventory Windows adapters without mutation**

The test script records:

~~~powershell
Get-NetAdapter
Get-NetIPConfiguration
Get-NetRoute -DestinationPrefix 0.0.0.0/0
Get-NetRoute -DestinationPrefix ::/0
netsh wlan show interfaces
netsh wlan show profiles
~~~

It must redact saved Wi-Fi keys and never use netsh wlan show profile key=clear.

- [ ] **Step 6.2: Export a rollback snapshot**

Record adapter names, enabled state, current WLAN profile, interface metrics, DHCP state and default routes in a local JSON file under a user-selected backup directory outside Git.

- [ ] **Step 6.3: Implement Enable-DarkstarOnly**

The script:

1. requires an elevated PowerShell session;
2. verifies Darkstar gateway 192.168.2.1 is reachable;
3. identifies the downstream Ethernet or Darkstar SSID by exact configured name;
4. disables auto-connect or disconnects the home Wi-Fi profile;
5. does not delete saved Wi-Fi profiles;
6. ensures the Darkstar interface has the preferred metric;
7. checks that all default routes use the Darkstar interface;
8. stops and calls rollback if verification fails.

- [ ] **Step 6.4: Implement Test-DarkstarOnly**

Fail unless:

- exactly one usable IPv4 default route exists;
- its next hop is 192.168.2.1 or the selected Darkstar gateway;
- no active home Wi-Fi interface has Internet connectivity;
- Darkstar SSH/API private endpoints are reachable;
- a public HTTPS probe succeeds;
- DNS works.

- [ ] **Step 6.5: Implement Restore-PreviousNetwork**

Read the local JSON snapshot and restore:

- adapter enabled states;
- auto-connect/profile state;
- interface metrics;
- DHCP/static state when previously captured.

The script does not delete the Darkstar profile unless explicitly requested.

- [ ] **Step 6.6: Add Pester tests for pure validation functions**

Separate parsing/route-selection functions from mutating commands. Test:

- one correct Darkstar route passes;
- a second home route fails;
- wrong gateway fails;
- missing backup fails closed;
- rollback data round-trips.

- [ ] **Step 6.7: Perform the cutover**

Run Enable-DarkstarOnly from an elevated local Windows terminal. Immediately run Test-DarkstarOnly.

- [ ] **Step 6.8: Prove there is no bypass**

Temporarily stop only the Darkstar upstream while keeping the Windows-to-Darkstar link. Windows must lose public Internet and must not reconnect to the home SSID. Restore upstream and verify recovery.

- [ ] **Step 6.9: Reboot Windows and retest**

After a real Windows reboot, run Test-DarkstarOnly again. Confirm the home Wi-Fi has not auto-connected.

- [ ] **Step 6.10: Commit scripts and sanitized evidence**

~~~bash
git add deploy/network/windows docs/operations/evidence/2026-08-29-windows-darkstar-only.md
git commit -m "feat(darkstar): enforce Windows gateway-only connection"
~~~

**Exit gate:** Windows uses only Darkstar for its default route and fails closed when Darkstar upstream is unavailable.

---

## Task 7: Add typed gateway contracts to Darkstar Core

**Entry gate:** Native scripts and service behavior from Tasks 5–6 are stable.

**Files:**

- Create: crates/darkstar-core/src/network_gateway.rs
- Modify: crates/darkstar-core/src/lib.rs
- Modify: crates/darkstar-core/src/module_registry.rs
- Modify: crates/darkstar-core/src/system_graph.rs
- Test: crates/darkstar-core/src/network_gateway.rs

**Interfaces:**

- GatewayStatus is inspection data only.
- GatewayAction is a closed enum, never an arbitrary string command.
- A gateway mutation still uses existing AuthorizedModuleCommand.

- [ ] **Step 7.1: Write failing serialization tests**

Tests cover:

- GatewayMode Ethernet and Hotspot JSON round-trip;
- GatewayHealth states;
- upstream/downstream interface names;
- private CIDR;
- client count;
- last verification timestamp;
- no secret fields.

- [ ] **Step 7.2: Define typed status models**

Create:

~~~rust
pub enum GatewayMode {
    Ethernet,
    Hotspot,
}

pub enum GatewayHealth {
    Offline,
    Starting,
    Ready,
    Degraded,
    Failed,
}

pub struct GatewayStatus {
    pub mode: GatewayMode,
    pub health: GatewayHealth,
    pub upstream_interface: String,
    pub downstream_interface: String,
    pub downstream_cidr: String,
    pub connected_clients: u32,
    pub last_verified_unix_ms: u64,
}
~~~

Add documentation and required M. SZUL metadata above the module.

- [ ] **Step 7.3: Add validation**

Reject:

- empty or path-like interface names;
- upstream equal to downstream unless a preflight-approved flag is present;
- non-private downstream subnets;
- unspecified or loopback downstream addresses.

Do not implement a full network parser if an existing dependency can safely provide it; document any new crate.

- [ ] **Step 7.4: Register the gateway module**

Add darkstar-native-gateway to the existing module registry with capabilities:

- network.inspect;
- network.start;
- network.stop;
- network.restart.

- [ ] **Step 7.5: Expose the module in System Graph**

Create stable node identity and edges to:

- upstream Internet boundary;
- Windows/private client segment;
- Darkstar policy/provider;
- Headscale after Task 11.

- [ ] **Step 7.6: Run focused and workspace tests**

~~~bash
cargo test -p darkstar-core network_gateway
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
~~~

- [ ] **Step 7.7: Commit core contracts**

~~~bash
git add crates/darkstar-core
git commit -m "feat(darkstar): model native gateway state"
~~~

**Exit gate:** Gateway state is typed, serializable, validated and visible in the graph without executing host commands.

---

## Task 8: Implement a constrained Ubuntu gateway provider and HTTP boundary

**Entry gate:** Task 7 contracts are green. Host scripts from Task 5 accept no unvalidated user input.

**Files:**

- Create: crates/darkstar-server/src/gateway_provider.rs
- Create: crates/darkstar-server/src/network_gateway_http.rs
- Modify: crates/darkstar-server/src/http.rs
- Modify: crates/darkstar-server/src/main.rs only if provider construction belongs there
- Create: crates/darkstar-server/tests/network_gateway_http.rs

**Interfaces:**

- GET /v1/network/gateway returns authorized status.
- POST /v1/modules/darkstar-native-gateway/actions reuses the existing module action contract.
- Provider maps a closed command enum to fixed systemctl arguments.

- [ ] **Step 8.1: Write provider contract tests first**

Use a fake command runner. Tests prove:

- Start maps only to systemctl start darkstar-gateway.service;
- Stop maps only to systemctl stop darkstar-gateway.service;
- Restart maps only to systemctl restart darkstar-gateway.service;
- unsupported module ID is rejected;
- command failure becomes ProviderError::Unavailable or Rejected;
- no request field becomes an executable path or argument.

- [ ] **Step 8.2: Introduce a narrow CommandRunner trait**

The production implementation invokes a fixed executable with a fixed allowlist. The fake records calls. Keep this trait inside darkstar-server because OS execution is not a core policy responsibility.

- [ ] **Step 8.3: Implement GatewayProvider**

GatewayProvider implements ModuleProvider and accepts only authorized commands for darkstar-native-gateway.

- [ ] **Step 8.4: Add read-only status parsing**

Prefer a stable machine-readable output from darkstar-gateway-verify. Parse strict keys and return Degraded rather than inventing success when fields are missing.

- [ ] **Step 8.5: Write route authorization tests**

Cover:

- missing bearer token -> 401;
- unknown session -> 404;
- missing network.inspect -> 403;
- authorized inspect -> 200;
- mutation without capability -> 403;
- mutation needing approval -> explicit non-200;
- provider unavailable -> 503;
- success includes request_id/provider_id/resulting state.

- [ ] **Step 8.6: Wire routes into Axum**

Do not expose a generic /exec route. Reuse AppState or add a typed provider handle with Arc and a test fake.

- [ ] **Step 8.7: Emit event/audit records**

For each mutation record:

- request_id;
- principal/session;
- module ID;
- action;
- authorization result;
- provider ID;
- start/end/result.

- [ ] **Step 8.8: Run all server tests**

~~~bash
cargo test -p darkstar-server network_gateway
cargo test -p darkstar-server --test network_gateway_http
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
~~~

- [ ] **Step 8.9: Smoke-test against Ubuntu**

Use the real bearer/session flow. First inspect, then request one restart while a physical recovery path exists. Confirm the browser/API never receives sudo credentials.

- [ ] **Step 8.10: Commit provider/API**

~~~bash
git add crates/darkstar-server
git commit -m "feat(darkstar): control native gateway through policy"
~~~

**Exit gate:** The real gateway can be inspected and controlled only through authenticated, capability-gated requests.

---

## Task 9: Migrate the perimeter name from Sheriff to Warlock safely

**Entry gate:** Baseline tests are green. The migration can be isolated in one commit.

**Files:**

- Modify: crates/darkstar-core/src/network_topology.rs
- Modify: docs/DARKSTAR_NETWORK_ARCHITECTURE.md
- Modify: docs/DARKSTAR_REFERENCE_ARCHITECTURE.md
- Modify: any graph fixtures containing sheriff-bridge
- Test: crates/darkstar-core/src/network_topology.rs

- [ ] **Step 9.1: Write backward-compatibility tests**

Tests prove:

- new serialization emits warlock_bridge;
- old sheriff_bridge JSON deserializes to WarlockBridge;
- reference topology uses node_id warlock-bridge;
- a legacy node alias resolves to the same logical node during migration.

- [ ] **Step 9.2: Rename the enum with a serde alias**

Use a canonical WarlockBridge variant with serde rename warlock_bridge and alias sheriff_bridge.

- [ ] **Step 9.3: Update topology IDs and labels**

Change the current reference output to:

- node_id: warlock-bridge;
- name: Warlock Bridge;
- visible identity: warlock-egress;
- required gates using Warlock wording.

- [ ] **Step 9.4: Keep an explicit legacy alias**

Add a small resolver from sheriff-bridge to warlock-bridge for stored events/config references. Do not keep two live perimeter nodes.

- [ ] **Step 9.5: Update documentation and UI fixtures**

Replace future-facing Sheriff wording with Warlock. Add one migration note explaining the historical name.

- [ ] **Step 9.6: Run topology and workspace tests**

~~~bash
cargo test -p darkstar-core network_topology
cargo test --workspace
~~~

- [ ] **Step 9.7: Commit the migration**

~~~bash
git add crates/darkstar-core/src/network_topology.rs docs
git commit -m "refactor(darkstar): rename perimeter bridge to Warlock"
~~~

**Exit gate:** New output says Warlock while old serialized Sheriff values remain readable.

---

## Task 10: Replace the Vite starter with the Control Room React foundation

**Entry gate:** Task 0 frontend build is green and the existing embedded Rust UI remains available as fallback.

**Files:**

- Modify: frontend/package.json
- Modify: frontend/eslint.config.js
- Modify: frontend/src/App.tsx
- Modify: frontend/src/App.css
- Modify: frontend/src/index.css
- Create: frontend/src/api/darkstarClient.ts
- Create: frontend/src/api/types.ts
- Create: frontend/src/auth/AuthContext.tsx
- Create: frontend/src/components/AppShell.tsx
- Create: frontend/src/features/system-graph/SystemGraph.tsx
- Create: frontend/src/features/live-run/LiveTimeline.tsx
- Create: frontend/src/features/modules/ModuleInspector.tsx
- Create: frontend/src/test/setup.ts
- Create: frontend/src/**/*.test.tsx as listed below

- [ ] **Step 10.1: Add test tooling through package manager**

Add Vitest, jsdom, Testing Library and user-event using npm so package-lock.json is updated reproducibly. Do not hand-edit resolved lock entries.

- [ ] **Step 10.2: Configure type-aware lint and tests**

Adopt typescript-eslint recommendedTypeChecked or strictTypeChecked with tsconfigRootDir. Add scripts:

~~~json
"typecheck": "tsc -b --pretty false",
"test": "vitest run",
"test:watch": "vitest"
~~~

- [ ] **Step 10.3: Write AppShell tests first**

Test that the shell renders:

- Darkstar identity/status;
- System Graph and Live Run tabs;
- connection/auth state;
- inspector region;
- timeline region;
- no Vite/React demo logos or counter.

- [ ] **Step 10.4: Define API types matching Rust**

Types include:

- SystemGraph;
- node/edge;
- module state;
- GatewayStatus;
- run event;
- typed error response.

Use exact snake_case wire fields and map them explicitly when presentation uses camelCase.

- [ ] **Step 10.5: Implement a single API client**

The client:

- takes base URL from build/runtime configuration;
- accepts an in-memory bearer token;
- never puts token in query string;
- never stores token in localStorage;
- handles non-JSON and typed JSON errors;
- supports AbortSignal;
- exposes graph, gateway status, session and module action methods.

- [ ] **Step 10.6: Implement AuthContext**

Keep the initial token only in memory. Provide login/token entry, logout and expired/unauthorized handling. Do not log the token.

- [ ] **Step 10.7: Replace starter UI**

Remove Vite starter assets/usages and render the four-region Control Room. Preserve keyboard focus and responsive behavior.

- [ ] **Step 10.8: Add loading, empty, error and disconnected states**

Tests must cover each state. An empty graph is not an error. A 401 returns to the auth state.

- [ ] **Step 10.9: Add required metadata headers**

All created/modified project-owned TS, TSX, CSS, JS, HTML and Markdown files must include a valid top comment with AUTHOR: M. SZUL. Preserve package-lock format; do not inject comments into JSON.

- [ ] **Step 10.10: Run frontend gates**

~~~bash
npm run lint
npm run typecheck
npm run test
npm run build
~~~

- [ ] **Step 10.11: Commit the React foundation**

~~~bash
git add frontend
git commit -m "feat(darkstar-ui): replace Vite starter with Control Room shell"
~~~

**Exit gate:** No starter content remains, API/auth boundaries are tested and the production build succeeds.

---

## Task 11: Implement the n8n-like graph, gateway controls and live timeline

**Entry gate:** React foundation and API client from Task 10 are green.

**Files:**

- Modify: frontend/src/features/system-graph/SystemGraph.tsx
- Create: frontend/src/features/system-graph/DarkstarNode.tsx
- Create: frontend/src/features/system-graph/layout.ts
- Modify: frontend/src/features/live-run/LiveTimeline.tsx
- Modify: frontend/src/features/modules/ModuleInspector.tsx
- Create: frontend/src/features/gateway/GatewayPanel.tsx
- Create: frontend/src/features/gateway/GatewayActions.tsx
- Create tests next to each feature

- [ ] **Step 11.1: Write graph mapping tests**

Given a Rust SystemGraph fixture, assert:

- stable node IDs;
- correct React Flow node types;
- preserved edges;
- Warlock legacy alias maps to canonical ID;
- gateway node has the expected status badge.

- [ ] **Step 11.2: Implement stable layout**

Keep positions stable for unchanged node IDs. Use a deterministic initial layout, not random coordinates. Persist only non-sensitive presentation state.

- [ ] **Step 11.3: Implement Darkstar node cards**

Show:

- name;
- kind;
- state/health;
- provider/capability badges when present;
- selected/focus/running/failed/blocked visual states.

- [ ] **Step 11.4: Implement node selection and inspector**

Selecting a node displays identity, dependencies, capabilities, health, provenance and allowed actions. The server response remains authoritative for action availability.

- [ ] **Step 11.5: Write action-state tests**

Test:

- missing capability hides or disables mutation;
- authorized Start/Stop/Restart sends exact module request;
- NeedsApproval is visibly distinct;
- 403, 409, 503 and network failure have clear messages;
- double-click/repeated submission does not send duplicate commands.

- [ ] **Step 11.6: Implement GatewayPanel**

Show upstream/downstream, mode, subnet, connected clients, last verification and health. Do not show Wi-Fi PSK, token or private key.

- [ ] **Step 11.7: Implement SSE timeline**

Parse the existing event stream, associate events with run/node IDs, reconnect with bounded backoff and stop reconnection after logout/abort.

- [ ] **Step 11.8: Highlight live execution**

Update only affected nodes/edges. Distinguish requested, authorized, running, completed, failed, blocked and approval-required states.

- [ ] **Step 11.9: Add keyboard and accessibility tests**

Verify:

- graph controls have accessible labels;
- inspector is reachable by keyboard;
- focus does not disappear when live events update;
- status is not conveyed by color alone;
- action confirmation describes the target and effect.

- [ ] **Step 11.10: Run frontend gates and browser smoke test**

~~~bash
npm run lint
npm run typecheck
npm run test
npm run build
~~~

Open the built app against the real local Darkstar API in Chromium and verify auth, graph, gateway status, one authorized action and live timeline.

- [ ] **Step 11.11: Commit feature UI**

~~~bash
git add frontend
git commit -m "feat(darkstar-ui): add live gateway control graph"
~~~

**Exit gate:** Operator can see and control the native gateway through the same authenticated visual flow, with live events and no direct infrastructure access.

---

## Task 12: Serve the React production build through Darkstar

**Entry gate:** Task 11 production build succeeds and the embedded Rust view remains a working fallback.

**Files:**

- Modify: crates/darkstar-server/Cargo.toml
- Modify: crates/darkstar-server/src/http.rs
- Modify: Dockerfile
- Modify: .dockerignore
- Modify: deploy/docker-compose.yml
- Modify: deploy/README.md
- Test: crates/darkstar-server route tests

- [ ] **Step 12.1: Write route tests first**

Test:

- /control-room returns the React index when build assets exist;
- static assets have correct content types;
- unknown client-side route falls back to index only under /control-room;
- /system-graph still serves the embedded fallback;
- protected APIs remain protected;
- static routing cannot read files outside the asset directory.

- [ ] **Step 12.2: Add a constrained static-file service**

Use tower-http ServeDir/ServeFile or the smallest compatible Rust solution. Do not create a generic filesystem browser.

- [ ] **Step 12.3: Build frontend in a Docker build stage**

The Dockerfile:

1. uses a pinned Node base image;
2. runs npm ci;
3. runs typecheck/test/build or uses a separate CI gate for tests;
4. copies only frontend/dist into the final image;
5. keeps the final Rust runtime non-root;
6. does not contain npm cache or source secrets.

- [ ] **Step 12.4: Keep local development simple**

Document two terminals:

~~~bash
cargo run -p darkstar-server
cd frontend
npm run dev
~~~

Vite dev proxy targets the local Darkstar API without weakening production CORS.

- [ ] **Step 12.5: Run complete build verification**

~~~bash
cargo test --workspace
cd frontend
npm run lint
npm run typecheck
npm run test
npm run build
cd ..
docker build -t darkstar:dev .
~~~

- [ ] **Step 12.6: Smoke-test the container**

Start Compose, verify /health, /control-room, an asset, protected API 401 without token and successful API access with a valid session.

- [ ] **Step 12.7: Commit serving integration**

~~~bash
git add crates/darkstar-server Dockerfile .dockerignore deploy frontend/vite.config.ts
git commit -m "feat(darkstar): serve the React Control Room"
~~~

**Exit gate:** One Darkstar deployment serves API and production Control Room while retaining the compatibility fallback.

---

## Task 13: Deploy a pinned, self-hosted Headscale module

**Entry gate:** Native Darkstar-only network from Task 6 is stable. Tailscale SaaS fallback still exists for recovery.

**Files:**

- Create: deploy/headscale/compose.yml
- Create: deploy/headscale/config.yaml.example
- Create: deploy/headscale/policy.hujson
- Create: deploy/headscale/version.lock
- Create: deploy/headscale/backup
- Create: deploy/headscale/restore
- Create: deploy/headscale/verify
- Create: deploy/headscale/README.md
- Create: deploy/systemd/darkstar-headscale.service
- Modify: deploy/docker-compose.yml only if using one combined project is explicitly chosen
- Modify: deploy/README.md

**Upstream:**

- Repository: https://github.com/juanfont/headscale
- License: BSD-3-Clause

- [ ] **Step 13.1: Review upstream before selecting a version**

Read upstream README, LICENSE, release notes, configuration example, policy documentation, database migration notes and container instructions. Record the selected immutable release tag and commit SHA.

- [ ] **Step 13.2: Resolve and record an immutable image**

Resolve the official image digest for the selected release. version.lock records:

- upstream repository;
- license;
- release tag;
- commit SHA;
- image name;
- sha256 digest;
- verification date;
- update procedure.

Do not use latest.

- [ ] **Step 13.3: Write failing configuration checks**

Before adding files:

~~~bash
test -f deploy/headscale/compose.yml
test -f deploy/headscale/config.yaml.example
test -f deploy/headscale/policy.hujson
~~~

- [ ] **Step 13.4: Create a private-first configuration**

Configure:

- SQLite initially unless measured scale requires PostgreSQL;
- data under a named persistent volume/root-owned host directory;
- server URL reachable from the Darkstar private network;
- metrics/debug bound to localhost or a private admin interface;
- no secrets in Git;
- random private keys generated on host;
- DNS settings appropriate for the private mesh;
- policy mode supported by the selected version.

- [ ] **Step 13.5: Define least-privilege policy**

Initial policy permits:

- Windows operator device to reach Darkstar SSH and Control Room private ports;
- Darkstar to reach registered test clients only where required;
- no public ingress;
- no broad any-to-any family access.

Add policy validation tests/commands supplied by the selected Headscale version.

- [ ] **Step 13.6: Add backup and restore scripts**

Backup:

- stops or quiesces writes according to upstream guidance;
- captures database and required private state;
- writes checksum and metadata;
- never commits the archive.

Restore:

- refuses to overwrite a running instance;
- validates checksum;
- restores into an explicit destination;
- preserves ownership/modes.

- [ ] **Step 13.7: Add a read-only verify script**

Verify:

- service health;
- expected version;
- database reachable;
- private bind;
- policy loaded;
- no public listener;
- no connection to commercial login server.

- [ ] **Step 13.8: Add systemd lifecycle**

darkstar-headscale.service delegates to the dedicated Compose definition, waits for Docker and native gateway, and has bounded restart behavior.

- [ ] **Step 13.9: Start Headscale locally**

Create host directories/secrets with restrictive permissions, start the service and run verify.

- [ ] **Step 13.10: Create a non-production namespace and preauth key**

Use a short-lived, reusable=false preauth key for the first test client. Do not print it into the committed journal.

- [ ] **Step 13.11: Register an isolated test client**

Before moving Windows, register a disposable Linux client/container or VM configured with the private Headscale login server. Verify node listing, key expiry, policy and direct local connectivity.

- [ ] **Step 13.12: Inspect relay dependence**

Run client network diagnostics. Record whether traffic is direct or uses DERP. Do not claim full independence if a public DERP map/relay is used.

- [ ] **Step 13.13: Perform backup/restore drill**

Create a backup, restore into a separate test data directory, start an isolated verification instance on a different local port, verify node metadata, then stop it. Remove only artifacts created by this drill after recording checksums.

- [ ] **Step 13.14: Add Headscale to Darkstar module registry**

Expose health/state in System Graph. Mutating actions use the existing policy/provider path and fixed systemd service name.

- [ ] **Step 13.15: Commit Headscale module**

~~~bash
git add deploy/headscale deploy/systemd/darkstar-headscale.service deploy/README.md crates
git commit -m "feat(darkstar): add self-hosted Headscale module"
~~~

**Exit gate:** A pinned private Headscale instance manages an isolated test client, survives restart and has a proven backup/restore path.

---

## Task 14: Deploy Headplane and integrate it with Control Room

**Entry gate:** Headscale Task 13 is healthy and its API access model is understood.

**Files:**

- Create: deploy/headplane/compose.yml
- Create: deploy/headplane/config.yaml.example
- Create: deploy/headplane/version.lock
- Create: deploy/headplane/verify
- Create: deploy/headplane/README.md
- Create: deploy/systemd/darkstar-headplane.service
- Create: frontend/src/features/mesh/MeshPanel.tsx
- Create: frontend/src/features/mesh/MeshPanel.test.tsx
- Modify: frontend/src/App.tsx
- Modify: crates/darkstar-core/src/system_graph.rs

**Upstream:**

- Repository: https://github.com/tale/headplane
- License: MIT

- [ ] **Step 14.1: Review upstream**

Read README, LICENSE, release notes, authentication model, Headscale version matrix, filesystem/API permissions and container deployment guide.

- [ ] **Step 14.2: Pin an immutable compatible release**

version.lock records upstream URL, MIT license, tag, commit SHA, image digest, compatible Headscale version and update procedure. Do not use latest.

- [ ] **Step 14.3: Write failing private-bind checks**

Before deployment, define a test that fails if Headplane listens on a public/WAN address or if its port is reachable from the upstream Wi-Fi segment.

- [ ] **Step 14.4: Create least-privilege configuration**

Headplane receives only the Headscale access/config it requires. Secrets are mounted from root-readable local files or Docker secrets, never baked into the image or sent to the browser.

- [ ] **Step 14.5: Configure private access**

Bind Headplane to localhost or 192.168.2.1 only. Protect it with its supported authentication. Firewall permits access only from the Darkstar private client subnet.

- [ ] **Step 14.6: Add systemd and verify script**

Verify:

- pinned version;
- Headscale API connection;
- authenticated UI;
- private listener;
- denial from upstream/public segment;
- no token in HTML/JS response or logs.

- [ ] **Step 14.7: Add a System Graph node**

Headplane is a UI/provider-support module connected to Headscale. It is not a policy authority.

- [ ] **Step 14.8: Implement MeshPanel in Control Room**

Show read-only summary from a Darkstar adapter:

- Headscale health/version;
- number of nodes;
- online/offline counts;
- key-expiry warnings;
- Headplane availability.

Do not expose the Headscale API token.

- [ ] **Step 14.9: Add safe navigation**

Initially open Headplane through a private authenticated URL in a separate tab. Use iframe only if upstream headers, same-origin isolation and token handling are proven safe. Do not weaken CSP merely to embed it.

- [ ] **Step 14.10: Test permission boundaries**

Confirm:

- unauthenticated browser is denied;
- normal Control Room session gets only allowed summary;
- admin Headplane functions require separate authorized role;
- Darkstar actions still pass policy.

- [ ] **Step 14.11: Run all gates**

~~~bash
cargo test --workspace
cd frontend
npm run lint
npm run typecheck
npm run test
npm run build
cd ..
docker compose -f deploy/headplane/compose.yml config
~~~

- [ ] **Step 14.12: Commit Headplane integration**

~~~bash
git add deploy/headplane deploy/systemd/darkstar-headplane.service frontend crates
git commit -m "feat(darkstar): integrate Headplane private mesh UI"
~~~

**Exit gate:** Headplane is private, authenticated, pinned and visible through a safe Control Room integration without leaking admin credentials.

---

## Task 15: Move Windows to the self-hosted control server and retire SaaS dependence

**Entry gate:** Tasks 3, 5, 6, 13 and 14 pass. Native Windows-to-Darkstar network works independently of Tailscale. Operator has physical access.

**Files:**

- Create: deploy/headscale/windows/Export-CurrentMeshState.ps1
- Create: deploy/headscale/windows/Join-DarkstarMesh.ps1
- Create: deploy/headscale/windows/Restore-PreviousMesh.ps1
- Create: deploy/headscale/windows/Test-DarkstarMesh.ps1
- Create: docs/operations/evidence/2026-08-29-headscale-cutover.md
- Modify: deploy/headscale/README.md

- [ ] **Step 15.1: Preserve the current recovery state**

Export sanitized client version, preferences, current control server identity, routes and node name. Never export private node keys into Git.

- [ ] **Step 15.2: Test rollback on a disposable client**

Prove that a client can leave the test Headscale instance and rejoin its previous control server using documented commands before changing Windows.

- [ ] **Step 15.3: Create a short-lived Windows enrollment**

Generate a single-use, short-expiry preauth key immediately before use. Pass it securely to the local script without command history or logs.

- [ ] **Step 15.4: Join Windows to Darkstar Headscale**

Configure the existing audited open-source client to use only the private Headscale login server. Confirm the UI/CLI reports the expected server and node identity.

- [ ] **Step 15.5: Run mesh acceptance tests**

Verify:

- Windows -> Darkstar private address;
- Darkstar -> Windows only on allowed ports;
- Control Room and Headplane access;
- DNS if enabled;
- ACL denial for a forbidden target;
- no request to the commercial login server;
- direct local data path where possible.

- [ ] **Step 15.6: Reboot both endpoints separately**

First reboot Windows and verify rejoin. Then perform a separate real Darkstar reboot with boot_id proof and verify Headscale, Headplane, firewall, gateway and client reconnection.

- [ ] **Step 15.7: Observe for 24 hours**

Record disconnects, key warnings, relay usage and service restarts. Do not remove the rollback during this observation window.

- [ ] **Step 15.8: Disable Tailscale SaaS autostart on Darkstar**

Only after the observation window:

- record current service state;
- disable/stop the SaaS-configured instance;
- do not uninstall packages yet;
- verify native SSH, gateway, Headscale and Headplane immediately.

- [ ] **Step 15.9: Prove independence**

Acceptance requires:

- Windows remains connected to Darkstar without the SaaS control plane;
- no Serveo process/service;
- local Headscale resolves and manages nodes;
- critical UI/API operations work;
- public egress still goes through Darkstar;
- rollback can restore the previous state if needed.

- [ ] **Step 15.10: Decide separately about package removal**

Do not uninstall the Tailscale client merely because SaaS is disabled. The open-source client may remain the Headscale data-plane client. Document the exact remaining dependency and license.

- [ ] **Step 15.11: Commit sanitized cutover evidence**

~~~bash
git add deploy/headscale/windows deploy/headscale/README.md docs/operations/evidence/2026-08-29-headscale-cutover.md
git commit -m "docs(darkstar): prove private mesh cutover"
~~~

**Exit gate:** No critical route or control function depends on Tailscale SaaS, while the audited open-source client dependency is explicitly documented.

---

## Task 16: Final verification, documentation and release checkpoint

**Entry gate:** All prior exit gates pass.

**Files:**

- Modify: docs/DARKSTAR_ROADMAP.md
- Modify: docs/DARKSTAR_NETWORK_ARCHITECTURE.md
- Modify: docs/DARKSTAR_REFERENCE_ARCHITECTURE.md
- Modify: docs/superpowers/specs/2026-08-28-darkstar-control-room-design.md
- Modify: docs/superpowers/plans/2026-08-28-darkstar-control-room.md
- Modify: docs/superpowers/plans/2026-08-28-darkstar-runtime-supervision-plan.md
- Modify: deploy/README.md
- Create: docs/operations/DARKSTAR_OPERATOR_RUNBOOK.md
- Create: docs/operations/evidence/2026-08-29-darkstar-gateway-mesh-acceptance.md

- [ ] **Step 16.1: Run source header audit**

List project-owned new source/docs files without AUTHOR: M. SZUL. Review exceptions:

- JSON and lock files that cannot legally contain comments;
- binary assets;
- preserved third-party source with upstream authorship.

No unexplained project-owned exception is allowed.

- [ ] **Step 16.2: Run Rust gates**

~~~bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo build --release -p darkstar-server
~~~

- [ ] **Step 16.3: Run frontend gates**

~~~bash
cd frontend
npm ci
npm run lint
npm run typecheck
npm run test
npm run build
~~~

- [ ] **Step 16.4: Validate deployment definitions**

~~~bash
docker compose -f deploy/docker-compose.yml config
docker compose -f deploy/headscale/compose.yml config
docker compose -f deploy/headplane/compose.yml config
systemd-analyze verify deploy/systemd/*.service deploy/systemd/*.timer
sudo nft -c -f deploy/firewall/host-guard.nft
~~~

- [ ] **Step 16.5: Run end-to-end acceptance**

From Windows:

1. connect only to Darkstar downstream;
2. authenticate to Control Room;
3. inspect firewall/gateway/Headscale/Headplane;
4. observe System Graph;
5. run one authorized gateway restart;
6. observe events and resulting Ready state;
7. verify public egress;
8. verify a forbidden mesh route is denied.

- [ ] **Step 16.6: Run failure drills**

Separately test:

- Darkstar application container restart;
- Headplane unavailable while Headscale remains healthy;
- Headscale restart and client recovery;
- gateway stop with Windows fail-closed;
- incorrect firewall candidate rejected by nft -c;
- expired client enrollment key.

Restore healthy state after each single drill before starting the next.

- [ ] **Step 16.7: Run final real reboot**

Use the boot_id procedure. After reboot verify:

- firewall active;
- gateway active;
- Darkstar active and healthy;
- Headscale active and healthy;
- Headplane active and private;
- dependency timer active;
- Windows route through Darkstar;
- Windows Headscale client reconnected.

- [ ] **Step 16.8: Write the operator runbook**

Include one-command checks for:

- health;
- service status;
- logs;
- connected clients;
- route/firewall status;
- backup;
- restore;
- emergency rollback;
- physical recovery;
- version/digest inventory.

- [ ] **Step 16.9: Update roadmap status using evidence only**

Mark a milestone DONE only when the evidence file contains the corresponding successful commands. Leave future Warlock/Kali, home cloud and PowerEdge/AIONS stages as PLANNED/BLOCKED.

- [ ] **Step 16.10: Check repository hygiene**

~~~bash
git status --short
git diff --check
git ls-files | grep -E '(^|/)(dist|target|node_modules|backup|secrets?)(/|$)'
~~~

Review every match; do not delete unknown files. Remove only artifacts created by this implementation after exact path validation.

- [ ] **Step 16.11: Commit the final checkpoint**

~~~bash
git add docs deploy crates frontend Dockerfile .dockerignore
git commit -m "feat(darkstar): complete native gateway and private mesh foundation"
~~~

- [ ] **Step 16.12: Push after final remote check**

~~~bash
git fetch origin feat/darkstar-module-control
git merge-base --is-ancestor origin/feat/darkstar-module-control HEAD
git push origin HEAD:feat/darkstar-module-control
~~~

Never force-push. If the ancestor check fails, stop and review the remote commits.

**Exit gate:** The source, deployment, real machines and documentation all agree; the final commit is pushed without force and its SHA is recorded.

---

## Deferred follow-up plans

These are roadmap items, not hidden work inside this implementation:

1. Warlock public perimeter, private DNS/TLS and self-hosted DERP/relay.
2. Kali Bridge as an isolated security capability host.
3. Forgejo, CI runners, artifact/container registry and backup for the home development cloud.
4. PowerEdge preparation, storage, AIONS migration and local model/GPU serving.

Each requires a separate design specification, threat review, implementation plan and rollback.

## Final definition of done

The plan is complete only when:

- all Task 0–16 exit gates and the Task 0A SSH gate are met;
- Rust and frontend quality gates pass;
- real reboot evidence contains changed boot IDs;
- Windows has no active Internet bypass around Darkstar;
- firewall/gateway/Headscale/Headplane return after reboot;
- Control Room shows real state and sends only policy-gated actions;
- Tailscale SaaS is not required for critical operation;
- open-source client/control/UI dependencies and licenses are explicit;
- Serveo is inactive;
- no secret or generated build debris is committed;
- roadmap and operator runbook match the actual system;
- all new project-owned files follow the AUTHOR: M. SZUL rule.
