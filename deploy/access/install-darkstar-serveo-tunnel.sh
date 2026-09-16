#!/usr/bin/env bash
# THIS IS VERY IMPORTANT!!!
# ==========================================
# AUTHOR: M. SZUL
# AI MODEL: GPT-5.6 Sol
# TIMESTAMP: 2026-08-29 03:16:00 Europe/London
# REASON FOR CREATION: Install and start the persistent Darkstar Serveo management tunnel with one command on Ubuntu.
# MECHANICS: Validates required binaries/local sshd, installs the tracked systemd unit, enables it at boot, starts it, and prints status.
# SYSTEM PART: Darkstar management access
# ARCHITECTURE FUNCTION: Bootstrap the administrative tunnel only; it does not change default Internet routing.
# DEPENDENCIES/LINKS: deploy/access/darkstar-serveo-tunnel.service, systemd, OpenSSH.
# TECH STACK: Bash + systemd.
# LOCAL WORKSPACE: /home/owner/polip-agi
# GIT COMMIT: PENDING
# GITHUB METADATA: jpytka666-jpg/polip-agi / feat/darkstar-gateway
# ==========================================

set -euo pipefail

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)"
UNIT_SRC="$ROOT_DIR/deploy/access/darkstar-serveo-tunnel.service"
UNIT_DST="/etc/systemd/system/darkstar-serveo-tunnel.service"

command -v ssh >/dev/null || { echo "ERROR: OpenSSH client (ssh) is not installed." >&2; exit 1; }
command -v systemctl >/dev/null || { echo "ERROR: systemd/systemctl is required." >&2; exit 1; }
[[ -f "$UNIT_SRC" ]] || { echo "ERROR: missing $UNIT_SRC" >&2; exit 1; }

if ! ss -ltn 2>/dev/null | grep -Eq 'LISTEN.+:22([[:space:]]|$)'; then
  echo "ERROR: no local SSH listener detected on TCP/22. Start openssh-server first." >&2
  exit 1
fi

sudo install -m 0644 "$UNIT_SRC" "$UNIT_DST"
sudo systemctl daemon-reload
sudo systemctl enable --now darkstar-serveo-tunnel.service
sudo systemctl status darkstar-serveo-tunnel.service --no-pager

echo
echo "Darkstar management tunnel installed."
echo "Windows entry command: ssh -J serveo.net owner@ds-cbms-8248"
