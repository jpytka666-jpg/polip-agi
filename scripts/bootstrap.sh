#!/usr/bin/env bash
# ================================================================================
# AIONS ARCHITECTURE - PROVENANCE / READ-ONLY CONTEXT HEADER
# ================================================================================
# AUTHOR:              Claude Code agent, working for Marcin (jpytka666-jpg)
# AI MODEL:            Anthropic Claude via Claude Code remote session (exact model id WITHHELD by harness policy)
# TIMESTAMP:           2026-09-25T23:14:45Z
# REASON FOR CREATION: Protocol deliverable 6: a bootstrap for this workspace that is idempotent and uses no Python.
# MECHANICS:           Checks the toolchain, builds aions-gate, points git at hooks/, validates the gate config and runs the test suite. Each step prints PASS/FAIL. Re-running changes nothing that is already in place.
# SYSTEM PART:         polip-agi / bootstrap
# ARCHITECTURE FUNC:   Execution plane: puts a fresh clone into the enforced state with one command (SCCE rule 18).
# DEPENDENCIES/LINKS:  cargo, git, hooks/pre-commit, aions-gates.conf
# TECH STACK:          bash
# LOCAL WORKSPACE:     polip-agi/scripts
# GIT COMMIT:          PENDING
# GITHUB METADATA:     jpytka666-jpg/polip-agi, branch claude/autonomous-automation-agent-ocxwke
# ================================================================================

set -euo pipefail
root=$(git -C "$(dirname "${BASH_SOURCE[0]}")" rev-parse --show-toplevel)
cd "$root"

step() { printf '%-28s' "$1"; shift; if "$@" >/tmp/aions-bootstrap.$$ 2>&1; then echo PASS; else echo FAIL; cat /tmp/aions-bootstrap.$$; rm -f /tmp/aions-bootstrap.$$; exit 1; fi; rm -f /tmp/aions-bootstrap.$$; }

step "toolchain (cargo, git)"   sh -c 'command -v cargo && command -v git'
step "build aions-gate"         cargo build -q -p aions-gate
current=$(git config --get core.hooksPath || true)
if [ "$current" != "hooks" ]; then
  step "install hooks path"     git config core.hooksPath hooks
else
  printf '%-28s%s\n' "install hooks path" "PASS (already set)"
fi
step "hook executable"          test -x hooks/pre-commit
step "gate config valid"        target/debug/aions-gate check-config
step "test suite"               cargo test -q --workspace
echo "bootstrap: PASS"
