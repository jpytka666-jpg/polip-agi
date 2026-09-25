<!--
================================================================================
AIONS ARCHITECTURE - PROVENANCE / READ-ONLY CONTEXT HEADER
================================================================================
AUTHOR:              Claude Code agent, working for Marcin (jpytka666-jpg)
AI MODEL:            Anthropic Claude via Claude Code remote session (exact model id WITHHELD by harness policy)
TIMESTAMP:           2026-09-25T23:14:45Z
REASON FOR CREATION: Operator documentation for aions-gate (AIONS protocol deliverables 3, 4, 5, 6).
MECHANICS:           Explains commands, exit codes, config grammar and what is and is not enforced yet.
SYSTEM PART:         polip-agi / aions-gate
ARCHITECTURE FUNC:   Front door for the enforcement-plane tool.
DEPENDENCIES/LINKS:  src/, tests/, ../../aions-gates.conf, ../../hooks/pre-commit, ../../scripts/bootstrap.sh, ../../.github/workflows/aions-gate.yml
TECH STACK:          Markdown
LOCAL WORKSPACE:     polip-agi/crates/aions-gate
GIT COMMIT:          PENDING
GITHUB METADATA:     jpytka666-jpg/polip-agi, branch claude/autonomous-automation-agent-ocxwke
================================================================================
-->

# aions-gate

Enforcement plane for the AIONS workspace. The model asks for a transition;
aions-gate decides it from command results. Rust, std only, no Python.

## One command to re-establish truth

    scripts/bootstrap.sh

It builds the tool, sets `core.hooksPath=hooks`, validates the config and
runs the tests. It is idempotent: re-running it changes nothing that is
already in place.

## Commands

| Command | What it does |
|---|---|
| `aions-gate provenance` | Checks every tracked file |
| `aions-gate provenance --since origin/main` | New files must carry a valid header. Edited files must keep their creation fields (AUTHOR, AI MODEL, TIMESTAMP) |
| `aions-gate provenance --staged` | Same check, on index content (used by the pre-commit hook) |
| `aions-gate provenance PATH...` | Checks the given files or directories |
| `aions-gate state init <task>` / `show` / `rewind <STATE>` | Task state. Rewind only goes backwards |
| `aions-gate advance` | Runs the gates for the next transition and moves forward only on PASS |
| `aions-gate check-config` | Validates `aions-gates.conf` |

Add `--json` to `provenance` or `advance` for machine-readable output.

Exit codes: `0` PASS, `1` FAIL, `2` usage or config error, `3` UNKNOWN.
UNKNOWN never counts as PASS.

## Provenance verdicts

- `PASS`: all 12 fields are present and non-placeholder, TIMESTAMP is
  ISO-8601 with a timezone, and GIT COMMIT is `PENDING`, `UNKNOWN`, `N/A…`
  or a hash that exists in this repository.
- `FAIL`: the reasons are listed.
- `BRAK_HEADERA`: a required file has no header. It blocks for new files;
  for edited legacy files it is reported but does not block.
- `EXCLUDED`: lock, license, binary, JSON, generated, vendor/target, dotfiles,
  or an `exclude` entry in the config.
- `UNKNOWN`: no policy exists for this file type, or the claim cannot be
  verified.

The header must start at the earliest valid line, after a shebang, encoding
line, XML declaration or front matter. `ARCHITECTURE FUNCTION` is accepted as
the legacy spelling of `ARCHITECTURE FUNC`.

## State machine

The states run PRE_FLIGHT → SOURCE_OF_TRUTH_LOADED → TARGET_VERIFIED →
DUPLICATE_SEARCHED → PLAN_VALIDATED → EXECUTION_AUTHORIZED → MUTATION → TEST →
REGRESSION_CHECK → PROVENANCE_CHECK → STATE_UPDATE → LANDING →
POST_FLIGHT_VERIFY → COMPLETE.

Config grammar (`aions-gates.conf` at the repo root):

    gate FROM->TO name = shell command    # exit 0 PASS, 3 UNKNOWN, else FAIL
    open FROM->TO reason                  # explicitly judgment-only transition
    repair name = shell command           # bounded recovery for gate `name`
    max-repair N                          # capped at 5 in code
    timeout SECONDS                       # per command; the whole process group is killed
    exclude path/prefix reason            # provenance opt-out

Rules the config parser enforces:
- Only adjacent forward transitions are allowed; skips are rejected.
- A transition with neither a gate nor an `open` declaration is UNKNOWN.
- A transition cannot be both `open` and gated.

Every gate and repair run is appended to `.aions/evidence.jsonl`, failures
included. State lives in `.aions/state`. Both are local to one clone and
git-ignored.

## Not enforced yet (stated, not hidden)

- TARGET_VERIFIED, DUPLICATE_SEARCHED and PLAN_VALIDATED are `open`: they are
  model judgment, and no mechanical check exists for them.
- Handoff records live in `obrazowanie-stanu` and are validated there by
  `00_CORE_SYSTEM/checks/sprawdz-rejestr.py`. aions-gate does not duplicate
  that validator.
- `crates/darkstar-core` is not built: it has no `Cargo.toml` and references
  a missing `capability_index` module.
