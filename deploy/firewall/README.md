<!--
THIS IS VERY IMPORTANT!!!
==========================================
AUTHOR: M. SZUL
AI MODEL: GPT-5 Codex
TIMESTAMP: 2026-08-29 00:00:00
REASON FOR CREATION: Instrukcja instalacji, weryfikacji i rollbacku host guard Darkstar.
==========================================
-->

# Darkstar host firewall

`host-guard.nft` owns only `table inet darkstar_host_guard`. It permits
loopback, established traffic, ICMP/ICMPv6, temporary management on
`tailscale0`, UDP `41641` on `wlp2s0`, DHCP client responses, and downstream
SSH only from `10.44.0.0/24` through `enp1s0`. Forwarding and IPv4 masquerade
are restricted to `10.44.0.0/24` from `enp1s0` to `wlp2s0`. It deliberately
does not open the Darkstar API to public input.

## Install

Perform this only with physical access to Darkstar and a working recovery
connection. Before changing any installed file, create timestamped root-owned
backups of exactly the three existing files:

```sh
sudo install -d -o root -g root -m 0700 /var/backups/darkstar-firewall
backup=/var/backups/darkstar-firewall/$(date -u +%Y%m%dT%H%M%SZ)
sudo install -d -o root -g root -m 0700 "$backup"
sudo cp -p /etc/darkstar/host-guard.nft "$backup/host-guard.nft"
sudo cp -p /usr/local/sbin/darkstar-firewall-apply "$backup/darkstar-firewall-apply"
sudo cp -p /etc/systemd/system/darkstar-firewall.service "$backup/darkstar-firewall.service"
sudo chown -R root:root "$backup"
sudo chmod 0700 "$backup"
```

Copy the versioned files to their exact destinations and set their modes:

```sh
sudo install -d -o root -g root -m 0755 /etc/darkstar
sudo install -o root -g root -m 0644 deploy/firewall/host-guard.nft /etc/darkstar/host-guard.nft
sudo install -o root -g root -m 0755 deploy/firewall/darkstar-firewall-apply /usr/local/sbin/darkstar-firewall-apply
sudo install -o root -g root -m 0755 deploy/firewall/darkstar-firewall-verify /usr/local/sbin/darkstar-firewall-verify
sudo install -o root -g root -m 0644 deploy/systemd/darkstar-firewall.service /etc/systemd/system/darkstar-firewall.service
sudo systemctl daemon-reload
```

Validate before starting; do not apply a failed candidate:

```sh
sudo nft -c -f deploy/firewall/host-guard.nft
systemd-analyze verify deploy/systemd/darkstar-firewall.service
shellcheck deploy/firewall/darkstar-firewall-apply deploy/firewall/darkstar-firewall-verify
sh -n deploy/firewall/darkstar-firewall-apply
sh -n deploy/firewall/darkstar-firewall-verify
```

After manual verification is planned, start and inspect the unit. Do not add
the verify script as `ExecStartPost` until it has been proven safe on the host.

```sh
sudo systemctl start darkstar-firewall.service
systemctl status darkstar-firewall.service --no-pager --full
sudo /usr/local/sbin/darkstar-firewall-verify
sudo nft list table inet darkstar_host_guard
journalctl -u darkstar-firewall.service --no-pager
```

Enable only after the manual start, route and recovery checks succeed:

```sh
sudo systemctl enable darkstar-firewall.service
```

## Rollback

Keep physical access available. Stop the unit, restore the three files from a
specific timestamped backup, reload systemd, and start the restored unit:

```sh
backup=/var/backups/darkstar-firewall/REPLACE_WITH_TIMESTAMP
sudo test -f "$backup/host-guard.nft"
sudo test -f "$backup/darkstar-firewall-apply"
sudo test -f "$backup/darkstar-firewall.service"
sudo systemctl stop darkstar-firewall.service
sudo install -o root -g root -m 0644 "$backup/host-guard.nft" /etc/darkstar/host-guard.nft
sudo install -o root -g root -m 0755 "$backup/darkstar-firewall-apply" /usr/local/sbin/darkstar-firewall-apply
sudo install -o root -g root -m 0644 "$backup/darkstar-firewall.service" /etc/systemd/system/darkstar-firewall.service
sudo systemctl daemon-reload
sudo systemctl start darkstar-firewall.service
```

Do not remove unrelated nftables tables during installation or rollback.
